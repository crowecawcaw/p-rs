//! The controlled runtime: machines, inboxes, monitors and the cooperative scheduler.
//!
//! Execution is single threaded. Every machine is an *operation* that runs until it
//! reaches a scheduling point (create, send, receive, default-event check, join or
//! completion). At that point the scheduling strategy picks the next operation, exactly
//! as `OperationScheduler.ScheduleNextEnabledOperation` does upstream, and the chosen
//! machine is resumed from its saved continuation.

use std::collections::VecDeque;
use std::rc::Rc;

use pnext_sema::{
    EventId, FunId, HandlerAction, InterfaceId, MachineId, MachineKind, ModuleInfo, ParamValue, Program, StateId,
    Temperature, VarRole,
};

use crate::bytecode::{CodeBlock, ReceiveCase, Root};
use crate::compile::Code;
use crate::jsonlog::{payload_key, to_dict, Details, JsonLog};
use crate::strategy::{OpInfo, ScheduleStep, ScheduleTrace, Strategy};
use crate::value::{default_value, event_cs_full_name, event_cs_name, MachineRef, Value};

/// An event instance: its declaration plus payload (`Value::Null` when it has none).
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub id: EventId,
    pub payload: Value,
    /// When the event reaches its receiver; only set while time is tracked.
    pub stamp: Option<Rc<crate::time::Stamp>>,
}

/// Marker returned when the current schedule must stop (bug found or execution canceled).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stop;

/// The kinds of scheduling points (`AsyncOperationType`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedKind {
    Start,
    Create,
    Send,
    Receive,
    Default,
    Join,
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpStatus {
    Enabled,
    BlockedOnReceive,
    Completed,
}

#[derive(Debug, Clone)]
pub struct Operation {
    pub id: u64,
    pub name: String,
    pub status: OpStatus,
    pub skip_next_receive: bool,
    /// Index into `Runtime::machines`, `None` for the test task.
    pub machine: Option<usize>,
    /// Kind of the last scheduling point (`AsyncOperation.Type`).
    pub kind: SchedKind,
    /// Target of the last send (`AsyncOperation.MessageReceiver`).
    pub receiver: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineStatus {
    Active,
    Halting,
    Halted,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Transition {
    None,
    Raise(Event),
    Goto(StateId),
    Halt,
}

impl Transition {
    pub fn kind_name(&self) -> &'static str {
        match self {
            Transition::None => "None",
            Transition::Raise(_) => "RaiseEvent",
            Transition::Goto(_) => "GotoState",
            Transition::Halt => "Halt",
        }
    }
}

/// The event inbox of a machine (`EventQueue`).
#[derive(Debug, Default)]
pub struct Inbox {
    pub queue: VecDeque<Event>,
    pub raised: Option<Event>,
    pub closed: bool,
    pub handler_running: bool,
    /// Event types a pending `receive` waits for.
    pub waiting: Option<Rc<[EventId]>>,
}

pub enum EnqueueStatus {
    HandlerRunning,
    HandlerNotRunning,
    NextEventUnavailable,
    Dropped,
}

/// What a machine does next once it is scheduled (its continuation).
#[derive(Debug, Clone)]
pub enum Stage {
    /// Fresh machine: run the start state's entry action with the constructor payload.
    Start(Value),
    /// Entry action of the start state finished; apply its transition and start the loop.
    AfterStartEntry,
    /// Dequeue the next event.
    Loop,
    /// Resumed after the `Default` scheduling point: try to dequeue.
    DequeueTry { has_default: bool },
    /// Resumed after the `Receive` scheduling point following a dequeue.
    Dequeued(Event),
    /// Dispatch an event in the current state (`HandleEventAsync`).
    Handle(Event),
    /// A `do` action finished: apply its transition.
    AfterAction(Event),
    /// The state's exit action finished (goto or pop path).
    AfterExit { e: Event, then: ExitThen },
    /// The `goto ... with` lambda finished.
    AfterWith { e: Event, target: StateId },
    /// Entry action of the new state finished.
    AfterEntry(Event),
    /// The machine halts.
    Halt,
    /// The event handler is not running (machine idle).
    Idle,
}

#[derive(Debug, Clone)]
pub enum ExitThen {
    Goto { target: StateId, with: Option<FunId> },
    Pop,
}

/// Where the interpreter stopped inside an action, waiting for a scheduling point.
#[derive(Debug, Clone)]
pub enum Pending {
    Create { machine: MachineId, iface: InterfaceId, payload: Value },
    Send { target: Rc<MachineRef>, event: Event },
    /// Blocked in `receive`; the matching event is delivered via `Machine::received`.
    ReceiveWait { cases: Rc<[ReceiveCase]>, exit: u32 },
    /// Found the event in the queue; continue after the `Receive` scheduling point.
    ReceiveFound { cases: Rc<[ReceiveCase]>, exit: u32, event: Event },
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub code: Rc<CodeBlock>,
    pub fun: FunId,
    pub pc: usize,
    pub locals: Vec<Value>,
    pub stack: Vec<Value>,
}

/// A `foreach` loop in progress, used to detect modification of the iterated variable.
#[derive(Debug, Clone)]
pub struct ActiveIter {
    pub frame: usize,
    pub slot: u32,
    pub root: Root,
}

/// The interpreter state of one running action.
#[derive(Debug, Clone, Default)]
pub struct Exec {
    pub frames: Vec<Frame>,
    pub pending: Option<Pending>,
    /// Name of the action for diagnostics.
    pub action: String,
    pub iters: Vec<ActiveIter>,
}

#[derive(Debug)]
pub struct Machine {
    pub id: u64,
    pub name: Rc<str>,
    pub decl: MachineId,
    /// The interface the machine was created as (drives the link and monitor maps).
    pub iface: InterfaceId,
    pub fields: Vec<Value>,
    pub state: Option<StateId>,
    pub status: MachineStatus,
    pub inbox: Inbox,
    pub pending_transition: Transition,
    pub goto_payload: Option<Value>,
    /// Target of a pending `GotoStateEvent` in the raised slot.
    pub goto_target: Option<StateId>,
    /// The state popped because it could not handle the current event (for the
    /// `UnhandledEventException` log lines).
    pub popped_from: Option<StateId>,
    pub stage: Stage,
    pub exec: Option<Exec>,
    /// Event delivered to a blocked `receive`.
    pub received: Option<Event>,
    pub this: Rc<MachineRef>,
    /// Local clock (only advanced while time is tracked).
    pub clock: crate::time::Time,
}

#[derive(Debug)]
pub struct Monitor {
    pub decl: MachineId,
    pub fields: Vec<Value>,
    pub state: Option<StateId>,
    pub temperature: usize,
    pub pending_transition: Transition,
    pub goto_payload: Option<Value>,
}

/// The pairwise timeline abstraction of a schedule (`PairwiseLocalRepresentation`):
/// for every receiver, every ordered pair of event labels it dequeued.
#[derive(Debug, Default, Clone)]
pub struct Timeline {
    receivers: Vec<Rc<str>>,
    labels: Vec<Rc<str>>,
    /// Labels seen so far, per receiver index.
    seen: Vec<Vec<u32>>,
    /// `(receiver, previous label, label)` triples.
    tokens: std::collections::HashSet<(u32, u32, u32)>,
}

impl Timeline {
    fn intern(table: &mut Vec<Rc<str>>, s: &str) -> u32 {
        match table.iter().position(|x| &**x == s) {
            Some(i) => i as u32,
            None => {
                table.push(Rc::from(s));
                (table.len() - 1) as u32
            }
        }
    }

    /// `OnDequeueEvent` of the pairwise representation.
    pub fn record(&mut self, receiver: &str, label: &str) {
        let r = Self::intern(&mut self.receivers, receiver);
        let l = Self::intern(&mut self.labels, label);
        if self.seen.len() <= r as usize {
            self.seen.resize(r as usize + 1, Vec::new());
        }
        for prev in &self.seen[r as usize] {
            self.tokens.insert((r, *prev, l));
        }
        if !self.seen[r as usize].contains(&l) {
            self.seen[r as usize].push(l);
        }
    }

    /// The timeline tokens (`receiver\u{1f}prev\u{1f}label\u{1f}`), sorted ordinally.
    pub fn tokens(&self) -> Vec<String> {
        const S: char = '\u{1f}';
        let mut v: Vec<String> = self
            .tokens
            .iter()
            .map(|(r, p, l)| {
                format!("{}{}{}{}{}{}", self.receivers[*r as usize], S, self.labels[*p as usize], S, self.labels[*l as usize], S)
            })
            .collect();
        v.sort();
        v
    }

    /// `GetAbstractTimeline`: the sorted tokens joined with `;`.
    pub fn abstract_timeline(&self) -> String {
        self.tokens().join(";")
    }
}

/// Configuration of one checking run (`CheckerConfiguration`).
#[derive(Debug, Clone)]
pub struct Config {
    pub max_unfair_steps: usize,
    pub max_fair_steps: usize,
    pub liveness_enabled: bool,
    pub liveness_temperature_threshold: usize,
    pub fail_on_max_steps: bool,
    pub strategy_name: String,
    /// Upper bound on the work (interpreted instructions, log lines weighted) an actor may do
    /// without reaching a scheduling point; 0 = unbounded. Upstream has no such bound and
    /// simply hangs on e.g. an endless raise/goto cycle.
    pub max_work_per_step: u64,
    /// Strategy description recorded in the JSON trace (overrides the running strategy's,
    /// used when a buggy schedule is replayed to produce its JSON trace).
    pub strategy_description: Option<String>,
    /// Produce the text log (`false` while exploring: buggy schedules are replayed with
    /// logging on).
    pub logging: bool,
    /// Name of the compiled project's assembly, reported as the source of exceptions
    /// raised by the model's own code.
    pub assembly_name: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            max_unfair_steps: 10000,
            max_fair_steps: 100000,
            liveness_enabled: true,
            liveness_temperature_threshold: 0,
            fail_on_max_steps: false,
            strategy_name: "random".into(),
            max_work_per_step: DEFAULT_MAX_WORK_PER_STEP,
            strategy_description: None,
            logging: true,
            assembly_name: "PImplementation".into(),
        }
    }
}

/// Default for [`Config::max_work_per_step`].
pub const DEFAULT_MAX_WORK_PER_STEP: u64 = 20_000_000;

/// Weight of one log line in the per-step work budget.
pub const LOG_LINE_WORK: u64 = 20;

/// Which actor the interpreter is executing for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Actor {
    Machine(usize),
    Monitor(usize),
}

/// A .NET-style exception raised by P code.
#[derive(Debug, Clone)]
pub struct Exception {
    pub ty: std::borrow::Cow<'static, str>,
    pub message: String,
    /// The .NET assembly upstream reports the exception from (`Exception.Source`); `None`
    /// for the compiled project itself (see [`Config::assembly_name`]).
    pub source: Option<&'static str>,
}

/// `Exception.Source` of exceptions thrown by the .NET base library.
pub const CORE_LIB: &str = "System.Private.CoreLib";
/// `Exception.Source` of exceptions thrown by the P runtime library.
pub const P_CHECKER_CORE: &str = "PCheckerCore";

impl Exception {
    /// An exception with the source upstream reports for its type: collection errors come
    /// from the base library, division by zero from the P runtime, the rest (null
    /// dereferences and casts in generated code, foreign code) from the project.
    pub fn new(ty: impl Into<std::borrow::Cow<'static, str>>, message: impl Into<String>) -> Exception {
        let ty = ty.into();
        let source = match ty.as_ref() {
            "System.Collections.Generic.KeyNotFoundException"
            | "System.ArgumentOutOfRangeException"
            | "System.ArgumentException" => Some(CORE_LIB),
            "System.DivideByZeroException" => Some(P_CHECKER_CORE),
            _ => None,
        };
        Exception { ty, message: message.into(), source }
    }

    pub fn from_source(mut self, source: &'static str) -> Exception {
        self.source = Some(source);
        self
    }
}

pub struct Runtime<'p> {
    pub prog: &'p Program,
    pub code: &'p Code,
    pub module: &'p ModuleInfo,
    pub config: &'p Config,
    pub globals: Vec<Value>,
    pub machines: Vec<Machine>,
    pub monitors: Vec<Monitor>,
    pub ops: Vec<Operation>,
    pub current: u64,
    pub strategy: &'p mut dyn Strategy,
    pub trace: ScheduleTrace,
    pub log: Vec<String>,
    pub timeline: Timeline,
    pub bug: Option<String>,
    pub stopped: bool,
    pub fully_explored: bool,
    pub scheduled_steps: usize,
    /// Work done since the last scheduling point (see [`Config::max_work_per_step`]).
    pub step_work: u64,
    /// The JSON trace, when enabled.
    pub json: Option<JsonLog>,
    /// The strategy description at the time the bug was found.
    pub bug_strategy_description: Option<String>,
    /// Implementations of foreign functions.
    pub foreign: Option<Rc<dyn crate::foreign::ForeignFunctions>>,
    /// The monitors of the test and the interfaces each observes: the module's monitor map
    /// plus every scenario monitor, which upstream attaches to all tests.
    pub monitor_map: Vec<(MachineId, Vec<InterfaceId>)>,
    /// Progress of each attached scenario monitor in this schedule.
    pub scenarios: Vec<ScenarioProgress>,
    /// Whether the text log is produced (see [`Config::logging`]).
    pub logging: bool,
    /// Trace position at which the step bound was hit, if it was.
    pub bound_hit_at: Option<usize>,
    /// Activity coverage being accumulated (enabled when set).
    pub coverage: Option<crate::coverage::CoverageInfo>,
    pub coverage_run: Option<crate::coverage::CoverageRun>,
    pub coverage_tables: Option<Rc<crate::coverage::CoverageTables>>,
    /// Time tracking (see `design/time/PROPOSAL.md`); `None` when the test does not use it.
    pub time: Option<Box<crate::time::TimeState>>,
    next_op_id: u64,
    anon_names: Vec<String>,
}

impl<'p> Runtime<'p> {
    pub fn new(
        prog: &'p Program,
        code: &'p Code,
        module: &'p ModuleInfo,
        config: &'p Config,
        strategy: &'p mut dyn Strategy,
        assignment: Option<&pnext_sema::ParamAssignment>,
    ) -> Runtime<'p> {
        let mut globals: Vec<Value> =
            prog.global_params.iter().map(|v| default_value(&prog.var(*v).ty, prog)).collect();
        if let Some(a) = assignment {
            for (v, val) in &a.values {
                let slot = prog.var(*v).slot as usize;
                if slot < globals.len() {
                    globals[slot] = match val {
                        ParamValue::Int(i) => Value::Int(*i),
                        ParamValue::Bool(b) => Value::Bool(*b),
                    };
                }
            }
        }
        // Anonymous functions are named Anon, Anon_1, ... by the upstream name manager.
        let mut anon_names = Vec::with_capacity(prog.funs.len());
        let mut n = 0;
        for f in &prog.funs {
            if f.is_anon() {
                anon_names.push(if n == 0 { "Anon".to_string() } else { format!("Anon_{}", n) });
                n += 1;
            } else {
                anon_names.push(f.name.clone());
            }
        }
        Runtime {
            prog,
            code,
            module,
            config,
            globals,
            machines: Vec::new(),
            monitors: Vec::new(),
            ops: Vec::new(),
            current: 0,
            strategy,
            trace: ScheduleTrace::default(),
            log: Vec::new(),
            timeline: Timeline::default(),
            bug: None,
            stopped: false,
            fully_explored: false,
            scheduled_steps: 0,
            step_work: 0,
            json: None,
            bug_strategy_description: None,
            foreign: None,
            monitor_map: monitor_map_with_scenarios(prog, module),
            scenarios: Vec::new(),
            logging: config.logging,
            bound_hit_at: None,
            coverage: None,
            coverage_run: None,
            coverage_tables: None,
            time: None,
            next_op_id: 0,
            anon_names,
        }
    }

    pub fn fun_name(&self, f: FunId) -> &str {
        &self.anon_names[f.idx()]
    }

    pub fn log_line(&mut self, s: String) {
        self.step_work += LOG_LINE_WORK;
        self.log.push(s);
    }

    // ---- names ------------------------------------------------------------

    pub fn state_name(&self, s: Option<StateId>) -> String {
        match s {
            Some(s) => self.prog.state(s).name.clone(),
            None => String::new(),
        }
    }

    pub fn machine_state_name(&self, m: usize) -> String {
        self.state_name(self.machines[m].state)
    }

    /// The state name for log lines only (empty when neither log is produced).
    pub fn log_state_name(&self, m: usize) -> String {
        if self.logging || self.json_enabled() {
            self.machine_state_name(m)
        } else {
            String::new()
        }
    }

    pub fn monitor_full_name(&self, mon: usize) -> String {
        format!("PImplementation.{}", self.prog.machine(self.monitors[mon].decl).name)
    }

    pub fn event_name(&self, e: EventId) -> &str {
        if e == GOTO_EVENT {
            return "GotoStateEvent";
        }
        event_cs_name(self.prog, e)
    }

    /// `GetEventNameWithPayload`.
    pub fn event_with_payload(&self, e: &Event) -> String {
        let name = self.event_name(e.id);
        if e.payload.is_null() {
            name.to_string()
        } else {
            format!("{} with payload ({})", name, e.payload.to_escaped(self.prog))
        }
    }

    // ---- JSON trace ------------------------------------------------------------

    /// Adds a JSON trace entry (no-op unless the JSON trace is enabled).
    pub fn jlog(&mut self, ty: &str, d: Details, update_vc: bool) {
        if let Some(j) = &mut self.json {
            j.add(ty, d, update_vc);
        }
    }

    pub fn json_enabled(&self) -> bool {
        self.json.is_some()
    }

    /// `(payload ToDict, payload identity)` of an event for the JSON trace.
    pub fn jpayload(&self, e: &Event) -> (Option<crate::jsonlog::Json>, String) {
        if e.id == GOTO_EVENT || e.payload.is_null() {
            return (None, String::new());
        }
        (Some(to_dict(&e.payload, self.prog)), payload_key(Some(&e.payload), self.prog))
    }

    /// Short (unqualified) state name.
    pub fn short_state(&self, s: Option<StateId>) -> String {
        let n = self.state_name(s);
        n.rsplit('.').next().unwrap_or(&n).to_string()
    }

    // ---- assertions and bugs ------------------------------------------------

    /// `OperationScheduler.NotifyAssertionFailure`.
    pub fn report_bug(&mut self, text: String) -> Stop {
        if self.bug.is_none() {
            if self.logging { self.log_line(format!("<ErrorLog> {}", text)); } else { self.step_work += LOG_LINE_WORK; }
            if self.logging { self.log_line(format!("<StrategyLog> Found bug using '{}' strategy.", self.config.strategy_name)); } else { self.step_work += LOG_LINE_WORK; }
            let desc = self.config.strategy_description.clone().unwrap_or_else(|| self.strategy.description());
            self.bug_strategy_description = Some(desc.clone());
            if self.json_enabled() {
                self.jlog(
                    "AssertionFailure",
                    Details { log: Some(text.clone()), error: Some(text.clone()), ..Default::default() },
                    false,
                );
                let suffix = if desc.is_empty() { " Description: " } else { "" };
                self.jlog(
                    "StrategyDescription",
                    Details {
                        log: Some(format!("Found bug using '{}' strategy.{}", self.config.strategy_name, suffix)),
                        strategy: Some(self.config.strategy_name.clone()),
                        strategy_description: Some(desc),
                        ..Default::default()
                    },
                    false,
                );
            }
            self.bug = Some(text);
        }
        self.stopped = true;
        Stop
    }

    /// Reports an unhandled .NET exception thrown inside an action.
    pub fn report_exception(&mut self, actor: Actor, exc: &Exception, action: &str) -> Stop {
        if let Actor::Machine(m) = actor {
            let short = exc.ty.rsplit('.').next().unwrap_or(&exc.ty).to_string();
            let body = format!(
                "{} running action '{}' in state '{}' threw exception '{}'.",
                self.machines[m].name,
                action,
                self.machine_state_name(m),
                short
            );
            if self.logging { self.log_line(format!("<ExceptionLog> {}", body)); } else { self.step_work += LOG_LINE_WORK; }
            if self.json_enabled() {
                let d = Details {
                    log: Some(body),
                    id: Some(self.machines[m].name.to_string()),
                    state: Some(self.short_state(self.machines[m].state)),
                    action: Some(action.to_string()),
                    exception: Some(short.to_string()),
                    ..Default::default()
                };
                self.jlog("ExceptionThrown", d, true);
            }
        }
        let ctx = match actor {
            Actor::Machine(m) => {
                let state = match self.machines[m].state {
                    Some(_) => self.machine_state_name(m),
                    None => "<unknown>".to_string(),
                };
                format!("{} (state '{}', action '{}')", self.machines[m].name, state, action)
            }
            Actor::Monitor(mon) => {
                let state = self.state_name(self.monitors[mon].state);
                format!("{} (state '{}', action '{}')", self.monitor_full_name(mon), state, action)
            }
        };
        let text = format!(
            "Exception '{}' was thrown in {}: {}\nfrom location '{}':\nThe stack trace is:\n",
            exc.ty,
            ctx,
            exc.message,
            exc.source.unwrap_or(&self.config.assembly_name)
        );
        self.report_bug(text)
    }

    // ---- operations ----------------------------------------------------------

    fn op_index(&self, id: u64) -> usize {
        self.ops.iter().position(|o| o.id == id).expect("unknown operation")
    }

    pub fn op_mut(&mut self, id: u64) -> &mut Operation {
        let i = self.op_index(id);
        &mut self.ops[i]
    }

    fn new_op(&mut self, name: String, machine: Option<usize>) -> u64 {
        let id = self.next_op_id;
        self.next_op_id += 1;
        self.ops.push(Operation {
            id,
            name,
            status: OpStatus::Enabled,
            skip_next_receive: false,
            machine,
            kind: SchedKind::Start,
            receiver: None,
        });
        id
    }

    // ---- scheduling ------------------------------------------------------------

    fn check_steps_bound(&mut self) -> Result<(), Stop> {
        if self.strategy.has_reached_max_steps() {
            self.bound_hit_at = Some(self.trace.steps.len());
            let bound = if self.strategy.is_fair() { self.config.max_fair_steps } else { self.config.max_unfair_steps };
            let msg = format!("Scheduling steps bound of {} reached.", bound);
            if self.config.fail_on_max_steps {
                return Err(self.report_bug(msg));
            }
            self.stopped = true;
            return Err(Stop);
        }
        Ok(())
    }

    fn check_liveness_temperatures(&mut self) -> Result<(), Stop> {
        if !self.config.liveness_enabled || !self.strategy.is_fair() {
            return Ok(());
        }
        let threshold = self.config.liveness_temperature_threshold;
        if threshold == 0 {
            return Ok(());
        }
        for i in 0..self.monitors.len() {
            if self.prog.machine(self.monitors[i].decl).is_scenario() {
                continue;
            }
            let hot = self.monitors[i]
                .state
                .map(|s| self.prog.state(s).temperature == Temperature::Hot)
                .unwrap_or(false);
            if hot {
                self.monitors[i].temperature += 1;
                if self.monitors[i].temperature > threshold {
                    let msg = format!(
                        "{} detected potential liveness bug in hot state '{}'.",
                        self.monitor_full_name(i),
                        self.state_name(self.monitors[i].state)
                    );
                    return Err(self.report_bug(msg));
                }
            }
        }
        Ok(())
    }

    /// `ScheduleNextEnabledOperation`: picks the operation to run next.
    pub fn schedule_next(&mut self, kind: SchedKind) -> Result<(), Stop> {
        let cur = self.current;
        self.op_mut(cur).kind = kind;
        self.check_steps_bound()?;
        self.check_liveness_temperatures()?;
        let infos: Vec<OpInfo> = self
            .ops
            .iter()
            .map(|o| OpInfo { id: o.id, enabled: o.status == OpStatus::Enabled, kind: o.kind, receiver: o.receiver })
            .collect();
        let mut infos = infos;
        if self.time.as_ref().is_some_and(|t| t.opts.timed) {
            self.filter_timed(&mut infos);
        }
        let current = self.current;
        match self.strategy.next_operation(current, &infos) {
            Some(next) => {
                self.step_work = 0;
                self.scheduled_steps = self.strategy.scheduled_steps();
                self.trace.steps.push(ScheduleStep::Operation(next));
                self.current = next;
                Ok(())
            }
            None => {
                self.check_deadlock()?;
                self.fully_explored = true;
                self.stopped = true;
                Err(Stop)
            }
        }
    }

    fn check_deadlock(&mut self) -> Result<(), Stop> {
        let blocked: Vec<String> =
            self.ops.iter().filter(|o| o.status == OpStatus::BlockedOnReceive).map(|o| o.name.clone()).collect();
        if blocked.is_empty() {
            return Ok(());
        }
        let mut msg = String::from("Deadlock detected.");
        let n = blocked.len();
        for (i, name) in blocked.iter().enumerate() {
            msg.push(' ');
            msg.push_str(name);
            if i + 2 == n {
                msg.push_str(" and");
            } else if i + 1 < n {
                msg.push(',');
            }
        }
        msg.push_str(if n == 1 { " is " } else { " are " });
        msg.push_str("waiting to receive an event, but no other controlled tasks are enabled.");
        Err(self.report_bug(msg))
    }

    /// `GetNondeterministicBooleanChoice`.
    pub fn choose_bool(&mut self, max: i32, caller: &str) -> Result<bool, Stop> {
        self.check_steps_bound()?;
        self.check_liveness_temperatures()?;
        let b = self.strategy.next_bool(max);
        self.scheduled_steps = self.strategy.scheduled_steps();
        self.trace.steps.push(ScheduleStep::Bool(b));
        if self.logging { self.log_line(format!(
            "<RandomLog> {} nondeterministically chose '{}'.",
            caller,
            if b { "True" } else { "False" }
        )); } else { self.step_work += LOG_LINE_WORK; }
        Ok(b)
    }

    /// `GetNondeterministicIntegerChoice`.
    pub fn choose_int(&mut self, max: i32, caller: &str) -> Result<i32, Stop> {
        self.check_steps_bound()?;
        self.check_liveness_temperatures()?;
        let i = self.strategy.next_int(max);
        self.scheduled_steps = self.strategy.scheduled_steps();
        self.trace.steps.push(ScheduleStep::Int(i));
        if self.logging { self.log_line(format!("<RandomLog> {} nondeterministically chose '{}'.", caller, i)); } else { self.step_work += LOG_LINE_WORK; }
        Ok(i)
    }

    // ---- test driver ---------------------------------------------------------------

    /// Runs one schedule of the test whose main machine is `main`.
    pub fn run_test(&mut self, main: MachineId, test_name: &str) {
        if self.logging { self.log_line(format!("<TestLog> Running test '{}'.", test_name)); } else { self.step_work += LOG_LINE_WORK; }
        let task = self.new_op("Task(0)".into(), None);
        self.current = task;
        let r = (|| -> Result<(), Stop> {
            self.create_monitors()?;
            // The test task creates the main machine.
            self.schedule_next(SchedKind::Create)?;
            let iface = self.prog.machine(main).interface.unwrap_or(InterfaceId(0));
            let name = self.prog.machine(main).name.clone();
            let (midx, _) = self.instantiate_machine(main, iface, &name, Value::Null, crate::time::Time::default());
            let creator = format!("task '{}'", 1);
            let mname = self.machines[midx].name.clone();
            if self.logging { self.log_line(format!("<CreateLog> {} was created by {}.", mname, creator)); } else { self.step_work += LOG_LINE_WORK; }
            if self.json_enabled() {
                let d = Details {
                    log: Some(format!("{} was created by {}.", mname, creator)),
                    id: Some(mname.to_string()),
                    ..Default::default()
                };
                self.jlog("CreateStateMachine", d, true);
            }
            self.start_handler(midx);
            self.op_mut(task).status = OpStatus::Completed;
            self.schedule_next(SchedKind::Stop)?;
            self.run_loop()
        })();
        let _ = r;
    }

    fn run_loop(&mut self) -> Result<(), Stop> {
        loop {
            if self.stopped {
                return Err(Stop);
            }
            let cur = self.current;
            let Some(m) = self.ops[self.op_index(cur)].machine else {
                // The test task is never rescheduled after completion.
                self.stopped = true;
                return Err(Stop);
            };
            match self.run_machine(m)? {
                SchedKind::Stop => {
                    let op_id = self.machines[m].id;
                    {
                        let op = self.op_mut(op_id);
                        op.status = OpStatus::Completed;
                        op.skip_next_receive = true;
                    }
                    self.schedule_next(SchedKind::Stop)?;
                }
                kind => self.schedule_next(kind)?,
            }
        }
    }

    /// Checks monitors for liveness violations once a schedule is fully explored.
    pub fn check_hot_monitors_at_termination(&mut self) {
        if !self.fully_explored || self.bug.is_some() {
            return;
        }
        for i in 0..self.monitors.len() {
            if self.prog.machine(self.monitors[i].decl).is_scenario() {
                continue;
            }
            if let Some(s) = self.monitors[i].state {
                if self.prog.state(s).temperature == Temperature::Hot {
                    let msg = format!(
                        "{} detected liveness bug in hot state '{}' at the end of program execution.",
                        self.monitor_full_name(i),
                        self.prog.state(s).name
                    );
                    let _ = self.report_bug(msg);
                    return;
                }
            }
        }
    }

    // ---- machine creation -------------------------------------------------------------

    fn instantiate_machine(&mut self, decl: MachineId, iface: InterfaceId, name: &str, payload: Value, clock: crate::time::Time) -> (usize, u64) {
        self.cov_declare(decl, false);
        let idx = self.machines.len();
        let id = self.next_op_id;
        let display: Rc<str> = Rc::from(format!("{}({})", name, id));
        let permissions: Rc<[EventId]> = self.code.permissions[iface.idx()].clone();
        let this = Rc::new(MachineRef { id, name: display.clone(), permissions });
        let fields = self
            .prog
            .machine(decl)
            .fields
            .iter()
            .map(|v| default_value(&self.prog.var(*v).ty, self.prog))
            .collect();
        let start = self.prog.machine(decl).start_state;
        self.machines.push(Machine {
            id,
            name: display.clone(),
            decl,
            iface,
            fields,
            state: start,
            status: MachineStatus::Active,
            inbox: Inbox { handler_running: true, ..Default::default() },
            pending_transition: Transition::None,
            goto_payload: None,
            goto_target: None,
            popped_from: None,
            stage: Stage::Start(payload),
            exec: None,
            received: None,
            this,
            clock,
        });
        let op = self.new_op(display.to_string(), Some(idx));
        debug_assert_eq!(op, id);
        (idx, id)
    }

    /// `RunStateMachineEventHandler`: enables the machine's operation.
    fn start_handler(&mut self, m: usize) {
        let id = self.machines[m].id;
        self.op_mut(id).status = OpStatus::Enabled;
    }

    /// Completes a `new I(...)` once the `Create` scheduling point has passed.
    pub fn finish_create(&mut self, creator: usize, machine: MachineId, iface: InterfaceId, payload: Value) -> Value {
        let name = self.prog.interface(iface).name.clone();
        let clock = self.machines[creator].clock.clone();
        let (idx, _) = self.instantiate_machine(machine, iface, &name, payload, clock);
        let created = self.machines[idx].name.clone();
        let creator_name = self.machines[creator].name.clone();
        if self.logging { self.log_line(format!("<CreateLog> {} was created by {}.", created, creator_name)); } else { self.step_work += LOG_LINE_WORK; }
        if self.json_enabled() {
            let d = Details {
                log: Some(format!("{} was created by {}.", created, creator_name)),
                id: Some(created.to_string()),
                creator_name: Some(creator_name.to_string()),
                creator_type: Some(self.prog.machine(self.machines[creator].decl).name.clone()),
                ..Default::default()
            };
            self.jlog("CreateStateMachine", d, true);
        }
        self.start_handler(idx);
        Value::Machine(self.machines[idx].this.clone())
    }

    pub fn find_machine(&self, id: u64) -> Option<usize> {
        self.machines.iter().position(|m| m.id == id)
    }

    // ---- sending --------------------------------------------------------------------

    /// Completes a `send` once the `Send` scheduling point has passed.
    pub fn finish_send(&mut self, sender: usize, target: &MachineRef, event: Event) {
        if self.coverage.is_some() {
            if let Some(t) = self.find_machine(target.id) {
                let decl = self.machines[t].decl;
                self.cov_send(sender, decl, event.id);
            }
        }
        let sender_name = self.machines[sender].name.clone();
        let sender_state = self.log_state_name(sender);
        let ev = if self.logging || self.json_enabled() { self.event_with_payload(&event) } else { String::new() };
        let tidx = self.find_machine(target.id);
        let halted = tidx.map(|t| self.machines[t].status == MachineStatus::Halted).unwrap_or(true);
        if self.json_enabled() {
            let (payload, key) = self.jpayload(&event);
            let d = Details {
                log: Some(format!(
                    "'{}' in state '{}' sent event '{}' to '{}'{}.",
                    sender_name,
                    sender_state,
                    ev,
                    target.name,
                    if halted { " which has halted" } else { "" }
                )),
                state: Some(self.short_state(self.machines[sender].state)),
                event: Some(self.event_name(event.id).to_string()),
                payload,
                sender: Some(sender_name.to_string()),
                target: Some(target.name.to_string()),
                is_target_halted: Some(halted),
                payload_key: key,
                ..Default::default()
            };
            self.jlog("SendEvent", d, true);
        }
        if halted {
            if self.logging { self.log_line(format!(
                "<SendLog> '{}' in state '{}' sent event '{}' to '{}' which has halted.",
                sender_name, sender_state, ev, target.name
            )); } else { self.step_work += LOG_LINE_WORK; }
            return;
        }
        if self.logging { self.log_line(format!(
            "<SendLog> '{}' in state '{}' sent event '{}' to '{}'.",
            sender_name, sender_state, ev, target.name
        )); } else { self.step_work += LOG_LINE_WORK; }
        let t = tidx.unwrap();
        let mut event = event;
        if self.time.is_some() {
            event.stamp = Some(Rc::new(self.send_stamp(sender, t, event.id)));
        }
        if let EnqueueStatus::HandlerNotRunning = self.enqueue(t, event) {
            self.start_handler(t);
        }
    }

    /// `EventQueue.Enqueue`.
    pub fn enqueue(&mut self, t: usize, e: Event) -> EnqueueStatus {
        if self.machines[t].status == MachineStatus::Halted || self.machines[t].inbox.closed {
            return EnqueueStatus::Dropped;
        }
        let waiting = self.machines[t].inbox.waiting.clone();
        if waiting.is_some_and(|w| w.contains(&e.id)) {
            if self.timed() {
                // the receiver picks among its queued events when it resumes
                self.machines[t].inbox.waiting = None;
                self.machines[t].inbox.queue.push_back(e);
                let id = self.machines[t].id;
                self.op_mut(id).status = OpStatus::Enabled;
                return EnqueueStatus::HandlerRunning;
            }
            {
                self.machines[t].inbox.waiting = None;
                let name = self.machines[t].name.clone();
                let state = self.log_state_name(t);
                let ev = if self.logging || self.json_enabled() { self.event_with_payload(&e) } else { String::new() };
                if self.logging { self.log_line(format!(
                    "<ReceiveLog> '{}' dequeued event '{}' and unblocked in state '{}'.",
                    name, ev, state
                )); } else { self.step_work += LOG_LINE_WORK; }
                if self.json_enabled() {
                    let (payload, key) = self.jpayload(&e);
                    let d = Details {
                        log: Some(format!("'{}' dequeued event '{}' and unblocked in state '{}'.", name, ev, state)),
                        id: Some(name.to_string()),
                        state: Some(self.short_state(self.machines[t].state)),
                        event: Some(self.event_name(e.id).to_string()),
                        payload,
                        was_blocked: Some(true),
                        payload_key: key,
                        ..Default::default()
                    };
                    self.jlog("ReceiveEvent", d, true);
                }
                self.cov_receive(t, e.id);
                self.machines[t].received = Some(e);
                let id = self.machines[t].id;
                self.op_mut(id).status = OpStatus::Enabled;
                return EnqueueStatus::HandlerRunning;
            }
        }
        self.machines[t].inbox.queue.push_back(e);
        if !self.machines[t].inbox.handler_running {
            if self.try_dequeue(t, true).is_none() {
                return EnqueueStatus::NextEventUnavailable;
            }
            self.machines[t].inbox.handler_running = true;
            return EnqueueStatus::HandlerNotRunning;
        }
        EnqueueStatus::HandlerRunning
    }

    fn handler_action(&self, m: usize, e: EventId) -> Option<&HandlerAction> {
        let state = self.machines[m].state?;
        self.prog.state(state).handler_for(e).map(|h| &h.action)
    }

    fn is_ignored(&self, m: usize, e: EventId) -> bool {
        matches!(self.handler_action(m, e), Some(HandlerAction::Ignore))
    }

    fn is_deferred(&self, m: usize, e: EventId) -> bool {
        matches!(self.handler_action(m, e), Some(HandlerAction::Defer))
    }

    fn has_default_handler(&self, m: usize) -> bool {
        self.handler_action(m, self.prog.null_event).is_some()
    }

    /// `EventQueue.TryDequeueEvent`.
    fn try_dequeue(&mut self, m: usize, check_only: bool) -> Option<Event> {
        let mut i = 0;
        while i < self.machines[m].inbox.queue.len() {
            let id = self.machines[m].inbox.queue[i].id;
            if self.is_ignored(m, id) {
                if check_only {
                    i += 1;
                } else {
                    self.machines[m].inbox.queue.remove(i);
                }
                continue;
            }
            if !self.is_deferred(m, id) {
                if check_only {
                    return self.machines[m].inbox.queue.get(i).cloned();
                }
                return self.machines[m].inbox.queue.remove(i);
            }
            i += 1;
        }
        None
    }

    // ---- announce -----------------------------------------------------------------

    /// `AnnounceInternal`: delivers an event to the monitors observing the sender's interface.
    pub fn announce_internal(&mut self, sender: usize, e: &Event) -> Result<(), Stop> {
        let iface = self.machines[sender].iface;
        let mut targets = Vec::new();
        for (mon, ifaces) in &self.monitor_map {
            if ifaces.contains(&iface) {
                if let Some(i) = self.monitors.iter().position(|m| m.decl == *mon) {
                    targets.push(i);
                }
            }
        }
        for i in targets {
            if self.prog.machine(self.monitors[i].decl).observes.contains(&e.id) {
                self.monitor_event(i, e.clone(), Some(sender))?;
            }
        }
        Ok(())
    }

    // ---- machine lifecycle -----------------------------------------------------------

    /// Runs machine `m` until it reaches a scheduling point. Returns the kind of point
    /// (`SchedKind::Stop` when the event handler completed).
    fn run_machine(&mut self, m: usize) -> Result<SchedKind, Stop> {
        // Still at the scheduling point this step started from (`--timed` picks events here).
        let mut at_point = true;
        loop {
            // Resume or run the current action, if any.
            if let Some(exec) = self.machines[m].exec.take() {
                at_point = false;
                match self.run_exec(Actor::Machine(m), exec)? {
                    ExecOutcome::Finished(_) | ExecOutcome::Unwound => {}
                    ExecOutcome::Yield(exec, kind) => {
                        self.machines[m].exec = Some(exec);
                        return Ok(kind);
                    }
                }
            }
            let stage = std::mem::replace(&mut self.machines[m].stage, Stage::Idle);
            if !matches!(stage, Stage::Idle | Stage::Loop | Stage::DequeueTry { .. }) {
                at_point = false;
            }
            match stage {
                Stage::Start(payload) => {
                    let e = Event { id: self.prog.null_event, payload, stamp: None };
                    self.machines[m].stage = Stage::AfterStartEntry;
                    self.enter_state(m, e)?;
                }
                Stage::AfterStartEntry => {
                    self.apply_transition(m)?;
                    if self.machines[m].status == MachineStatus::Halting {
                        self.machines[m].stage = Stage::Halt;
                    } else {
                        self.machines[m].stage = Stage::Loop;
                    }
                }
                Stage::Loop => {
                    if self.machines[m].status == MachineStatus::Halting {
                        self.machines[m].stage = Stage::Halt;
                        continue;
                    }
                    if let Some(raised) = self.machines[m].inbox.raised.take() {
                        if raised.id == GOTO_EVENT || !self.is_ignored(m, raised.id) {
                            self.cov_handle_raised(m, raised.id);
                            self.machines[m].stage = Stage::Handle(raised);
                            continue;
                        }
                    }
                    let has_default = self.has_default_handler(m);
                    self.machines[m].stage = Stage::DequeueTry { has_default };
                    if self.timed() {
                        // pick the event at a scheduling point, once earlier senders have run
                        if !at_point && (has_default || self.timed_pick(m, None).is_some()) {
                            return Ok(if has_default { SchedKind::Default } else { SchedKind::Receive });
                        }
                    } else if has_default {
                        return Ok(SchedKind::Default);
                    }
                }
                Stage::DequeueTry { has_default } => {
                    let taken = if self.timed() { self.timed_take(m, has_default) } else { self.try_dequeue(m, false) };
                    if let Some(e) = taken.as_ref().filter(|_| self.timed()) {
                        self.machines[m].stage = Stage::Dequeued(e.clone());
                        continue;
                    }
                    if let Some(e) = taken {
                        let id = self.machines[m].id;
                        let op = self.op_mut(id);
                        if op.skip_next_receive {
                            op.skip_next_receive = false;
                            self.machines[m].stage = Stage::Dequeued(e);
                        } else {
                            self.machines[m].stage = Stage::Dequeued(e);
                            return Ok(SchedKind::Receive);
                        }
                    } else if has_default {
                        let name = self.machines[m].name.clone();
                        let state = self.log_state_name(m);
                        let e = Event { id: self.prog.null_event, payload: Value::Null, stamp: None };
                        self.machines[m].stage = Stage::Handle(e);
                        self.cov_default(m);
                        if self.logging { self.log_line(format!(
                            "<DefaultLog> {} is executing the default handler in state '{}'.",
                            name, state
                        )); } else { self.step_work += LOG_LINE_WORK; }
                        if self.json_enabled() {
                            let d = Details {
                                log: Some(format!("{} is executing the default handler in state '{}'.", name, state)),
                                id: Some(name.to_string()),
                                state: Some(self.short_state(self.machines[m].state)),
                                ..Default::default()
                            };
                            self.jlog("DefaultEventHandler", d, true);
                        }
                        return Ok(SchedKind::Receive);
                    } else {
                        self.machines[m].inbox.handler_running = false;
                        self.machines[m].stage = Stage::Idle;
                        return Ok(SchedKind::Stop);
                    }
                }
                Stage::Dequeued(e) => {
                    self.cov_dequeue(m, e.id);
                    let name = self.machines[m].name.clone();
                    let state = self.log_state_name(m);
                    let ev = if self.logging || self.json_enabled() { self.event_with_payload(&e) } else { String::new() };
                    if self.logging { self.log_line(format!("<DequeueLog> '{}' dequeued event '{}' in state '{}'.", name, ev, state)); } else { self.step_work += LOG_LINE_WORK; }
                    if self.json_enabled() {
                        let (payload, key) = self.jpayload(&e);
                        let d = Details {
                            log: Some(format!("'{}' dequeued event '{}' in state '{}'.", name, ev, state)),
                            id: Some(name.to_string()),
                            event: Some(self.event_name(e.id).to_string()),
                            state: Some(self.short_state(self.machines[m].state)),
                            payload,
                            payload_key: key,
                            ..Default::default()
                        };
                        self.jlog("DequeueEvent", d, true);
                    }
                    let label = event_cs_name(self.prog, e.id);
                    self.timeline.record(&name, label);
                    self.time_receive(m, &e);
                    self.machines[m].stage = Stage::Handle(e);
                }
                Stage::Handle(e) => {
                    if self.machines[m].status != MachineStatus::Active {
                        self.machines[m].stage = Stage::Halt;
                        continue;
                    }
                    if e.id == GOTO_EVENT {
                        let target = self.machines[m].goto_target.take().expect("goto target");
                        self.begin_goto(m, target, None, e)?;
                        continue;
                    }
                    let Some(state) = self.machines[m].state else {
                        // UnhandledEventException, logged by OnUnhandledEventExceptionHandler
                        let name = self.machines[m].name.clone();
                        let popped = self.machines[m].popped_from.take();
                        let from = self.state_name(popped);
                        if self.logging { self.log_line(format!(
                            "<ExceptionLog> {} running action '' in state '{}' threw exception 'UnhandledEventException'.",
                            name, from
                        )); } else { self.step_work += LOG_LINE_WORK; }
                        if self.json_enabled() {
                            let d = Details {
                                log: Some(format!(
                                    "{} running action '' in state '{}' threw exception 'UnhandledEventException'.",
                                    name, from
                                )),
                                id: Some(name.to_string()),
                                state: Some(from.clone()),
                                action: Some(String::new()),
                                exception: Some("UnhandledEventException".into()),
                                ..Default::default()
                            };
                            self.jlog("ExceptionThrown", d, true);
                        }
                        if e.id == self.prog.halt_event {
                            if self.logging { self.log_line(format!(
                                "<ExceptionLog> {} running action '' in state '{}' chose to handle exception 'UnhandledEventException'.",
                                name, from
                            )); } else { self.step_work += LOG_LINE_WORK; }
                            if self.json_enabled() {
                                let d = Details {
                                    log: Some(format!(
                                        "{} running action '' in state '{}' chose to handle exception 'UnhandledEventException'.",
                                        name, from
                                    )),
                                    id: Some(name.to_string()),
                                    state: Some(from.clone()),
                                    action: Some(String::new()),
                                    exception: Some("UnhandledEventException".into()),
                                    ..Default::default()
                                };
                                self.jlog("ExceptionHandled", d, true);
                            }
                            self.machines[m].status = MachineStatus::Halting;
                            self.machines[m].stage = Stage::Halt;
                            continue;
                        }
                        let msg = format!(
                            "{} received event '{}' that cannot be handled.",
                            self.machines[m].name,
                            event_cs_full_name(self.prog, e.id)
                        );
                        return Err(self.report_bug(msg));
                    };
                    let action = self.prog.state(state).handler_for(e.id).map(|h| h.action.clone());
                    match action {
                        Some(HandlerAction::Do(f)) => {
                            self.machines[m].stage = Stage::AfterAction(e.clone());
                            self.start_action(m, f, &e);
                        }
                        Some(HandlerAction::Goto { target, with }) => {
                            self.begin_goto(m, target, with, e)?;
                        }
                        Some(HandlerAction::Ignore) | Some(HandlerAction::Defer) => {
                            self.machines[m].stage = Stage::Loop;
                        }
                        None => {
                            // Unhandled: run the exit action, then pop the state.
                            self.machines[m].stage = Stage::AfterExit { e: e.clone(), then: ExitThen::Pop };
                            self.exit_state(m, e)?;
                        }
                    }
                }
                Stage::AfterAction(_e) => {
                    self.apply_transition(m)?;
                    self.machines[m].stage = Stage::Loop;
                }
                Stage::AfterExit { e, then } => {
                    // The exit action may only raise `halt` or nothing.
                    let t = std::mem::replace(&mut self.machines[m].pending_transition, Transition::None);
                    match &t {
                        Transition::None | Transition::Halt => {}
                        other => {
                            let msg = format!(
                                "{} has performed a '{}' transition from an OnExit action.",
                                self.machines[m].name,
                                other.kind_name()
                            );
                            return Err(self.report_bug(msg));
                        }
                    }
                    if matches!(t, Transition::Halt) {
                        self.machines[m].status = MachineStatus::Halting;
                    }
                    match then {
                        ExitThen::Goto { target, with } => {
                            if let (Some(w), MachineStatus::Active) = (with, self.machines[m].status) {
                                self.machines[m].stage = Stage::AfterWith { e: e.clone(), target };
                                self.start_action(m, w, &e);
                            } else {
                                self.machines[m].stage = Stage::AfterWith { e, target };
                            }
                        }
                        ExitThen::Pop => {
                            if self.machines[m].status == MachineStatus::Active {
                                let name = self.machines[m].name.clone();
                                let state = self.log_state_name(m);
                                if self.logging { self.log_line(format!(
                                    "<PopLog> {} popped state {} due to unhandled event '{}'.",
                                    name,
                                    state,
                                    self.event_name(e.id)
                                )); } else { self.step_work += LOG_LINE_WORK; }
                                if self.json_enabled() {
                                    let d = Details {
                                        log: Some(format!(
                                            "{} popped state {} due to unhandled event '{}'.",
                                            name,
                                            state,
                                            self.event_name(e.id)
                                        )),
                                        id: Some(name.to_string()),
                                        state: Some(state.clone()),
                                        event: Some(self.event_name(e.id).to_string()),
                                        ..Default::default()
                                    };
                                    self.jlog("PopStateUnhandledEvent", d, true);
                                }
                                self.machines[m].popped_from = self.machines[m].state;
                                self.machines[m].state = None;
                                self.machines[m].stage = Stage::Handle(e);
                            } else {
                                self.machines[m].stage = Stage::Halt;
                            }
                        }
                    }
                }
                Stage::AfterWith { e, target } => {
                    let t = std::mem::replace(&mut self.machines[m].pending_transition, Transition::None);
                    match &t {
                        Transition::None | Transition::Halt => {}
                        other => {
                            let msg = format!(
                                "{} has performed a '{}' transition from an OnExit action.",
                                self.machines[m].name,
                                other.kind_name()
                            );
                            return Err(self.report_bug(msg));
                        }
                    }
                    if matches!(t, Transition::Halt) {
                        self.machines[m].status = MachineStatus::Halting;
                    }
                    if self.machines[m].status == MachineStatus::Active {
                        self.machines[m].state = Some(target);
                        self.machines[m].stage = Stage::AfterEntry(e.clone());
                        self.enter_state(m, e)?;
                    } else {
                        self.machines[m].stage = Stage::Halt;
                    }
                }
                Stage::AfterEntry(_e) => {
                    self.apply_transition(m)?;
                    self.machines[m].stage = Stage::Loop;
                }
                Stage::Halt => {
                    self.machines[m].status = MachineStatus::Halted;
                    self.machines[m].inbox.closed = true;
                    let n = self.machines[m].inbox.queue.len();
                    let name = self.machines[m].name.clone();
                    if self.logging { self.log_line(format!("<HaltLog> {} halted with {} events in its inbox.", name, n)); } else { self.step_work += LOG_LINE_WORK; }
                    if self.json_enabled() {
                        let d = Details {
                            log: Some(format!("{} halted with {} events in its inbox.", name, n)),
                            id: Some(name.to_string()),
                            halt_inbox_size: Some(n as i64),
                            ..Default::default()
                        };
                        self.jlog("Halt", d, true);
                    }
                    self.machines[m].inbox.queue.clear();
                    self.machines[m].stage = Stage::Idle;
                    return Ok(SchedKind::Stop);
                }
                Stage::Idle => {
                    // Restarted by an enqueue: continue the event handler loop.
                    self.machines[m].stage = Stage::Loop;
                }
            }
        }
    }

    /// The payload passed to a handler with a parameter: a pending goto payload wins
    /// over the event's (`gotoPayload ?? e.Payload`, consumed only by parameterized
    /// actions, as in the generated code).
    fn take_payload(&mut self, m: usize, e: &Event) -> Value {
        match self.machines[m].goto_payload.take() {
            Some(p) if !p.is_null() => p,
            _ => e.payload.clone(),
        }
    }

    fn start_action(&mut self, m: usize, f: FunId, e: &Event) {
        self.cov_execute_action(m);
        let arg = if self.prog.fun(f).params.is_empty() { Value::Null } else { self.take_payload(m, e) };
        let exec = self.new_exec(f, vec![arg]);
        self.machines[m].exec = Some(exec);
    }

    pub fn new_exec(&self, f: FunId, args: Vec<Value>) -> Exec {
        let mut exec = Exec { frames: Vec::new(), pending: None, action: self.fun_name(f).to_string(), iters: Vec::new() };
        if let Some(code) = self.code.get(f) {
            exec.frames.push(make_frame(code.clone(), f, args));
        }
        exec
    }

    fn enter_state(&mut self, m: usize, e: Event) -> Result<(), Stop> {
        let name = self.machines[m].name.clone();
        let state = self.log_state_name(m);
        if self.logging { self.log_line(format!("<StateLog> {} enters state '{}'.", name, state)); } else { self.step_work += LOG_LINE_WORK; }
        self.json_state_transition(m, true);
        if let Some(s) = self.machines[m].state {
            if let Some(entry) = self.prog.state(s).entry {
                self.start_action(m, entry, &e);
            }
        }
        Ok(())
    }

    fn json_state_transition(&mut self, m: usize, is_entry: bool) {
        if !self.json_enabled() {
            return;
        }
        let name = self.machines[m].name.to_string();
        let st = self.short_state(self.machines[m].state);
        let d = Details {
            log: Some(format!("{} {} state '{}'.", name, if is_entry { "enters" } else { "exits" }, st)),
            id: Some(name),
            state: Some(st),
            is_entry: Some(is_entry),
            ..Default::default()
        };
        self.jlog("StateTransition", d, true);
    }

    fn exit_state(&mut self, m: usize, e: Event) -> Result<(), Stop> {
        let name = self.machines[m].name.clone();
        let state = self.log_state_name(m);
        if self.logging { self.log_line(format!("<StateLog> {} exits state '{}'.", name, state)); } else { self.step_work += LOG_LINE_WORK; }
        self.json_state_transition(m, false);
        if let Some(s) = self.machines[m].state {
            if let Some(exit) = self.prog.state(s).exit {
                self.start_action(m, exit, &e);
            }
        }
        Ok(())
    }

    fn begin_goto(&mut self, m: usize, target: StateId, with: Option<FunId>, e: Event) -> Result<(), Stop> {
        self.cov_goto(m, target);
        let name = self.machines[m].name.clone();
        let cur = self.log_state_name(m);
        let mname = self.prog.machine(self.machines[m].decl).name.clone();
        let tname = self.prog.state(target).name.clone();
        if self.logging { self.log_line(format!(
            "<GotoLog> {} is transitioning from state '{}' to state 'PImplementation.{}.{}'.",
            name, cur, mname, tname
        )); } else { self.step_work += LOG_LINE_WORK; }
        if self.json_enabled() {
            let short_cur = self.short_state(self.machines[m].state);
            let d = Details {
                log: Some(format!("{} is transitioning from state '{}' to state '{}'.", name, short_cur, tname)),
                id: Some(name.to_string()),
                start_state: Some(short_cur),
                end_state: Some(tname.clone()),
                ..Default::default()
            };
            self.jlog("GotoState", d, true);
        }
        self.machines[m].stage = Stage::AfterExit { e: e.clone(), then: ExitThen::Goto { target, with } };
        self.exit_state(m, e)
    }

    /// `ApplyEventHandlerTransitionAsync` for the pending transition.
    fn apply_transition(&mut self, m: usize) -> Result<(), Stop> {
        let t = std::mem::replace(&mut self.machines[m].pending_transition, Transition::None);
        match t {
            Transition::None => {}
            Transition::Raise(e) => {
                self.cov_raise(m, e.id);
                let name = self.machines[m].name.clone();
                let state = self.log_state_name(m);
                let ev = if self.logging || self.json_enabled() { self.event_with_payload(&e) } else { String::new() };
                if self.logging { self.log_line(format!("<RaiseLog> '{}' raised event '{}' in state '{}'.", name, ev, state)); } else { self.step_work += LOG_LINE_WORK; }
                if self.json_enabled() {
                    let (payload, key) = self.jpayload(&e);
                    let st = self.short_state(self.machines[m].state);
                    let d = Details {
                        log: Some(format!("'{}' raised event '{}' in state '{}'.", name, ev, st)),
                        id: Some(name.to_string()),
                        event: Some(self.event_name(e.id).to_string()),
                        state: Some(st),
                        payload,
                        payload_key: key,
                        ..Default::default()
                    };
                    self.jlog("RaiseEvent", d, true);
                }
                self.machines[m].inbox.raised = Some(e);
            }
            Transition::Goto(target) => {
                self.cov_raise(m, GOTO_EVENT);
                // Modeled as a raised goto event, handled on the next dequeue.
                self.machines[m].inbox.raised = Some(Event { id: GOTO_EVENT, payload: Value::Null, stamp: None });
                self.machines[m].goto_target = Some(target);
            }
            Transition::Halt => {
                self.machines[m].status = MachineStatus::Halting;
            }
        }
        Ok(())
    }

    // ---- monitors ----------------------------------------------------------------------

    fn create_monitors(&mut self) -> Result<(), Stop> {
        let decls: Vec<MachineId> = self.monitor_map.iter().map(|(m, _)| *m).collect();
        for decl in decls {
            if self.monitors.iter().any(|m| m.decl == decl) {
                continue;
            }
            self.cov_declare(decl, true);
            if self.prog.machine(decl).is_scenario() {
                self.scenarios.push(ScenarioProgress {
                    decl,
                    name: self.prog.machine(decl).name.clone(),
                    total_states: self.prog.machine(decl).states.len(),
                    reached: std::collections::BTreeSet::new(),
                    saw_start_entry: false,
                    satisfied: false,
                });
            }
            let fields = self
                .prog
                .machine(decl)
                .fields
                .iter()
                .map(|v| default_value(&self.prog.var(*v).ty, self.prog))
                .collect();
            let idx = self.monitors.len();
            self.monitors.push(Monitor {
                decl,
                fields,
                state: self.prog.machine(decl).start_state,
                temperature: 0,
                pending_transition: Transition::None,
                goto_payload: None,
            });
            if self.logging { self.log_line(format!("<CreateLog> {} was created.", self.monitor_full_name(idx))); } else { self.step_work += LOG_LINE_WORK; }
            if self.json_enabled() {
                let short = self.prog.machine(decl).name.clone();
                let d = Details { log: Some(format!("{} was created.", short)), monitor: Some(short), ..Default::default() };
                self.jlog("CreateMonitor", d, true);
            }
            let e = Event { id: self.prog.null_event, payload: Value::Null, stamp: None };
            self.monitor_enter_state(idx, &e)?;
        }
        Ok(())
    }

    fn monitor_state_log(&mut self, mon: usize, enters: bool) {
        if enters {
            self.cov_monitor_enter(mon);
        }
        let s = self.monitors[mon].state;
        let liveness = match s.map(|s| self.prog.state(s).temperature) {
            Some(Temperature::Hot) => "hot ",
            Some(Temperature::Cold) => "cold ",
            _ => "",
        };
        if self.logging {
            let text = format!(
            "<MonitorLog> {} {} {}state '{}'.",
            self.monitor_full_name(mon),
            if enters { "enters" } else { "exits" },
            liveness,
            self.state_name(s)
        );
            self.log_line(text);
        } else {
            self.step_work += LOG_LINE_WORK;
        }
        let decl = self.monitors[mon].decl;
        if enters {
            if let (Some(st), Some(sc)) = (s, self.scenarios.iter_mut().find(|p| p.decl == decl)) {
                // ScenarioComplianceObserver.OnMonitorStateTransition
                sc.reached.insert(st);
                let is_start_entry = !sc.saw_start_entry;
                sc.saw_start_entry = true;
                if self.prog.state(st).temperature == Temperature::Cold && !is_start_entry {
                    sc.satisfied = true;
                }
            }
        }
        if self.json_enabled() {
            let short = self.prog.machine(self.monitors[mon].decl).name.clone();
            let st = self.short_state(s);
            let hot = match s.map(|s| self.prog.state(s).temperature) {
                Some(Temperature::Hot) => Some(true),
                Some(Temperature::Cold) => Some(false),
                _ => None,
            };
            let d = Details {
                log: Some(format!("{} {} {}state '{}'.", short, if enters { "enters" } else { "exits" }, liveness, st)),
                state: Some(st),
                is_entry: Some(enters),
                is_in_hot_state: hot,
                monitor: Some(short),
                ..Default::default()
            };
            self.jlog("MonitorStateTransition", d, true);
        }
    }

    fn monitor_run_action(&mut self, mon: usize, f: FunId, e: &Event) -> Result<(), Stop> {
        let arg = if self.prog.fun(f).params.is_empty() {
            Value::Null
        } else {
            match self.monitors[mon].goto_payload.take() {
                Some(p) if !p.is_null() => p,
                _ => e.payload.clone(),
            }
        };
        let exec = self.new_exec(f, vec![arg]);
        match self.run_exec(Actor::Monitor(mon), exec)? {
            ExecOutcome::Finished(_) | ExecOutcome::Unwound => Ok(()),
            ExecOutcome::Yield(_, _) => {
                // Monitors cannot block; treat as completed.
                Ok(())
            }
        }
    }

    fn monitor_enter_state(&mut self, mon: usize, e: &Event) -> Result<(), Stop> {
        self.monitor_state_log(mon, true);
        if let Some(s) = self.monitors[mon].state {
            if let Some(entry) = self.prog.state(s).entry {
                self.monitor_run_action(mon, entry, e)?;
                self.monitor_apply_transition(mon)?;
            }
        }
        Ok(())
    }

    fn monitor_exit_state(&mut self, mon: usize, with: Option<FunId>, e: &Event) -> Result<(), Stop> {
        self.monitor_state_log(mon, false);
        if let Some(s) = self.monitors[mon].state {
            if let Some(exit) = self.prog.state(s).exit {
                self.monitor_run_action(mon, exit, e)?;
                self.monitor_check_exit_transition(mon)?;
            }
        }
        if let Some(w) = with {
            self.monitor_run_action(mon, w, e)?;
            self.monitor_check_exit_transition(mon)?;
        }
        Ok(())
    }

    fn monitor_check_exit_transition(&mut self, mon: usize) -> Result<(), Stop> {
        let t = std::mem::replace(&mut self.monitors[mon].pending_transition, Transition::None);
        if !matches!(t, Transition::None) {
            let msg = format!(
                "{} has performed a '{}' transition from an OnExit action.",
                self.monitor_full_name(mon),
                monitor_transition_name(&t)
            );
            return Err(self.report_bug(msg));
        }
        Ok(())
    }

    fn monitor_apply_transition(&mut self, mon: usize) -> Result<(), Stop> {
        let t = std::mem::replace(&mut self.monitors[mon].pending_transition, Transition::None);
        match t {
            Transition::None | Transition::Halt => Ok(()),
            Transition::Raise(e) => {
                self.cov_monitor_raise(mon, e.id);
                if self.logging {
            let text = format!(
                    "<MonitorLog> Monitor '{}' raised event '{}' in state '{}'.",
                    self.prog.machine(self.monitors[mon].decl).name,
                    self.event_with_payload(&e),
                    self.state_name(self.monitors[mon].state)
                );
            self.log_line(text);
        } else {
            self.step_work += LOG_LINE_WORK;
        }
                if self.json_enabled() {
                    let (payload, key) = self.jpayload(&e);
                    let short = self.prog.machine(self.monitors[mon].decl).name.clone();
                    let st = self.short_state(self.monitors[mon].state);
                    let d = Details {
                        log: Some(format!("Monitor '{}' raised event '{}' in state '{}'.", short, self.event_with_payload(&e), st)),
                        event: Some(self.event_name(e.id).to_string()),
                        state: Some(st),
                        payload,
                        monitor: Some(short),
                        payload_key: key,
                        ..Default::default()
                    };
                    self.jlog("MonitorRaiseEvent", d, true);
                }
                self.monitor_handle_event(mon, e)
            }
            Transition::Goto(target) => {
                self.cov_monitor_raise(mon, GOTO_EVENT);
                if self.logging {
            let text = format!(
                    "<MonitorLog> Monitor '{}' raised event 'GotoStateEvent' in state '{}'.",
                    self.prog.machine(self.monitors[mon].decl).name,
                    self.state_name(self.monitors[mon].state)
                );
            self.log_line(text);
        } else {
            self.step_work += LOG_LINE_WORK;
        }
                if self.json_enabled() {
                    let short = self.prog.machine(self.monitors[mon].decl).name.clone();
                    let st = self.short_state(self.monitors[mon].state);
                    let d = Details {
                        log: Some(format!("Monitor '{}' raised event 'GotoStateEvent' in state '{}'.", short, st)),
                        event: Some("GotoStateEvent".into()),
                        state: Some(st),
                        monitor: Some(short),
                        ..Default::default()
                    };
                    self.jlog("MonitorRaiseEvent", d, true);
                }
                let e = Event { id: self.prog.null_event, payload: Value::Null, stamp: None };
                self.monitor_goto(mon, target, None, &e)
            }
        }
    }

    fn monitor_goto(&mut self, mon: usize, target: StateId, with: Option<FunId>, e: &Event) -> Result<(), Stop> {
        self.monitor_exit_state(mon, with, e)?;
        self.monitors[mon].state = Some(target);
        if self.prog.state(target).temperature == Temperature::Cold {
            self.monitors[mon].temperature = 0;
        }
        self.monitor_enter_state(mon, e)
    }

    /// `Monitor.MonitorEvent`.
    pub fn monitor_event(&mut self, mon: usize, e: Event, sender_idx: Option<usize>) -> Result<(), Stop> {
        let sender: Option<Rc<str>> = sender_idx.map(|i| self.machines[i].name.clone());
        self.cov_monitor_process(mon, e.id, sender_idx);
        if self.logging {
            let text = format!(
            "<MonitorLog> {} is processing event '{}' in state '{}'.",
            self.prog.machine(self.monitors[mon].decl).name,
            self.event_with_payload(&e),
            self.state_name(self.monitors[mon].state)
        );
            self.log_line(text);
        } else {
            self.step_work += LOG_LINE_WORK;
        }
        if self.json_enabled() {
            let (payload, key) = self.jpayload(&e);
            let short = self.prog.machine(self.monitors[mon].decl).name.clone();
            let d = Details {
                log: Some(format!(
                    "{} is processing event '{}' in state '{}'.",
                    short,
                    self.event_with_payload(&e),
                    self.state_name(self.monitors[mon].state)
                )),
                state: Some(self.state_name(self.monitors[mon].state)),
                event: Some(self.event_name(e.id).to_string()),
                payload,
                sender: sender.map(|s| s.to_string()),
                monitor: Some(short),
                payload_key: key,
                ..Default::default()
            };
            self.jlog("MonitorProcessEvent", d, true);
        }
        let prev = self.time.as_mut().map(|t| std::mem::replace(&mut t.observer, sender_idx));
        let r = self.monitor_handle_event(mon, e);
        if let (Some(t), Some(p)) = (self.time.as_mut(), prev) {
            t.observer = p;
        }
        r
    }

    // ---- time ------------------------------------------------------------------------

    /// Stamp of an event `sender` sends to machine `target` now.
    fn send_stamp(&mut self, sender: usize, target: usize, event: EventId) -> crate::time::Stamp {
        let clock = self.machines[sender].clock.clone();
        let sender_id = self.machines[sender].id;
        let ts = self.time.as_mut().expect("time tracking");
        let range = ts.opts.profile.as_ref().and_then(|p| {
            p.latency(
                &self.prog.machine(self.machines[sender].decl).name,
                &self.prog.machine(self.machines[target].decl).name,
                &self.prog.event(event).name,
                sender == target,
            )
        });
        let at = match range {
            Some(r) => ts.advance(&clock, r),
            None => clock,
        };
        crate::time::Stamp { at, sender: sender_id }
    }

    /// Machine `m` takes event `e`: its clock catches up with the event's stamp.
    pub fn time_receive(&mut self, m: usize, e: &Event) {
        let (Some(ts), Some(stamp)) = (self.time.as_mut(), e.stamp.as_ref()) else { return };
        let clock = ts.join(&self.machines[m].clock, &stamp.at);
        self.machines[m].clock = clock;
        if self.logging {
            let line = format!("<TimeLog> '{}' clock is {} us.", self.machines[m].name, self.machines[m].clock);
            self.log_line(line);
        }
    }

    /// When operation `op` would act if scheduled now (`--timed`).
    fn ready_time(&mut self, op: usize) -> crate::time::Time {
        let Some(m) = self.ops[op].machine else { return crate::time::Time::default() };
        let clock = self.machines[m].clock.clone();
        let mach = &self.machines[m];
        let next = match &mach.stage {
            Stage::Idle | Stage::Loop | Stage::DequeueTry { .. } if mach.inbox.raised.is_none() && mach.exec.is_none() => {
                // a default handler runs at the clock when nothing has certainly arrived
                if self.has_default_handler(m) {
                    None
                } else {
                    self.timed_pick(m, None).and_then(|i| self.machines[m].inbox.queue[i].stamp.clone())
                }
            }
            Stage::Dequeued(e) => e.stamp.clone(),
            _ => match mach.exec.as_ref().and_then(|x| x.pending.as_ref()) {
                Some(Pending::ReceiveWait { cases, .. }) => {
                    let wait = receive_wait_set(self.prog, cases);
                    self.timed_pick(m, Some(&wait)).and_then(|i| self.machines[m].inbox.queue[i].stamp.clone())
                }
                _ => None,
            },
        };
        match next {
            Some(s) => self.time.as_mut().unwrap().join(&clock, &s.at),
            None => clock,
        }
    }

    pub fn timed(&self) -> bool {
        self.time.as_ref().is_some_and(|t| t.opts.timed)
    }

    /// `--timed`: the queued event machine `m` takes next, among the events `wait`
    /// lists (a `receive`) or that the current state neither ignores nor defers. The
    /// first such event of each sender is a candidate (per-sender FIFO); the one with
    /// the earliest worst-case stamp wins, the earliest queued on ties.
    pub fn timed_pick(&self, m: usize, wait: Option<&[EventId]>) -> Option<usize> {
        let mut senders: Vec<u64> = Vec::new();
        let mut best: Option<(i64, usize)> = None;
        for (i, e) in self.machines[m].inbox.queue.iter().enumerate() {
            let eligible = match wait {
                Some(w) => w.contains(&e.id),
                None => !self.is_ignored(m, e.id) && !self.is_deferred(m, e.id),
            };
            if !eligible {
                continue;
            }
            if let Some(st) = &e.stamp {
                if senders.contains(&st.sender) {
                    continue;
                }
                senders.push(st.sender);
            }
            let w = e.stamp.as_ref().map_or(i64::MIN, |st| st.at.worst());
            if best.is_none_or(|(bw, _)| w < bw) {
                best = Some((w, i));
            }
        }
        best.map(|(_, i)| i)
    }

    /// `--timed` dequeue: removes the picked event, dropping ignored events queued
    /// ahead of it as `try_dequeue` does. With a default handler, an event that
    /// certainly arrives after the machine's clock counts as not there yet.
    fn timed_take(&mut self, m: usize, has_default: bool) -> Option<Event> {
        let mut i = self.timed_pick(m, None)?;
        if has_default {
            let e = &self.machines[m].inbox.queue[i];
            if e.stamp.as_ref().is_some_and(|st| st.at.best() > self.machines[m].clock.worst()) {
                return None;
            }
        }
        let mut j = 0;
        while j < i {
            let id = self.machines[m].inbox.queue[j].id;
            if self.is_ignored(m, id) {
                self.machines[m].inbox.queue.remove(j);
                i -= 1;
            } else {
                j += 1;
            }
        }
        self.machines[m].inbox.queue.remove(i)
    }

    /// `--timed`: disables every enabled operation that certainly acts later than
    /// another enabled one. The earliest one always stays enabled.
    fn filter_timed(&mut self, infos: &mut [OpInfo]) {
        let mut ready: Vec<Option<(i64, i64)>> = Vec::with_capacity(infos.len());
        for (i, info) in infos.iter().enumerate() {
            ready.push(info.enabled.then(|| {
                let t = self.ready_time(i);
                (t.best(), t.worst())
            }));
        }
        // The two smallest worst cases, so each operation can be compared with the others.
        let (mut w1, mut w2) = ((i64::MAX, usize::MAX), i64::MAX);
        for (i, r) in ready.iter().enumerate() {
            if let Some((_, w)) = *r {
                if w < w1.0 {
                    w2 = w1.0;
                    w1 = (w, i);
                } else if w < w2 {
                    w2 = w;
                }
            }
        }
        for (i, r) in ready.iter().enumerate() {
            if let Some((b, _)) = *r {
                let other_worst = if i == w1.1 { w2 } else { w1.0 };
                if other_worst < b {
                    infos[i].enabled = false;
                }
            }
        }
    }

    /// The clock an actor reads: its own, or for a monitor the observed machine's.
    pub fn actor_clock(&self, actor: Actor) -> crate::time::Time {
        let m = match actor {
            Actor::Machine(m) => Some(m),
            Actor::Monitor(_) => self.time.as_ref().and_then(|t| t.observer),
        };
        m.map(|m| self.machines[m].clock.clone()).unwrap_or_default()
    }

    /// The natively implemented time functions; `None` if `name` is not one of them.
    pub fn call_time_function(&mut self, actor: Actor, name: &str, args: &[Value]) -> Option<Result<Value, Exception>> {
        self.time.as_ref()?;
        crate::time::time_signature(name)?;
        let int = |i: usize| match args.get(i) {
            Some(Value::Int(v)) => Ok(*v),
            _ => Err(Exception::new("System.ArgumentException", format!("{} expects int arguments", name))),
        };
        let clock = self.actor_clock(actor);
        let elapse = |rt: &mut Self, r: crate::time::Range| -> Result<Value, Exception> {
            let Actor::Machine(m) = actor else {
                return Err(Exception::new("System.InvalidOperationException", format!("{} cannot be called from a monitor", name)));
            };
            let c = rt.time.as_mut().unwrap().advance(&rt.machines[m].clock, r);
            rt.machines[m].clock = c;
            Ok(Value::Null)
        };
        let mark = |rt: &Self, h: i64| -> Result<crate::time::Time, Exception> {
            let ts = rt.time.as_ref().unwrap();
            usize::try_from(h).ok().and_then(|h| ts.marks.get(h)).cloned().ok_or_else(|| {
                Exception::new("System.ArgumentException", format!("{}: {} is not a handle returned by Mark()", name, h))
            })
        };
        Some((|| match name {
            "Now" => Ok(Value::Int(clock.worst())),
            "NowBest" => Ok(Value::Int(clock.best())),
            "Elapse" => {
                let (lo, hi) = (int(0)?, int(1)?);
                if lo < 0 || lo > hi || hi > crate::time::MAX_RANGE {
                    return Err(Exception::new(
                        "System.ArgumentException",
                        format!("Elapse({}, {}): expected 0 <= lo <= hi <= {}", lo, hi, crate::time::MAX_RANGE),
                    ));
                }
                elapse(self, crate::time::Range { lo, hi })
            }
            "ElapseCost" => {
                let Some(Value::Str(n)) = args.first() else {
                    return Err(Exception::new("System.ArgumentException", "ElapseCost expects a string"));
                };
                // without a profile everything costs 0; with one, an unpriced name is a typo
                let r = match &self.time.as_ref().unwrap().opts.profile {
                    None => crate::time::Range { lo: 0, hi: 0 },
                    Some(p) => p.cost(n).ok_or_else(|| {
                        Exception::new(
                            "System.ArgumentException",
                            format!("ElapseCost(\"{}\"): latency profile '{}' has no entry \"cost.{}\"", n, p.name, n),
                        )
                    })?,
                };
                elapse(self, r)
            }
            "Mark" => {
                let ts = self.time.as_mut().unwrap();
                ts.marks.push(clock);
                Ok(Value::Int(ts.marks.len() as i64 - 1))
            }
            "Elapsed" => Ok(Value::Int(clock.sub(&mark(self, int(0)?)?).worst())),
            "ElapsedBest" => Ok(Value::Int(clock.sub(&mark(self, int(0)?)?).best())),
            _ => unreachable!(),
        })())
    }

    fn monitor_handle_event(&mut self, mon: usize, e: Event) -> Result<(), Stop> {
        // Ignored (exact or wildcard) events are dropped.
        if let Some(s) = self.monitors[mon].state {
            if matches!(self.prog.state(s).handler_for(e.id).map(|h| &h.action), Some(HandlerAction::Ignore)) {
                return Ok(());
            }
        }
        loop {
            let Some(s) = self.monitors[mon].state else {
                let msg = format!(
                    "{} received event '{}' that cannot be handled.",
                    self.monitor_full_name(mon),
                    event_cs_full_name(self.prog, e.id)
                );
                return Err(self.report_bug(msg));
            };
            let action = self.prog.state(s).handler_for(e.id).map(|h| h.action.clone());
            match action {
                Some(HandlerAction::Do(f)) => {
                    self.cov_monitor_do(mon);
                    self.monitor_run_action(mon, f, &e)?;
                    return self.monitor_apply_transition(mon);
                }
                Some(HandlerAction::Goto { target, with }) => {
                    return self.monitor_goto(mon, target, with, &e);
                }
                Some(HandlerAction::Ignore) | Some(HandlerAction::Defer) => return Ok(()),
                None => {
                    self.monitor_state_log(mon, false);
                    self.monitors[mon].state = None;
                }
            }
        }
    }
}

fn monitor_transition_name(t: &Transition) -> &'static str {
    match t {
        Transition::None => "None",
        Transition::Raise(_) => "Raise",
        Transition::Goto(_) => "Goto",
        Transition::Halt => "Halt",
    }
}

pub fn make_frame(code: Rc<CodeBlock>, fun: FunId, args: Vec<Value>) -> Frame {
    let mut locals = vec![Value::Null; code.frame_size as usize];
    for (slot, arg) in code.param_slots.iter().zip(args) {
        locals[*slot as usize] = arg;
    }
    Frame { code, fun, pc: 0, locals, stack: Vec::new() }
}

/// Outcome of running an action.
pub enum ExecOutcome {
    Finished(Value),
    Yield(Exec, SchedKind),
    /// A raise or goto unwound the action.
    Unwound,
}

/// Pseudo event id used for the internal `GotoStateEvent`.
pub const GOTO_EVENT: EventId = EventId(u32::MAX);

/// Helper to query the role of a variable id (used by the interpreter).
pub fn is_field(prog: &Program, v: pnext_sema::VarId) -> bool {
    prog.var(v).role == VarRole::Field
}

pub fn is_spec(prog: &Program, m: MachineId) -> bool {
    prog.machine(m).kind != MachineKind::Impl
}

/// Progress of a scenario (coverage) monitor within one schedule.
#[derive(Debug, Clone)]
pub struct ScenarioProgress {
    pub decl: MachineId,
    pub name: String,
    pub total_states: usize,
    /// Distinct states entered.
    pub reached: std::collections::BTreeSet<StateId>,
    pub saw_start_entry: bool,
    /// Entered a cold (accepting) state other than its start state.
    pub satisfied: bool,
}

/// `MonitorMapWithScenarios`: the module's monitors plus every scenario monitor observing
/// all interfaces of the module (only when the module defines interfaces).
pub fn monitor_map_with_scenarios(prog: &Program, module: &ModuleInfo) -> Vec<(MachineId, Vec<InterfaceId>)> {
    let mut map = module.monitor_map.clone();
    if !module.interface_def.is_empty() {
        let all: Vec<InterfaceId> = module.interface_def.iter().map(|(i, _)| *i).collect();
        for (i, m) in prog.machines.iter().enumerate() {
            let id = MachineId(i as u32);
            if m.is_scenario() && !map.iter().any(|(k, _)| *k == id) {
                map.push((id, all.clone()));
            }
        }
    }
    map
}

/// The events a `receive` with these cases waits for (`halt` always included).
pub fn receive_wait_set(prog: &Program, cases: &[ReceiveCase]) -> Vec<EventId> {
    let mut wait: Vec<EventId> = Vec::new();
    for c in cases {
        if !wait.contains(&c.event) {
            wait.push(c.event);
        }
    }
    if !wait.contains(&prog.halt_event) {
        wait.push(prog.halt_event);
    }
    wait
}
