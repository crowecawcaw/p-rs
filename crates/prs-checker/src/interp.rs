//! The bytecode interpreter: executes one action of a machine or monitor.
//!
//! Actions run to completion unless they reach a scheduling point (`new`, `send`,
//! `receive`), in which case the whole execution state is handed back to the runtime
//! and resumed later. Errors that the upstream runtime reports as .NET exceptions are
//! modeled as [`Exception`] values and turned into bug reports with the same text.

use std::rc::Rc;

use prs_sema::{BinOp, EventId, PType, Program};

use crate::bytecode::{Op, PathElem, ReceiveCase, Root};
use crate::runtime::{
    make_frame, ActiveIter, Actor, Event, Exception, Exec, ExecOutcome, MachineStatus, OpStatus, Pending, Runtime,
    SchedKind, Stop, Transition,
};
use crate::value::{default_value, event_cs_full_name, format_composite, format_exception, NamedTuple, Value};

enum Fault {
    Stop,
    Exc(Exception),
}

impl From<Stop> for Fault {
    fn from(_: Stop) -> Fault {
        Fault::Stop
    }
}

impl From<Exception> for Fault {
    fn from(e: Exception) -> Fault {
        Fault::Exc(e)
    }
}

enum Flow {
    Continue,
    Finished(Value),
    Yield(SchedKind),
    Unwound,
}

// ---- .NET exceptions ----------------------------------------------------------

fn nre() -> Exception {
    Exception::new("System.NullReferenceException", "Object reference not set to an instance of an object.")
}

fn index_oor() -> Exception {
    Exception::new(
        "System.ArgumentOutOfRangeException",
        "Index was out of range. Must be non-negative and less than the size of the collection. (Parameter 'index')",
    )
}

fn insert_oor() -> Exception {
    Exception::new(
        "System.ArgumentOutOfRangeException",
        "Index must be within the bounds of the List. (Parameter 'index')",
    )
}

fn element_at_oor() -> Exception {
    Exception::new(
        "System.ArgumentOutOfRangeException",
        "Specified argument was out of the range of valid values. (Parameter 'index')",
    )
}

fn key_not_found(key: &str) -> Exception {
    Exception::new(
        "System.Collections.Generic.KeyNotFoundException",
        format!("The given key '{}' was not present in the dictionary.", key),
    )
}

fn duplicate_key(key: &str) -> Exception {
    Exception::new(
        "System.ArgumentException",
        format!("An item with the same key has already been added. Key: {}", key),
    )
}

fn div_zero() -> Exception {
    Exception::new("System.DivideByZeroException", "Attempted to divide by zero.")
}

/// `long.MinValue / -1` (and `% -1`), which .NET does not wrap.
fn overflow() -> Exception {
    Exception::new("System.OverflowException", "Arithmetic operation resulted in an overflow.")
}

fn modified() -> Exception {
    Exception::new(
        "System.InvalidOperationException",
        "Collection was modified; enumeration operation may not execute.",
    )
    .from_source(crate::runtime::CORE_LIB)
}

/// The C# class name of a value, for `InvalidCastException` messages.
fn value_class(v: &Value, prog: &Program) -> String {
    match v {
        Value::Event(e) => event_cs_full_name(prog, *e),
        other => format!("PChecker.Runtime.Values.{}", other.type_name()),
    }
}

/// The C# class name of a P type.
fn type_class(ty: &PType, prog: &Program) -> String {
    let name = match ty.canonicalize(prog) {
        PType::Bool => "PChecker.Runtime.Values.PBool",
        PType::Int | PType::Enum(_) => "PChecker.Runtime.Values.PInt",
        PType::Float => "PChecker.Runtime.Values.PFloat",
        PType::String => "PChecker.Runtime.Values.PString",
        PType::Event => "PChecker.Runtime.Events.Event",
        PType::Machine | PType::Permission(_) => "PChecker.Runtime.Values.PMachineValue",
        PType::Seq(_) => "PChecker.Runtime.Values.PSeq",
        PType::Set(_) => "PChecker.Runtime.Values.PSet",
        PType::Map(..) => "PChecker.Runtime.Values.PMap",
        PType::Tuple(_) => "PChecker.Runtime.Values.PTuple",
        PType::NamedTuple(_) => "PChecker.Runtime.Values.PNamedTuple",
        _ => "PChecker.Runtime.Values.IPValue",
    };
    name.to_string()
}

fn invalid_cast(v: &Value, to: &str, prog: &Program) -> Exception {
    Exception::new(
        "System.InvalidCastException",
        format!("Unable to cast object of type '{}' to type '{}'.", value_class(v, prog), to),
    )
}

/// Checks a C# cast `(T)value`.
fn check_cast(v: &Value, ty: &PType, prog: &Program) -> Result<(), Exception> {
    let canon = ty.canonicalize(prog);
    let ok = match (&canon, v) {
        (PType::Any | PType::Data | PType::Null | PType::Foreign(_) | PType::Error | PType::TypeDef(_), _) => true,
        (PType::Bool | PType::Int | PType::Float | PType::Enum(_), Value::Null) => {
            return Err(nre().from_source(crate::runtime::CORE_LIB))
        }
        (_, Value::Null) => true,
        (PType::Bool, Value::Bool(_)) => true,
        (PType::Int | PType::Enum(_), Value::Int(_)) => true,
        (PType::Float, Value::Float(_)) => true,
        (PType::String, Value::Str(_)) => true,
        (PType::Event, Value::Event(_)) => true,
        (PType::Machine | PType::Permission(_), Value::Machine(_)) => true,
        (PType::Seq(_), Value::Seq(_)) => true,
        (PType::Set(_), Value::Set(_)) => true,
        (PType::Map(..), Value::Map(_)) => true,
        (PType::Tuple(_), Value::Tuple(_)) => true,
        (PType::NamedTuple(_), Value::NamedTuple(_)) => true,
        _ => false,
    };
    if ok {
        Ok(())
    } else {
        Err(invalid_cast(v, &type_class(&canon, prog), prog))
    }
}

// ---- l-value navigation ----------------------------------------------------------

fn key_count(path: &[PathElem]) -> usize {
    path.iter().filter(|p| matches!(p, PathElem::SeqIndex | PathElem::MapKey)).count()
}

/// Walks an l-value path with getter semantics (`((PSeq)x)[i]`, `((PMap)m)[k]`, ...),
/// copying shared containers on write.
fn navigate<'a>(
    v: &'a mut Value,
    path: &[PathElem],
    keys: &[Value],
    prog: &Program,
) -> Result<&'a mut Value, Exception> {
    let mut cur = v;
    let mut ki = 0;
    for p in path {
        let next: &'a mut Value = match p {
            PathElem::TupleField(i) => match cur {
                Value::Tuple(t) => Rc::make_mut(t).get_mut(*i as usize).ok_or_else(index_oor)?,
                Value::NamedTuple(t) => Rc::make_mut(t).values.get_mut(*i as usize).ok_or_else(index_oor)?,
                Value::Null => return Err(nre()),
                other => return Err(invalid_cast(other, "PChecker.Runtime.Values.PTuple", prog)),
            },
            PathElem::SeqIndex => {
                let k = &keys[ki];
                ki += 1;
                match cur {
                    Value::Seq(s) => {
                        let i = k.as_int();
                        if i < 0 || i as usize >= s.len() {
                            return Err(index_oor());
                        }
                        &mut Rc::make_mut(s)[i as usize]
                    }
                    Value::Null => return Err(nre()),
                    other => return Err(invalid_cast(other, "PChecker.Runtime.Values.PSeq", prog)),
                }
            }
            PathElem::MapKey => {
                let k = &keys[ki];
                ki += 1;
                match cur {
                    Value::Map(m) => match Rc::make_mut(m).get_mut(k) {
                        Some(v) => v,
                        None => return Err(key_not_found(&k.to_display(prog))),
                    },
                    Value::Null => return Err(nre()),
                    other => return Err(invalid_cast(other, "PChecker.Runtime.Values.PMap", prog)),
                }
            }
        };
        cur = next;
    }
    Ok(cur)
}

/// Performs the final store of an assignment through the last path element.
fn store_at(
    base: &mut Value,
    last: &PathElem,
    key: Option<&Value>,
    rhs: Value,
    prog: &Program,
) -> Result<(), Exception> {
    match last {
        PathElem::TupleField(i) => match base {
            Value::Tuple(t) => {
                *Rc::make_mut(t).get_mut(*i as usize).ok_or_else(index_oor)? = rhs;
            }
            Value::NamedTuple(t) => {
                *Rc::make_mut(t).values.get_mut(*i as usize).ok_or_else(index_oor)? = rhs;
            }
            Value::Null => return Err(nre()),
            other => return Err(invalid_cast(other, "PChecker.Runtime.Values.PTuple", prog)),
        },
        PathElem::SeqIndex => match base {
            Value::Seq(s) => {
                let i = key.expect("sequence index").as_int();
                if i < 0 || i as usize >= s.len() {
                    return Err(index_oor());
                }
                Rc::make_mut(s)[i as usize] = rhs;
            }
            Value::Null => return Err(nre()),
            other => return Err(invalid_cast(other, "PChecker.Runtime.Values.PSeq", prog)),
        },
        PathElem::MapKey => match base {
            Value::Map(m) => {
                Rc::make_mut(m).set(key.expect("map key").clone(), rhs);
            }
            Value::Null => return Err(nre()),
            other => return Err(invalid_cast(other, "PChecker.Runtime.Values.PMap", prog)),
        },
    }
    Ok(())
}

// The compiler balances the operand stack, so these never underflow.

fn pop_n(stack: &mut Vec<Value>, n: usize) -> Vec<Value> {
    let at = stack.len().checked_sub(n).expect("operand stack underflow");
    stack.split_off(at)
}

fn pop(stack: &mut Vec<Value>) -> Value {
    stack.pop().expect("operand stack underflow")
}

/// Invalidates the enumerators of `foreach` loops over the variable just modified.
fn note_mutation(exec: &mut Exec, frame_idx: usize, root: Root) {
    let hits: Vec<(usize, u32)> = exec
        .iters
        .iter()
        .filter(|it| it.root == root && (matches!(root, Root::Field(_)) || it.frame == frame_idx))
        .map(|it| (it.frame, it.slot))
        .collect();
    for (f, slot) in hits {
        if let Some(frame) = exec.frames.get_mut(f) {
            if let Some(v) = frame.locals.get_mut(slot as usize + 1) {
                *v = Value::Int(-1);
            }
        }
    }
}

fn arith(op: BinOp, a: Value, b: Value) -> Result<Value, Exception> {
    use Value::*;
    Ok(match (op, &a, &b) {
        (BinOp::Add, Int(x), Int(y)) => Int(x.wrapping_add(*y)),
        (BinOp::Sub, Int(x), Int(y)) => Int(x.wrapping_sub(*y)),
        (BinOp::Mul, Int(x), Int(y)) => Int(x.wrapping_mul(*y)),
        (BinOp::Div, Int(x), Int(y)) => {
            if *y == 0 {
                return Err(div_zero());
            }
            Int(x.checked_div(*y).ok_or_else(overflow)?)
        }
        (BinOp::Mod, Int(x), Int(y)) => {
            if *y == 0 {
                return Err(div_zero());
            }
            Int(x.checked_rem(*y).ok_or_else(overflow)?)
        }
        (BinOp::Add, Str(x), Str(y)) => Str(Rc::from(format!("{}{}", x, y))),
        (BinOp::Add, _, _) => Float(a.as_float() + b.as_float()),
        (BinOp::Sub, _, _) => Float(a.as_float() - b.as_float()),
        (BinOp::Mul, _, _) => Float(a.as_float() * b.as_float()),
        (BinOp::Div, _, _) => Float(a.as_float() / b.as_float()),
        (BinOp::Mod, _, _) => Float(a.as_float() % b.as_float()),
        (BinOp::Lt, Int(x), Int(y)) => Bool(x < y),
        (BinOp::Le, Int(x), Int(y)) => Bool(x <= y),
        (BinOp::Gt, Int(x), Int(y)) => Bool(x > y),
        (BinOp::Ge, Int(x), Int(y)) => Bool(x >= y),
        (BinOp::Lt, Str(x), Str(y)) => Bool(x < y),
        (BinOp::Le, Str(x), Str(y)) => Bool(x <= y),
        (BinOp::Gt, Str(x), Str(y)) => Bool(x > y),
        (BinOp::Ge, Str(x), Str(y)) => Bool(x >= y),
        (BinOp::Lt, _, _) => Bool(a.as_float() < b.as_float()),
        (BinOp::Le, _, _) => Bool(a.as_float() <= b.as_float()),
        (BinOp::Gt, _, _) => Bool(a.as_float() > b.as_float()),
        (BinOp::Ge, _, _) => Bool(a.as_float() >= b.as_float()),
        (BinOp::Eq, _, _) => Bool(a == b),
        (BinOp::Neq, _, _) => Bool(a != b),
        (BinOp::And, _, _) => Bool(a.as_bool() && b.as_bool()),
        (BinOp::Or, _, _) => Bool(a.as_bool() || b.as_bool()),
        (BinOp::Then, _, _) => Bool(!a.as_bool() || b.as_bool()),
        (BinOp::Iff, _, _) => Bool(a.as_bool() == b.as_bool()),
    })
}

impl<'p> Runtime<'p> {
    /// Runs (or resumes) an action until it finishes, unwinds or reaches a scheduling point.
    pub fn run_exec(&mut self, actor: Actor, mut exec: Exec) -> Result<ExecOutcome, Stop> {
        let r = self.run_exec_inner(actor, &mut exec);
        match r {
            Ok(Flow::Finished(v)) => Ok(ExecOutcome::Finished(v)),
            Ok(Flow::Yield(kind)) => Ok(ExecOutcome::Yield(exec, kind)),
            Ok(Flow::Unwound) => {
                if let (Actor::Machine(m), true) = (actor, self.xml.is_some()) {
                    // `raise`/`goto` return through a `PNonStandardReturnException`
                    const TY: &str = "PChecker.Runtime.Exceptions.PNonStandardReturnException";
                    let text = format!("{}: Exception of type '{}' was thrown.", TY, TY);
                    let (id, state) = (self.machines[m].name.to_string(), self.machine_state_name(m));
                    let attrs = [
                        ("id", id.as_str()),
                        ("state", state.as_str()),
                        ("action", exec.action.as_str()),
                        ("type", TY),
                    ];
                    self.xlog("ExceptionThrown", &attrs, Some(&text));
                    let handled: Vec<_> =
                        attrs.iter().copied().filter(|(k, v)| *k != "state" || !v.is_empty()).collect();
                    self.xlog("ExceptionHandled", &handled, Some(&text));
                }
                Ok(ExecOutcome::Unwound)
            }
            Ok(Flow::Continue) => Ok(ExecOutcome::Finished(Value::Null)),
            Err(Fault::Stop) => Err(Stop),
            Err(Fault::Exc(e)) => {
                let action = exec.action.clone();
                Err(self.report_exception(actor, &e, &action))
            }
        }
    }

    fn run_exec_inner(&mut self, actor: Actor, exec: &mut Exec) -> Result<Flow, Fault> {
        if let Some(p) = exec.pending.take() {
            match self.resume(actor, exec, p)? {
                Flow::Continue => {}
                other => return Ok(other),
            }
        }
        loop {
            if exec.frames.is_empty() {
                return Ok(Flow::Finished(Value::Null));
            }
            self.step_work += 1;
            let limit = self.config.max_work_per_step;
            if limit > 0 && self.step_work > limit {
                let msg = format!(
                    "{} in state {} exceeded the work bound of {} without reaching a scheduling point (possible infinite loop).",
                    self.actor_name(actor),
                    self.actor_state_name(actor),
                    limit
                );
                return Err(self.report_bug(msg).into());
            }
            match self.step(actor, exec)? {
                Flow::Continue => {}
                other => return Ok(other),
            }
        }
    }

    /// Calls a foreign function through the embedder's [`ForeignFunctions`].
    fn call_foreign(&mut self, actor: Actor, fun: prs_sema::FunId, args: &[Value]) -> Result<Value, Fault> {
        let prog = self.prog;
        let decl = prog.fun(fun);
        let name = decl.name.clone();
        let owner = decl.owner.map(|m| prog.machine(m).name.clone());
        let machine = match actor {
            Actor::Machine(m) => Some(m),
            Actor::Monitor(_) => None,
        };
        if decl.owner.is_none() {
            if let Some(r) = self.call_time_function(actor, &name, args) {
                return r.map_err(Into::into);
            }
        }
        let Some(f) = self.foreign.clone() else {
            return Err(foreign_missing(&name).into());
        };
        let mut cx = crate::foreign::ForeignCx { rt: self, machine };
        match f.call(&name, owner.as_deref(), args, &mut cx) {
            None => Err(foreign_missing(&name).into()),
            Some(Ok(v)) => Ok(v),
            Some(Err(crate::foreign::ForeignError::Stopped(s))) => Err(s.into()),
            Some(Err(crate::foreign::ForeignError::Exception(ty, msg))) => Err(Exception::new(ty, msg).into()),
        }
    }

    fn resume(&mut self, actor: Actor, exec: &mut Exec, p: Pending) -> Result<Flow, Fault> {
        let Actor::Machine(m) = actor else { unreachable!("monitors never reach a scheduling point") };
        match p {
            Pending::Create { machine, iface, payload } => {
                let v = self.finish_create(m, machine, iface, payload);
                if let Some(f) = exec.frames.last_mut() {
                    f.stack.push(v);
                }
                Ok(Flow::Continue)
            }
            Pending::Send { target, event } => {
                self.finish_send(m, &target, event);
                Ok(Flow::Continue)
            }
            Pending::ReceiveWait { cases } if self.timed() => self.do_receive(m, exec, cases, true),
            Pending::ReceiveWait { cases } => {
                // the operation is enabled again only by the event it waits for
                let e = self.machines[m].received.take().expect("a blocked receive resumes with its event");
                Ok(self.dispatch_receive(m, exec, &cases, e))
            }
            Pending::ReceiveFound { cases, event } => Ok(self.dispatch_receive(m, exec, &cases, event)),
        }
    }

    fn dispatch_receive(&mut self, m: usize, exec: &mut Exec, cases: &[ReceiveCase], e: Event) -> Flow {
        self.time_receive(m, &e);
        if let Some(case) = cases.iter().find(|c| c.event == e.id) {
            if let Some(frame) = exec.frames.last_mut() {
                if let Some(slot) = case.param_slot {
                    frame.locals[slot as usize] = e.payload;
                }
                frame.pc = case.target as usize;
            }
            Flow::Continue
        } else {
            // `halt` is received without a case: re-raise it (`case PHalt _hv: RaiseEvent(_hv)`).
            self.machines[m].pending_transition = Transition::Raise(e);
            Flow::Unwound
        }
    }

    fn actor_name(&self, actor: Actor) -> String {
        match actor {
            Actor::Machine(m) => self.machines[m].name.to_string(),
            Actor::Monitor(mon) => self.monitor_full_name(mon),
        }
    }

    fn actor_state_name(&self, actor: Actor) -> String {
        match actor {
            Actor::Machine(m) => self.machine_state_name(m),
            Actor::Monitor(mon) => self.state_name(self.monitors[mon].state),
        }
    }

    fn fields_mut(&mut self, actor: Actor) -> &mut Vec<Value> {
        match actor {
            Actor::Machine(m) => &mut self.machines[m].fields,
            Actor::Monitor(mon) => &mut self.monitors[mon].fields,
        }
    }

    fn make_event(&self, id: EventId, payload: Option<Value>) -> Event {
        let payload = match payload {
            Some(p) => p,
            None => default_value(&self.prog.event(id).payload, self.prog),
        };
        Event { id, payload, stamp: None, vt: None }
    }

    /// `Runtime.Assert(false, s)`: the message goes through `String.Format` with no
    /// arguments, so braces in it make upstream throw a `FormatException` instead.
    fn assertion_failure(&mut self, text: String) -> Fault {
        match format_composite(&text, &[]) {
            Ok(t) => self.report_bug(t).into(),
            Err(m) => format_exception(m).into(),
        }
    }

    fn choose_error(&self, actor: Actor, detail: &str) -> String {
        format!(
            "Incorrect use of choose by Machine {} in state {}. {}",
            self.actor_name(actor),
            self.actor_state_name(actor),
            detail
        )
    }

    fn pick(&mut self, actor: Actor, count: usize, what: &str) -> Result<usize, Fault> {
        if count == 0 {
            let msg = self.choose_error(actor, &format!("Trying to choose from an empty {}!", what));
            return Err(self.assertion_failure(msg));
        }
        if count > 10000 {
            let msg = self.choose_error(
                actor,
                &format!("choose expects a parameter with at most 10000 choices, got {} choices instead.", count),
            );
            return Err(self.assertion_failure(msg));
        }
        let name = self.actor_name(actor);
        Ok(self.choose_int(count as i32, &name)? as usize)
    }

    /// Executes one instruction.
    fn step(&mut self, actor: Actor, exec: &mut Exec) -> Result<Flow, Fault> {
        let frame_idx = exec.frames.len() - 1;
        let op = {
            let frame = &mut exec.frames[frame_idx];
            match frame.code.ops.get(frame.pc) {
                Some(op) => {
                    frame.pc += 1;
                    op.clone()
                }
                None => Op::Return,
            }
        };
        let prog = self.prog;
        macro_rules! frame {
            () => {
                exec.frames[frame_idx]
            };
        }
        macro_rules! stack {
            () => {
                exec.frames[frame_idx].stack
            };
        }
        match op {
            Op::PushNull => stack!().push(Value::Null),
            Op::PushBool(b) => stack!().push(Value::Bool(b)),
            Op::PushInt(i) => stack!().push(Value::Int(i)),
            Op::PushFloat(f) if self.config.java_semantics => stack!().push(Value::Float(f as f32 as f64)),
            Op::PushFloat(f) => stack!().push(Value::Float(f)),
            Op::PushStr(s) => stack!().push(Value::Str(s)),
            Op::PushEvent(e) => stack!().push(Value::Event(e)),
            Op::PushThis => {
                let v = match actor {
                    Actor::Machine(m) => Value::Machine(self.machines[m].this.clone()),
                    Actor::Monitor(_) => Value::Null,
                };
                stack!().push(v);
            }
            Op::LoadLocal(s) => {
                let v = frame!().locals[s as usize].clone();
                stack!().push(v);
            }
            Op::StoreLocal(s) => {
                let v = pop(&mut stack!());
                frame!().locals[s as usize] = v;
            }
            Op::LoadField(s) => {
                let v = self.fields_mut(actor)[s as usize].clone();
                stack!().push(v);
            }
            Op::LoadGlobal(s) => {
                let v = self.globals[s as usize].clone();
                stack!().push(v);
            }
            Op::Pop => {
                pop(&mut stack!());
            }
            Op::Dup => {
                let v = stack!().last().expect("operand stack underflow").clone();
                stack!().push(v);
            }
            Op::MakeTuple(n) => {
                let vs = pop_n(&mut stack!(), n as usize);
                stack!().push(Value::Tuple(Rc::new(vs)));
            }
            Op::MakeNamedTuple(names) => {
                let vs = pop_n(&mut stack!(), names.len());
                stack!().push(Value::NamedTuple(Rc::new(NamedTuple { names, values: vs })));
            }
            Op::TupleGet(i) => {
                let base = pop(&mut stack!());
                let v = match &base {
                    Value::Tuple(t) => t.get(i as usize).cloned().ok_or_else(index_oor)?,
                    Value::NamedTuple(t) => t.values.get(i as usize).cloned().ok_or_else(index_oor)?,
                    Value::Null => return Err(nre().into()),
                    other => return Err(invalid_cast(other, "PChecker.Runtime.Values.PTuple", prog).into()),
                };
                stack!().push(v);
            }
            Op::SeqGet => {
                let idx = pop(&mut stack!());
                let base = pop(&mut stack!());
                let v = match &base {
                    Value::Seq(s) => {
                        let i = idx.as_int();
                        if i < 0 || i as usize >= s.len() {
                            return Err(index_oor().into());
                        }
                        s[i as usize].clone()
                    }
                    Value::Null => return Err(nre().into()),
                    other => return Err(invalid_cast(other, "PChecker.Runtime.Values.PSeq", prog).into()),
                };
                stack!().push(v);
            }
            Op::SetGet => {
                let idx = pop(&mut stack!());
                let base = pop(&mut stack!());
                let v = match &base {
                    Value::Set(s) => {
                        let i = idx.as_int();
                        if i < 0 {
                            return Err(element_at_oor().into());
                        }
                        s.element_at(i as usize).cloned().ok_or_else(element_at_oor)?
                    }
                    Value::Null => return Err(nre().into()),
                    other => return Err(invalid_cast(other, "PChecker.Runtime.Values.PSet", prog).into()),
                };
                stack!().push(v);
            }
            Op::MapGet { java_null } => {
                let key = pop(&mut stack!());
                let base = pop(&mut stack!());
                let v = match &base {
                    Value::Map(m) if java_null && self.config.java_semantics => {
                        m.get(&key).cloned().unwrap_or(Value::Null)
                    }
                    Value::Map(m) => m.get(&key).cloned().ok_or_else(|| key_not_found(&key.to_display(prog)))?,
                    Value::Null => return Err(nre().into()),
                    other => return Err(invalid_cast(other, "PChecker.Runtime.Values.PMap", prog).into()),
                };
                stack!().push(v);
            }
            Op::Keys => {
                let base = pop(&mut stack!());
                let v = match &base {
                    Value::Map(m) if self.config.java_semantics => {
                        Value::Seq(Rc::new(crate::javafmt::map_keys_in_java_order(m)))
                    }
                    Value::Map(m) => Value::Seq(Rc::new(m.keys().cloned().collect())),
                    Value::Null => return Err(nre().into()),
                    other => return Err(invalid_cast(other, "PChecker.Runtime.Values.PMap", prog).into()),
                };
                stack!().push(v);
            }
            Op::Values => {
                let base = pop(&mut stack!());
                let v = match &base {
                    Value::Map(m) if self.config.java_semantics => {
                        Value::Seq(Rc::new(crate::javafmt::map_values_in_java_order(m)))
                    }
                    Value::Map(m) => Value::Seq(Rc::new(m.values().cloned().collect())),
                    Value::Null => return Err(nre().into()),
                    other => return Err(invalid_cast(other, "PChecker.Runtime.Values.PMap", prog).into()),
                };
                stack!().push(v);
            }
            Op::Sizeof => {
                let base = pop(&mut stack!());
                if base.is_null() {
                    return Err(nre().into());
                }
                stack!().push(Value::Int(base.count() as i64));
            }
            Op::Contains => {
                let coll = pop(&mut stack!());
                let item = pop(&mut stack!());
                let b = match &coll {
                    Value::Seq(s) => s.contains(&item),
                    Value::Set(s) => s.contains(&item),
                    Value::Map(m) => m.contains_key(&item),
                    Value::Null => return Err(nre().into()),
                    _ => false,
                };
                stack!().push(Value::Bool(b));
            }
            Op::Default(ty) => {
                stack!().push(ty.clone());
            }
            Op::Format { base, nargs, types } => {
                let vals = pop_n(&mut stack!(), nargs as usize);
                let text = if self.config.java_semantics {
                    // a string without arguments is a plain Java literal
                    if nargs == 0 {
                        base.to_string()
                    } else {
                        let args: Vec<String> = vals
                            .iter()
                            .zip(types.iter())
                            .map(|(v, t)| crate::javafmt::format_arg(v, t, prog))
                            .collect();
                        crate::javafmt::message_format(&base, &args)
                            .map_err(|m| Exception::new("java.lang.IllegalArgumentException", m))?
                    }
                } else {
                    // render every argument into one reused buffer, then slice it
                    let buf = &mut self.format_buf;
                    buf.clear();
                    let mut ends = Vec::with_capacity(vals.len());
                    for v in &vals {
                        v.write_display(buf, prog);
                        ends.push(buf.len());
                    }
                    let args: Vec<&str> = ends
                        .iter()
                        .scan(0, |start, &end| {
                            let arg = &buf[*start..end];
                            *start = end;
                            Some(arg)
                        })
                        .collect();
                    format_composite(&base, &args).map_err(format_exception)?
                };
                stack!().push(Value::Str(Rc::from(text)));
            }
            Op::Bin(op) => {
                let b = pop(&mut stack!());
                let a = pop(&mut stack!());
                let r = arith(op, a, b)?;
                // P floats are Java `float`s in the PObserve backend
                let r = match r {
                    Value::Float(f) if self.config.java_semantics => Value::Float(f as f32 as f64),
                    r => r,
                };
                stack!().push(r);
            }
            Op::Neg => {
                let a = pop(&mut stack!());
                let v = match a {
                    Value::Int(i) => Value::Int(i.wrapping_neg()),
                    Value::Float(f) => Value::Float(-f),
                    Value::Null => return Err(nre().into()),
                    other => other,
                };
                stack!().push(v);
            }
            Op::Not => {
                let a = pop(&mut stack!());
                if a.is_null() {
                    return Err(nre().into());
                }
                stack!().push(Value::Bool(!a.as_bool()));
            }
            Op::Jump(t) => frame!().pc = t as usize,
            Op::JumpIfFalse(t) => {
                let c = pop(&mut stack!());
                if !c.as_bool() {
                    frame!().pc = t as usize;
                }
            }
            Op::JumpIfTrue(t) => {
                let c = pop(&mut stack!());
                if c.as_bool() {
                    frame!().pc = t as usize;
                }
            }
            Op::Cast(ty) => {
                check_cast(stack!().last().expect("operand stack underflow"), &ty, prog)?;
            }
            Op::CoerceFloat => {
                let v = pop(&mut stack!());
                let v = match v {
                    Value::Int(i) => Value::Float(i as f64),
                    Value::Null => return Err(nre().into()),
                    other => other,
                };
                stack!().push(v);
            }
            Op::CoerceInt => {
                let v = pop(&mut stack!());
                let v = match v {
                    Value::Float(f) => Value::Int(f as i64),
                    Value::Null => return Err(nre().into()),
                    other => other,
                };
                stack!().push(v);
            }
            Op::CoerceInterface(iface) => {
                let v = pop(&mut stack!());
                let v = match v {
                    Value::Machine(r) => {
                        let perms = self.code.permissions[iface.idx()].clone();
                        if !perms.iter().all(|e| r.permissions.contains(e)) {
                            return Err(Exception::new(
                                "PChecker.Runtime.Exceptions.PIllegalCoercionException",
                                format!("value cannot be coerced to interface I_{}", prog.interface(iface).name),
                            )
                            .into());
                        }
                        Value::Machine(Rc::new(crate::value::MachineRef {
                            id: r.id,
                            name: r.name.clone(),
                            permissions: perms,
                        }))
                    }
                    other => other,
                };
                stack!().push(v);
            }
            Op::ChooseBool => {
                let name = self.actor_name(actor);
                let b = self.choose_bool(2, &name)?;
                stack!().push(Value::Bool(b));
            }
            Op::ChooseInt => {
                let v = pop(&mut stack!());
                if v.is_null() {
                    return Err(nre().into());
                }
                let max = v.as_int();
                if max > 10000 {
                    let msg = self.choose_error(
                        actor,
                        "choose expects a parameter with at most 10000 choices, got {maxValue} choices instead.",
                    );
                    return Err(self.assertion_failure(msg));
                }
                let name = self.actor_name(actor);
                let i = self.choose_int(max.max(0) as i32, &name)?;
                stack!().push(Value::Int(i as i64));
            }
            Op::ChooseSeq => {
                let v = pop(&mut stack!());
                let Value::Seq(s) = &v else { return Err(nre().into()) };
                let i = self.pick(actor, s.len(), "sequence")?;
                let r = s[i].clone();
                stack!().push(r);
            }
            Op::ChooseSet => {
                let v = pop(&mut stack!());
                let Value::Set(s) = &v else { return Err(nre().into()) };
                let i = self.pick(actor, s.len(), "set")?;
                let r = s.element_at(i).expect("choice in range").clone();
                stack!().push(r);
            }
            Op::ChooseMap => {
                let v = pop(&mut stack!());
                let Value::Map(m) = &v else { return Err(nre().into()) };
                let i = self.pick(actor, m.len(), "map")?;
                let r = m.key_at(i).expect("choice in range").clone();
                stack!().push(r);
            }
            Op::Call { fun, nargs } => {
                let args = pop_n(&mut stack!(), nargs as usize);
                match self.code.get(fun) {
                    Some(code) => {
                        exec.frames.push(make_frame(code.clone(), fun, args));
                    }
                    None => {
                        let v = self.call_foreign(actor, fun, &args)?;
                        stack!().push(v);
                    }
                }
            }
            Op::Return => {
                let v = pop(&mut stack!());
                exec.frames.pop();
                exec.iters.retain(|it| it.frame < exec.frames.len());
                match exec.frames.last_mut() {
                    Some(f) => f.stack.push(v),
                    None => return Ok(Flow::Finished(v)),
                }
            }
            Op::Store { root, path } => {
                let rhs = pop(&mut stack!());
                let keys = pop_n(&mut stack!(), key_count(&path));
                if path.len() == 1 {
                    note_mutation(exec, frame_idx, root);
                }
                let target: &mut Value = match root {
                    Root::Local(s) => &mut exec.frames[frame_idx].locals[s as usize],
                    Root::Field(s) => &mut self.fields_mut(actor)[s as usize],
                };
                match path.split_last() {
                    None => *target = rhs,
                    Some((last, init)) => {
                        let nk = key_count(init);
                        let base = navigate(target, init, &keys[..nk], prog)?;
                        store_at(base, last, keys.get(nk), rhs, prog)?;
                    }
                }
            }
            Op::Insert { root, path } => {
                let value = pop(&mut stack!());
                let index = pop(&mut stack!());
                let keys = pop_n(&mut stack!(), key_count(&path));
                if path.is_empty() {
                    note_mutation(exec, frame_idx, root);
                }
                let target: &mut Value = match root {
                    Root::Local(s) => &mut exec.frames[frame_idx].locals[s as usize],
                    Root::Field(s) => &mut self.fields_mut(actor)[s as usize],
                };
                let coll = navigate(target, &path, &keys, prog)?;
                match coll {
                    Value::Seq(s) => {
                        let i = index.as_int();
                        if i < 0 || i as usize > s.len() {
                            return Err(insert_oor().into());
                        }
                        Rc::make_mut(s).insert(i as usize, value);
                    }
                    Value::Map(m) => {
                        if !Rc::make_mut(m).add(index.clone(), value) {
                            return Err(duplicate_key(&index.to_display(prog)).into());
                        }
                    }
                    Value::Set(s) => {
                        Rc::make_mut(s).insert(value);
                    }
                    Value::Null => return Err(nre().into()),
                    other => return Err(invalid_cast(other, "PChecker.Runtime.Values.PSeq", prog).into()),
                }
            }
            Op::Add { root, path } => {
                let value = pop(&mut stack!());
                let keys = pop_n(&mut stack!(), key_count(&path));
                if path.is_empty() {
                    note_mutation(exec, frame_idx, root);
                }
                let target: &mut Value = match root {
                    Root::Local(s) => &mut exec.frames[frame_idx].locals[s as usize],
                    Root::Field(s) => &mut self.fields_mut(actor)[s as usize],
                };
                let coll = navigate(target, &path, &keys, prog)?;
                match coll {
                    Value::Set(s) => {
                        Rc::make_mut(s).insert(value);
                    }
                    Value::Seq(s) => Rc::make_mut(s).push(value),
                    Value::Null => return Err(nre().into()),
                    other => return Err(invalid_cast(other, "PChecker.Runtime.Values.PSet", prog).into()),
                }
            }
            Op::Remove { root, path } => {
                let value = pop(&mut stack!());
                let keys = pop_n(&mut stack!(), key_count(&path));
                if path.is_empty() {
                    note_mutation(exec, frame_idx, root);
                }
                let target: &mut Value = match root {
                    Root::Local(s) => &mut exec.frames[frame_idx].locals[s as usize],
                    Root::Field(s) => &mut self.fields_mut(actor)[s as usize],
                };
                let coll = navigate(target, &path, &keys, prog)?;
                match coll {
                    Value::Seq(s) => {
                        let i = value.as_int();
                        if i < 0 || i as usize >= s.len() {
                            return Err(index_oor().into());
                        }
                        Rc::make_mut(s).remove(i as usize);
                    }
                    Value::Map(m) => {
                        Rc::make_mut(m).remove(&value);
                    }
                    Value::Set(s) => {
                        Rc::make_mut(s).remove(&value);
                    }
                    Value::Null => return Err(nre().into()),
                    other => return Err(invalid_cast(other, "PChecker.Runtime.Values.PSeq", prog).into()),
                }
            }
            Op::Assert => {
                let msg = pop(&mut stack!());
                let cond = pop(&mut stack!());
                if !cond.as_bool() {
                    if self.config.java_semantics {
                        // `Monitor.tryAssert`: a `PAssertionFailureException`
                        let text = format!("Assertion failure: {}", msg.to_display(prog));
                        self.failure.get_or_insert(crate::runtime::Failure::Assertion(text.clone()));
                        return Err(self.report_bug(text).into());
                    }
                    let text = format!("Assertion Failed: {}", msg.to_display(prog));
                    return Err(self.assertion_failure(text));
                }
            }
            Op::Print if self.config.java_semantics => {
                // the generated monitors log through a java.util.logging logger that is off
                pop(&mut stack!());
            }
            Op::Print => {
                let msg = pop(&mut stack!());
                let text = msg.to_display(prog);
                self.log(|_| format!("<PrintLog> {}", text));
                if self.json_enabled() {
                    self.jlog("Print", crate::jsonlog::Details { log: Some(text), ..Default::default() }, false);
                }
            }
            Op::Create { iface, has_payload } => {
                let payload = if has_payload { pop(&mut stack!()) } else { Value::Null };
                let Actor::Machine(m) = actor else {
                    return Err(Exception::new("System.InvalidOperationException", "monitors cannot create machines")
                        .from_source(crate::runtime::P_CHECKER_CORE)
                        .into());
                };
                let creator_iface = self.machines[m].iface;
                let created = self.module.link(creator_iface, iface).unwrap_or(iface);
                let decl = self.machines[m].decl;
                if !prog.machine(decl).creates.contains(&created) {
                    let msg = format!(
                        "Machine {} cannot create interface I_{}, not in its creates set",
                        prog.machine(decl).cs_name,
                        prog.interface(created).name
                    );
                    return Err(self.report_bug(msg).into());
                }
                let Some(machine) = self.module.machine_for(created) else {
                    let msg = format!(
                        "interface I_{} is not bound to a machine in this module",
                        prog.interface(created).name
                    );
                    return Err(self.report_bug(msg).into());
                };
                exec.pending = Some(Pending::Create { machine, iface: created, payload });
                return Ok(Flow::Yield(SchedKind::Create));
            }
            Op::Send { has_payload } => {
                let payload = if has_payload { Some(pop(&mut stack!())) } else { None };
                let event = pop(&mut stack!());
                let target = pop(&mut stack!());
                let Actor::Machine(m) = actor else {
                    return Err(Exception::new("System.InvalidOperationException", "monitors cannot send events")
                        .from_source(crate::runtime::P_CHECKER_CORE)
                        .into());
                };
                let name = self.machines[m].name.clone();
                let state = self.machine_state_name(m);
                let ev_id = match &event {
                    Value::Event(e) => *e,
                    Value::Null => {
                        let msg = format!(
                            "Machine cannot send a null event. Machine {} trying to send null event in state {}.",
                            name, state
                        );
                        return Err(self.report_bug(msg).into());
                    }
                    other => return Err(invalid_cast(other, "PChecker.Runtime.Events.Event", prog).into()),
                };
                let ev_name = self.event_name(ev_id).to_string();
                let target = match target {
                    Value::Machine(r) => r,
                    Value::Null => {
                        let msg = format!(
                            "Target in send cannot be null. Machine {} trying to send event {} to null target in state {}.",
                            name, ev_name, state
                        );
                        return Err(self.report_bug(msg).into());
                    }
                    other => return Err(invalid_cast(&other, "PChecker.Runtime.Values.PMachineValue", prog).into()),
                };
                let decl = self.machines[m].decl;
                if let Some(sends) = &prog.machine(decl).sends {
                    if !sends.contains(&ev_id) {
                        let msg = format!(
                            "Event {} is not in the sends set of the Machine {}",
                            ev_name,
                            prog.machine(decl).cs_name
                        );
                        return Err(self.report_bug(msg).into());
                    }
                }
                if !target.permissions.contains(&ev_id) {
                    let msg = format!(
                        "Event {} is not in the permissions set of the target machine {}",
                        ev_name, target.name
                    );
                    return Err(self.report_bug(msg).into());
                }
                let e = self.make_event(ev_id, payload);
                self.announce_internal(m, &e)?;
                let cur = self.current;
                self.op_mut(cur).receiver = Some(target.id);
                exec.pending = Some(Pending::Send { target, event: e });
                return Ok(Flow::Yield(SchedKind::Send));
            }
            Op::Raise { has_payload } => {
                let payload = if has_payload { Some(pop(&mut stack!())) } else { None };
                let event = pop(&mut stack!());
                let name = self.actor_name(actor);
                let state = self.actor_state_name(actor);
                if let Actor::Machine(m) = actor {
                    if self.machines[m].status != MachineStatus::Active {
                        let msg = format!("{} invoked RaiseEvent while halting.", name);
                        return Err(self.report_bug(msg).into());
                    }
                }
                let ev_id = match &event {
                    Value::Event(e) => *e,
                    Value::Null => {
                        let msg = format!("{} is raising a null event in state {}", name, state);
                        return Err(self.report_bug(msg).into());
                    }
                    other => return Err(invalid_cast(other, "PChecker.Runtime.Events.Event", prog).into()),
                };
                let e = self.make_event(ev_id, payload);
                match actor {
                    Actor::Machine(m) => self.machines[m].pending_transition = Transition::Raise(e),
                    Actor::Monitor(mon) => self.monitors[mon].pending_transition = Transition::Raise(e),
                }
                return Ok(Flow::Unwound);
            }
            Op::Announce { has_payload } => {
                let payload = if has_payload { pop(&mut stack!()) } else { Value::Null };
                let event = pop(&mut stack!());
                let Actor::Machine(m) = actor else {
                    return Err(Exception::new("System.InvalidOperationException", "monitors cannot announce events")
                        .from_source(crate::runtime::P_CHECKER_CORE)
                        .into());
                };
                let ev_id = match &event {
                    Value::Event(e) => *e,
                    Value::Null => {
                        let msg = format!(
                            "Incorrect use of announce by Machine {} in state {}. Machine cannot announce a null event",
                            self.machines[m].name,
                            self.machine_state_name(m)
                        );
                        return Err(self.report_bug(msg).into());
                    }
                    other => return Err(invalid_cast(other, "PChecker.Runtime.Events.Event", prog).into()),
                };
                let ptext = if payload.is_null() || !(self.logging || self.json_enabled()) {
                    String::new()
                } else {
                    format!(" with payload {}", payload.to_escaped(prog))
                };
                self.log(|rt| {
                    format!(
                        "<AnnounceLog> '{}' announced event '{}'{}.",
                        rt.machines[m].name,
                        rt.event_name(ev_id),
                        ptext
                    )
                });
                if self.json_enabled() {
                    let name = self.machines[m].name.to_string();
                    let ename = self.event_name(ev_id).to_string();
                    let d = crate::jsonlog::Details {
                        log: Some(format!("{} announced event {}{}.", name, ename, ptext)),
                        id: Some(name),
                        event: Some(ename),
                        payload: (!payload.is_null()).then(|| crate::jsonlog::to_dict(&payload, prog)),
                        ..Default::default()
                    };
                    self.jlog("Announce", d, true);
                }
                let e = Event { id: ev_id, payload, stamp: None, vt: None };
                self.announce_internal(m, &e)?;
            }
            Op::Goto { state, has_payload } => {
                let payload = if has_payload { Some(pop(&mut stack!())) } else { None };
                match actor {
                    Actor::Machine(m) => {
                        if self.machines[m].status != MachineStatus::Active {
                            let msg = format!("{} invoked GotoState while halting.", self.machines[m].name);
                            return Err(self.report_bug(msg).into());
                        }
                        self.machines[m].goto_payload = payload;
                        self.machines[m].pending_transition = Transition::Goto(state);
                    }
                    Actor::Monitor(mon) => {
                        self.monitors[mon].goto_payload = payload;
                        self.monitors[mon].pending_transition = Transition::Goto(state);
                    }
                }
                return Ok(Flow::Unwound);
            }
            Op::Receive { cases } => {
                let Actor::Machine(m) = actor else {
                    return Err(Exception::new("System.InvalidOperationException", "monitors cannot receive events")
                        .from_source(crate::runtime::P_CHECKER_CORE)
                        .into());
                };
                return self.do_receive(m, exec, cases, false);
            }
            Op::ForeachInit { iter_slot, root } => {
                let coll = pop(&mut stack!());
                if coll.is_null() {
                    return Err(nre().into());
                }
                frame!().locals[iter_slot as usize] = coll;
                frame!().locals[iter_slot as usize + 1] = Value::Int(0);
                exec.iters.retain(|it| !(it.frame == frame_idx && it.slot == iter_slot));
                if let Some(root) = root {
                    exec.iters.push(ActiveIter { frame: frame_idx, slot: iter_slot, root });
                }
            }
            Op::ForeachNext { iter_slot, item_slot, exit } => {
                let idx = frame!().locals[iter_slot as usize + 1].as_int();
                if idx < 0 {
                    return Err(modified().into());
                }
                let item = match &frame!().locals[iter_slot as usize] {
                    Value::Seq(s) => s.get(idx as usize).cloned(),
                    Value::Set(s) => s.element_at(idx as usize).cloned(),
                    _ => unreachable!("foreach over a sequence or set"),
                };
                match item {
                    Some(v) => {
                        frame!().locals[item_slot as usize] = v;
                        frame!().locals[iter_slot as usize + 1] = Value::Int(idx + 1);
                    }
                    None => {
                        exec.iters.retain(|it| !(it.frame == frame_idx && it.slot == iter_slot));
                        frame!().pc = exit as usize;
                    }
                }
            }
        }
        Ok(Flow::Continue)
    }

    /// `receive { ... }`: takes a matching event from the queue or blocks.
    /// `ReceiveEventAsync`. `at_point`: resumed at a scheduling point (`--timed`, where
    /// the event is picked only there, after earlier senders have run).
    fn do_receive(
        &mut self,
        m: usize,
        exec: &mut Exec,
        cases: Rc<[ReceiveCase]>,
        at_point: bool,
    ) -> Result<Flow, Fault> {
        if self.machines[m].status != MachineStatus::Active {
            let msg = format!("{} invoked ReceiveEventAsync while halting.", self.machines[m].name);
            return Err(self.report_bug(msg).into());
        }
        let wait = crate::runtime::receive_wait_set(self.prog, &cases);
        let name = self.machines[m].name.clone();
        let state = self.log_state_name(m);
        let timed = self.timed();
        let found = if timed && !at_point {
            self.timed_candidates(m, Some(&wait)).first().copied()
        } else if timed {
            self.timed_pick(m, Some(&wait), false)?
        } else {
            self.machines[m].inbox.queue.iter().position(|e| wait.contains(&e.id))
        };
        if found.is_some() && timed && !at_point {
            exec.pending = Some(Pending::ReceiveWait { cases });
            return Ok(Flow::Yield(SchedKind::Receive));
        }
        if let Some(i) = found {
            let e = self.machines[m].inbox.queue.remove(i).unwrap();
            let ev = if self.logging || self.json_enabled() { self.event_with_payload(&e) } else { String::new() };
            self.log(|_| format!("<ReceiveLog> '{}' dequeued event '{}' in state '{}'.", name, ev, state));
            if self.json_enabled() {
                let (payload, key) = self.jpayload(&e);
                let d = crate::jsonlog::Details {
                    log: Some(format!("'{}' dequeued event '{}' in state '{}'.", name, ev, state)),
                    id: Some(name.to_string()),
                    state: Some(self.short_state(self.machines[m].state)),
                    event: Some(self.event_name(e.id).to_string()),
                    payload,
                    was_blocked: Some(false),
                    payload_key: key,
                    ..Default::default()
                };
                self.jlog("ReceiveEvent", d, true);
            }
            if self.xml.is_some() {
                let ev = self.xml_event(e.id);
                self.xlog(
                    "Receive",
                    &[("id", &name), ("state", &state), ("event", &ev), ("wasBlocked", "False")],
                    None,
                );
            }
            self.cov_receive(m, e.id);
            if timed {
                return Ok(self.dispatch_receive(m, exec, &cases, e));
            }
            exec.pending = Some(Pending::ReceiveFound { cases, event: e });
            return Ok(Flow::Yield(SchedKind::Receive));
        }
        let names: Vec<String> = wait.iter().map(|e| self.event_name(*e).to_string()).collect();
        let event_names = if names.len() == 1 {
            // the single-type overload logs the full type name
            format!("'{}'", event_cs_full_name(self.prog, wait[0]))
        } else {
            wait_event_names(&names)
        };
        self.log(|_| {
            format!(
                "<ReceiveLog> {} is waiting to dequeue an event of type {} in state '{}'.",
                name, event_names, state
            )
        });
        if self.json_enabled() {
            let d = crate::jsonlog::Details {
                log: Some(format!(
                    "{} is waiting to dequeue an event of type {} in state '{}'.",
                    name, event_names, state
                )),
                id: Some(name.to_string()),
                event_types: Some(names.clone()),
                ..Default::default()
            };
            self.jlog("WaitMultipleEvents", d, true);
        }
        if self.xml.is_some() {
            let evs: Vec<String> = wait.iter().map(|e| self.xml_event(*e)).collect();
            self.xlog("WaitEvent", &[("id", &name), ("event", &evs.join(", ")), ("state", &state)], None);
        }
        self.machines[m].inbox.waiting = Some(Rc::from(wait));
        let id = self.machines[m].id;
        self.op_mut(id).status = OpStatus::BlockedOnReceive;
        exec.pending = Some(Pending::ReceiveWait { cases });
        Ok(Flow::Yield(SchedKind::Join))
    }
}

/// Event list of the receive wait log (`OnWaitEvent`), including upstream's quirk of
/// leaving the second-to-last name empty for four or more types.
fn wait_event_names(names: &[String]) -> String {
    match names.len() {
        0 => "'<missing>'".to_string(),
        1 => format!("'{}'", names[0]),
        2 => format!("'{}' or '{}'", names[0], names[1]),
        3 => format!("'{}', '{}' or '{}'", names[0], names[1], names[2]),
        n => {
            let mut arr: Vec<&str> = names[..n - 2].iter().map(String::as_str).collect();
            arr.push("");
            format!("'{}' or '{}'", arr.join("', '"), names[n - 1])
        }
    }
}

fn foreign_missing(name: &str) -> Exception {
    Exception::new(
        "System.NotImplementedException",
        format!("foreign function '{}' has no implementation registered with the checker", name),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_division_overflow_throws() {
        for op in [BinOp::Div, BinOp::Mod] {
            let e = arith(op, Value::Int(i64::MIN), Value::Int(-1)).unwrap_err();
            assert_eq!(e.ty, "System.OverflowException");
            assert_eq!(e.message, "Arithmetic operation resulted in an overflow.");
            assert_eq!(e.source, Some(crate::runtime::P_CHECKER_CORE));
        }
        assert_eq!(arith(BinOp::Div, Value::Int(i64::MIN + 1), Value::Int(-1)).unwrap(), Value::Int(i64::MAX));
        assert_eq!(arith(BinOp::Mod, Value::Int(-7), Value::Int(-1)).unwrap(), Value::Int(0));
        // the other operations wrap, as unchecked C# arithmetic does
        assert_eq!(arith(BinOp::Mul, Value::Int(i64::MIN), Value::Int(-1)).unwrap(), Value::Int(i64::MIN));
    }
}
