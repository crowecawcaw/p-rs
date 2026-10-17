//! Activity coverage (`ReportActivityCoverage`): which events each state received and
//! sent, which handled events were never exercised, and the state transitions observed —
//! the `<file>.coverage.txt` report of upstream's `ActivityCoverageReporter`.
//!
//! Upstream assembles this from two runtime loggers, `ControlledRuntimeLogEventCoverage`
//! and `ControlledRuntimeLogGraphBuilder` (with machine instances collapsed to their type).
//! The hooks below are called at the same points as those loggers' callbacks, and keep the
//! same quirks (one global "dequeued" event, monitor do-actions named with their
//! temperature, ...), with nodes and events interned for speed.

use std::collections::{HashMap, HashSet};

use pnext_sema::{EventId, HandlerAction, MachineId, Program, StateId, Temperature};

use crate::engine::culture_order;
use crate::runtime::{Runtime, GOTO_EVENT};
use crate::value::event_cs_name;

/// Node and event naming tables for a program (built once).
#[derive(Debug, Clone)]
pub struct CoverageTables {
    /// Node id strings (`PImplementation.Machine.State`, possibly with `[hot]`/`[cold]`).
    pub nodes: Vec<String>,
    /// Plain node of each state (indexed by `StateId`).
    plain: Vec<u32>,
    /// Node of each state named with its temperature (`CurrentStateNameWithTemperature`).
    tempered: Vec<u32>,
    /// Per machine, the `<Machine>.StateMachine` node that upstream's `OnReceiveEvent`
    /// links to (it passes the node category where the state name belongs).
    receive_target: Vec<u32>,
}

impl CoverageTables {
    pub fn new(prog: &Program) -> CoverageTables {
        let mut nodes = Vec::new();
        let mut index: HashMap<String, u32> = HashMap::new();
        let mut intern = |s: String| -> u32 {
            *index.entry(s.clone()).or_insert_with(|| {
                nodes.push(s);
                (nodes.len() - 1) as u32
            })
        };
        let mut plain = vec![0; prog.states.len()];
        let mut tempered = vec![0; prog.states.len()];
        let mut receive_target = Vec::with_capacity(prog.machines.len());
        for m in &prog.machines {
            for s in &m.states {
                let st = prog.state(*s);
                let base = format!("PImplementation.{}.{}", m.cs_name, st.name);
                plain[s.idx()] = intern(base.clone());
                let suffix = match st.temperature {
                    Temperature::Hot => "[hot]",
                    Temperature::Cold => "[cold]",
                    _ => "",
                };
                tempered[s.idx()] = intern(format!("{}{}", base, suffix));
            }
            receive_target.push(intern(format!("PImplementation.{}.StateMachine", m.cs_name)));
        }
        CoverageTables { nodes, plain, tempered, receive_target }
    }

    fn plain(&self, s: StateId) -> u32 {
        self.plain[s.idx()]
    }

    fn tempered(&self, s: StateId) -> u32 {
        self.tempered[s.idx()]
    }
}

/// Coverage accumulated over all schedules (`CoverageInfo`).
#[derive(Debug, Clone, Default)]
pub struct CoverageInfo {
    /// Declared machines and monitors, in declaration order, with whether each is a monitor.
    pub declared: Vec<(MachineId, bool)>,
    /// `(state node, event)` pairs received / sent.
    pub received: HashSet<(u32, u32)>,
    pub sent: HashSet<(u32, u32)>,
    /// Transition links `(source node, target node)` of the coverage graph.
    pub links: HashSet<(u32, u32)>,
}

/// Interned event key: `EventId` values, plus the goto pseudo event.
fn event_key(e: EventId) -> u32 {
    e.0
}

/// Who put an entry in a graph-builder inbox (`EventInfo.Name/Type`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sender {
    Machine(usize),
    Monitor(usize),
}

#[derive(Debug, Clone, Copy)]
struct InboxEntry {
    sender: Sender,
    /// Node of the sender's state (`EventInfo.State`).
    state: u32,
    event: u32,
}

/// Per-schedule state of the coverage loggers.
#[derive(Debug, Clone, Default)]
pub struct CoverageRun {
    /// `ControlledRuntimeLogEventCoverage.Dequeued` (a single, global slot upstream).
    dequeued: Option<u32>,
    /// Graph builder inboxes per machine/monitor type.
    inbox: HashMap<MachineId, Vec<InboxEntry>>,
    /// Graph builder `Dequeued`, per machine instance.
    graph_dequeued: HashMap<usize, InboxEntry>,
}

impl Runtime<'_> {
    fn cov_on(&self) -> bool {
        self.coverage.is_some()
    }

    fn mstate_node(&self, m: usize) -> Option<u32> {
        let tables = self.coverage_tables.as_ref()?;
        self.machines[m].state.map(|s| tables.plain(s))
    }

    fn cov_add_event(&mut self, target: MachineId, entry: InboxEntry) {
        if let Some(run) = self.coverage_run.as_mut() {
            run.inbox.entry(target).or_default().push(entry);
        }
    }

    /// `PopEvent`: removes every matching entry, returns the earliest.
    fn cov_pop_event(&mut self, target: MachineId, event: u32) -> Option<InboxEntry> {
        let run = self.coverage_run.as_mut()?;
        let inbox = run.inbox.get_mut(&target)?;
        let mut result = None;
        let mut i = inbox.len();
        while i > 0 {
            i -= 1;
            if inbox[i].event == event {
                result = Some(inbox.remove(i));
            }
        }
        result
    }

    fn cov_link(&mut self, a: u32, b: u32) {
        if let Some(c) = self.coverage.as_mut() {
            c.links.insert((a, b));
        }
    }

    /// `ReportActivityCoverageOfStateMachine/Monitor` + `OnCreateStateMachine/Monitor`.
    pub(crate) fn cov_declare(&mut self, decl: MachineId, monitor: bool) {
        if let Some(c) = self.coverage.as_mut() {
            if !c.declared.iter().any(|(d, _)| *d == decl) {
                c.declared.push((decl, monitor));
            }
        }
    }

    /// `OnSendEvent`.
    pub(crate) fn cov_send(&mut self, sender: usize, target_decl: MachineId, e: EventId) {
        if !self.cov_on() {
            return;
        }
        let Some(state) = self.mstate_node(sender) else { return };
        let ev = event_key(e);
        if let Some(c) = self.coverage.as_mut() {
            c.sent.insert((state, ev));
        }
        let entry = InboxEntry { sender: Sender::Machine(sender), state, event: ev };
        self.cov_add_event(target_decl, entry);
    }

    /// `OnRaiseEvent` (user raises and the goto pseudo event).
    pub(crate) fn cov_raise(&mut self, m: usize, e: EventId) {
        if !self.cov_on() {
            return;
        }
        let Some(state) = self.mstate_node(m) else { return };
        let ev = event_key(e);
        if let Some(c) = self.coverage.as_mut() {
            c.sent.insert((state, ev));
        }
        let decl = self.machines[m].decl;
        self.cov_add_event(decl, InboxEntry { sender: Sender::Machine(m), state, event: ev });
    }

    /// `OnDequeueEvent`.
    pub(crate) fn cov_dequeue(&mut self, m: usize, e: EventId) {
        if !self.cov_on() {
            return;
        }
        let ev = event_key(e);
        if let Some(run) = self.coverage_run.as_mut() {
            run.dequeued = Some(ev);
        }
        let decl = self.machines[m].decl;
        if let Some(info) = self.cov_pop_event(decl, ev) {
            if let Some(run) = self.coverage_run.as_mut() {
                run.graph_dequeued.insert(m, info);
            }
        }
    }

    /// `OnDefaultEventHandler`.
    pub(crate) fn cov_default(&mut self, m: usize) {
        if !self.cov_on() {
            return;
        }
        let ev = event_key(self.prog.null_event);
        if let Some(run) = self.coverage_run.as_mut() {
            run.dequeued = Some(ev);
        }
        let Some(state) = self.mstate_node(m) else { return };
        let decl = self.machines[m].decl;
        self.cov_add_event(decl, InboxEntry { sender: Sender::Machine(m), state, event: ev });
        let popped = self.cov_pop_event(decl, ev);
        if let Some(run) = self.coverage_run.as_mut() {
            match popped {
                Some(i) => {
                    run.graph_dequeued.insert(m, i);
                }
                None => {
                    run.graph_dequeued.remove(&m);
                }
            }
        }
    }

    /// `OnHandleRaisedEvent`.
    pub(crate) fn cov_handle_raised(&mut self, m: usize, e: EventId) {
        if !self.cov_on() {
            return;
        }
        let ev = event_key(e);
        let decl = self.machines[m].decl;
        if let Some(run) = self.coverage_run.as_mut() {
            run.dequeued = Some(ev);
            let found = run.inbox.get(&decl).and_then(|inbox| inbox.iter().rev().find(|i| i.event == ev).copied());
            if let Some(info) = found {
                run.graph_dequeued.insert(m, info);
            }
        }
    }

    /// `ControlledRuntimeLogEventCoverage.OnEventHandled`.
    fn cov_event_handled(&mut self, state: u32) {
        let Some(ev) = self.coverage_run.as_mut().and_then(|r| r.dequeued.take()) else { return };
        if let Some(c) = self.coverage.as_mut() {
            c.received.insert((state, ev));
        }
    }

    /// `LinkTransition`.
    fn cov_link_transition(&mut self, m: usize, current: u32, new_state: Option<u32>) {
        let info = self.coverage_run.as_mut().and_then(|r| r.graph_dequeued.remove(&m));
        if let Some(i) = info {
            if i.sender != Sender::Machine(m) || i.state != current {
                self.cov_link(i.state, current);
            }
        }
        if let Some(n) = new_state {
            self.cov_link(current, n);
        }
    }

    /// `OnExecuteAction` (do, entry, exit and `with` actions of machines).
    pub(crate) fn cov_execute_action(&mut self, m: usize) {
        if !self.cov_on() {
            return;
        }
        let Some(state) = self.mstate_node(m) else { return };
        self.cov_event_handled(state);
        self.cov_link_transition(m, state, None);
    }

    /// `OnGotoState`.
    pub(crate) fn cov_goto(&mut self, m: usize, target: StateId) {
        if !self.cov_on() {
            return;
        }
        let Some(state) = self.mstate_node(m) else { return };
        self.cov_event_handled(state);
        let t = self.coverage_tables.as_ref().map(|t| t.plain(target)).unwrap_or(0);
        self.cov_link_transition(m, state, Some(t));
    }

    /// `OnReceiveEvent` (a `receive` statement got its event).
    pub(crate) fn cov_receive(&mut self, m: usize, e: EventId) {
        if !self.cov_on() {
            return;
        }
        let Some(state) = self.mstate_node(m) else { return };
        let ev = event_key(e);
        if let Some(c) = self.coverage.as_mut() {
            c.received.insert((state, ev));
        }
        let decl = self.machines[m].decl;
        let found = self.coverage_run.as_mut().and_then(|run| {
            let inbox = run.inbox.get_mut(&decl)?;
            let i = inbox.iter().rposition(|i| i.event == ev)?;
            Some(inbox.remove(i))
        });
        if let Some(info) = found {
            let target = self.coverage_tables.as_ref().map(|t| t.receive_target[decl.idx()]).unwrap_or(state);
            self.cov_link(info.state, target);
        }
    }

    fn mon_state(&self, mon: usize) -> Option<StateId> {
        self.monitors[mon].state
    }

    /// `OnMonitorProcessEvent`.
    pub(crate) fn cov_monitor_process(&mut self, mon: usize, e: EventId, sender: Option<usize>) {
        if !self.cov_on() {
            return;
        }
        let Some(s) = self.mon_state(mon) else { return };
        let Some(tables) = self.coverage_tables.as_ref() else { return };
        let state = tables.plain(s);
        let ev = event_key(e);
        if let Some(c) = self.coverage.as_mut() {
            c.received.insert((state, ev));
        }
        let decl = self.monitors[mon].decl;
        self.cov_add_event(decl, InboxEntry { sender: Sender::Monitor(mon), state, event: ev });
        if let Some(src) = sender.and_then(|m| self.mstate_node(m)) {
            self.cov_link(src, state);
        }
    }

    /// `OnMonitorExecuteAction` (do actions; the state is named with its temperature).
    pub(crate) fn cov_monitor_do(&mut self, mon: usize) {
        if !self.cov_on() {
            return;
        }
        let Some(s) = self.mon_state(mon) else { return };
        let Some(target) = self.coverage_tables.as_ref().map(|t| t.tempered(s)) else { return };
        let decl = self.monitors[mon].decl;
        let popped = self.coverage_run.as_mut().and_then(|r| r.inbox.get_mut(&decl).and_then(|i| i.pop()));
        if let Some(e) = popped {
            self.cov_link(e.state, target);
        }
    }

    /// `OnMonitorRaiseEvent` (raises and gotos inside monitor actions).
    pub(crate) fn cov_monitor_raise(&mut self, mon: usize, e: EventId) {
        if !self.cov_on() {
            return;
        }
        let Some(s) = self.mon_state(mon) else { return };
        let Some(state) = self.coverage_tables.as_ref().map(|t| t.plain(s)) else { return };
        let ev = event_key(e);
        if let Some(c) = self.coverage.as_mut() {
            c.sent.insert((state, ev));
        }
        let decl = self.monitors[mon].decl;
        self.cov_add_event(decl, InboxEntry { sender: Sender::Monitor(mon), state, event: ev });
    }

    /// `OnMonitorStateTransition` on entry.
    pub(crate) fn cov_monitor_enter(&mut self, mon: usize) {
        if !self.cov_on() {
            return;
        }
        let Some(s) = self.mon_state(mon) else { return };
        let Some(target) = self.coverage_tables.as_ref().map(|t| t.tempered(s)) else { return };
        let decl = self.monitors[mon].decl;
        let popped = self.coverage_run.as_mut().and_then(|r| r.inbox.get_mut(&decl).and_then(|i| i.pop()));
        if let Some(e) = popped {
            self.cov_link(e.state, target);
        }
    }
}

/// Fully qualified event type name, as the coverage keys use it.
fn event_full_name(prog: &Program, ev: u32) -> String {
    if ev == GOTO_EVENT.0 {
        return "PChecker.Runtime.StateMachines.Events.GotoStateEvent".into();
    }
    let e = EventId(ev);
    if e == prog.null_event {
        return "PChecker.Runtime.Events.DefaultEvent".into();
    }
    format!("PImplementation.{}", event_cs_name(prog, e))
}

fn is_builtin(prog: &Program, ev: u32) -> bool {
    ev == GOTO_EVENT.0 || EventId(ev) == prog.null_event
}

/// After the last `.` (`GetSanitizedName`, `GetStateName`).
fn after_dot(s: &str) -> &str {
    match s.rfind('.') {
        Some(i) if i > 0 => &s[i + 1..],
        _ => s,
    }
}

/// Up to the last `.` (`GetMachineId`).
fn before_dot(s: &str) -> &str {
    match s.rfind('.') {
        Some(i) if i > 0 => &s[..i],
        _ => s,
    }
}

fn sorted_names(names: impl Iterator<Item = String>) -> Vec<String> {
    let mut v: Vec<String> = names.map(|n| after_dot(&n).to_string()).collect();
    v.sort_by(|a, b| culture_order(a, b));
    v
}

fn percent(covered: usize, total: usize) -> String {
    if total == 0 {
        "100.0".into()
    } else {
        format!("{:.1}", covered as f64 * 100.0 / total as f64)
    }
}

/// `ActivityCoverageReporter.WriteCoverageText`.
pub fn coverage_report(prog: &Program, tables: &CoverageTables, info: &CoverageInfo) -> String {
    let node = |n: u32| tables.nodes[n as usize].as_str();
    // registered (handled, not deferred/ignored) events per state node
    let mut registered: Vec<(MachineId, u32, Vec<u32>)> = Vec::new();
    for (decl, _) in &info.declared {
        for s in &prog.machine(*decl).states {
            let evs: Vec<u32> = prog
                .state(*s)
                .handlers
                .iter()
                .filter(|h| matches!(h.action, HandlerAction::Do(_) | HandlerAction::Goto { .. }))
                .map(|h| event_key(h.event))
                .collect();
            if !evs.is_empty() {
                registered.push((*decl, tables.plain(*s), evs));
            }
        }
    }
    let uncovered = |state: u32, evs: &[u32]| -> Vec<u32> {
        let mut v: Vec<u32> = evs.iter().copied().filter(|e| !info.received.contains(&(state, *e))).collect();
        v.dedup();
        v
    };
    let total: usize = registered.iter().map(|(_, _, e)| e.len()).sum();
    let total_uncovered: usize = registered.iter().map(|(_, s, e)| uncovered(*s, e).len()).sum();
    let mut out = String::new();
    let header = |out: &mut String, h: String| {
        out.push_str(&h);
        out.push('\n');
        out.push_str(&"=".repeat(h.chars().count()));
        out.push('\n');
    };
    header(&mut out, format!("Total event coverage: {}%", percent(total - total_uncovered, total)));
    let mut machines: Vec<(String, MachineId, bool)> = info
        .declared
        .iter()
        .map(|(d, mon)| (format!("PImplementation.{}", prog.machine(*d).cs_name), *d, *mon))
        .collect();
    machines.sort_by(|a, b| culture_order(&a.0, &b.0));
    for (full, decl, monitor) in machines {
        header(&mut out, format!("{}: {}", if monitor { "Monitor" } else { "StateMachine" }, after_dot(&full)));
        let mine: Vec<&(MachineId, u32, Vec<u32>)> = registered.iter().filter(|(d, _, _)| *d == decl).collect();
        let mt: usize = mine.iter().map(|(_, _, e)| e.len()).sum();
        let mu: usize = mine.iter().map(|(_, s, e)| uncovered(*s, e).len()).sum();
        out.push_str(&format!("Event coverage: {}%\n", percent(mt - mu, mt)));
        for s in &prog.machine(decl).states {
            let key = tables.plain(*s);
            let state_name = &prog.state(*s).name;
            let evs: Vec<u32> = mine.iter().filter(|(_, k, _)| *k == key).flat_map(|(_, _, e)| e.clone()).collect();
            let st = evs.len();
            let su = uncovered(key, &evs);
            out.push('\n');
            out.push_str(&format!(
                "\tState: {}{}\n",
                state_name,
                if st > 0 && st == su.len() { " is uncovered" } else { "" }
            ));
            if st == 0 {
                out.push_str("\t\tState has no expected events, so coverage is 100%\n");
            } else if st != su.len() {
                out.push_str(&format!("\t\tState event coverage: {}%\n", percent(st - su.len(), st)));
            }
            let key_name = node(key);
            let mut incoming: HashSet<String> = HashSet::new();
            let mut outgoing: HashSet<String> = HashSet::new();
            for (a, b) in &info.links {
                let (sa, sb) = (node(*a), node(*b));
                if before_dot(sa) == full && before_dot(sb) == full {
                    if sb == key_name {
                        incoming.insert(after_dot(sa).to_string());
                    }
                    if sa == key_name {
                        outgoing.insert(after_dot(sb).to_string());
                    }
                }
            }
            let received: HashSet<String> = info
                .received
                .iter()
                .filter(|(n, e)| *n == key && !is_builtin(prog, *e))
                .map(|(_, e)| event_full_name(prog, *e))
                .collect();
            if !received.is_empty() {
                out.push_str(&format!("\t\tEvents received: {}\n", sorted_names(received.into_iter()).join(", ")));
            }
            let sent: HashSet<String> = info
                .sent
                .iter()
                .filter(|(n, e)| *n == key && !is_builtin(prog, *e))
                .map(|(_, e)| event_full_name(prog, *e))
                .collect();
            if !sent.is_empty() {
                out.push_str(&format!("\t\tEvents sent: {}\n", sorted_names(sent.into_iter()).join(", ")));
            }
            if !su.is_empty() {
                let names = su.iter().map(|e| event_full_name(prog, *e)).collect::<HashSet<_>>();
                out.push_str(&format!("\t\tEvents not covered: {}\n", sorted_names(names.into_iter()).join(", ")));
            }
            if !incoming.is_empty() {
                out.push_str(&format!("\t\tPrevious states: {}\n", sorted_names(incoming.into_iter()).join(", ")));
            }
            if !outgoing.is_empty() {
                out.push_str(&format!("\t\tNext states: {}\n", sorted_names(outgoing.into_iter()).join(", ")));
            }
        }
        out.push('\n');
    }
    out
}
