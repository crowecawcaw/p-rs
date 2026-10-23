//! Code coverage: which statements, branches and event handlers of a test case's
//! machines and monitors ran over all its schedules, and what never did.
//!
//! A checker built with [`Checker::with_code_coverage`](crate::checker::engine::Checker::with_code_coverage)
//! records the [`Probe`]s that run in its activity coverage; [`CodeCoverage::of_case`]
//! adds what the test case could have run, and coverage of several test cases merges.

use rustc_hash::FxHashSet;

use crate::checker::bytecode::{Probe, ProbeKind};
use crate::checker::coverage::{CoverageInfo, CoverageTables, ProbeSet};
use crate::sema::{EventId, FunId, HandlerAction, Program, StateId, TestCase};
use crate::syntax::{SourceMap, Span};

/// What some test cases ran, and what they could have.
#[derive(Debug, Clone, Default)]
pub struct CodeCoverage {
    /// Probes that ran.
    pub hit: ProbeSet,
    /// `(state, event)` handlers that handled an event.
    pub handled: FxHashSet<(StateId, EventId)>,
    /// Machines and monitors the test cases include, by `MachineId`.
    pub machines: Vec<bool>,
    /// Functions they include: those of their machines and the global functions those call.
    pub funs: Vec<bool>,
}

/// Covered and total counts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Count {
    pub covered: usize,
    pub total: usize,
}

impl Count {
    fn add(&mut self, covered: bool) {
        self.total += 1;
        self.covered += usize::from(covered);
    }
}

/// Coverage counts of a [`CodeCoverage`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Summary {
    pub statements: Count,
    pub branches: Count,
    pub handlers: Count,
}

/// Code that never ran.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Uncovered {
    pub file: String,
    pub line: u32,
    /// Last line, for a range of statements.
    pub end_line: u32,
    /// Where the code is, e.g. `Server.Serving, on eRequest`.
    pub context: String,
    /// What never ran, e.g. `never handled` or `if (x > 0) { never true`.
    pub what: String,
}

impl Uncovered {
    /// `file:line` or `file:line-end`.
    pub fn location(&self) -> String {
        if self.end_line > self.line {
            format!("{}:{}-{}", self.file, self.line, self.end_line)
        } else {
            format!("{}:{}", self.file, self.line)
        }
    }
}

impl CodeCoverage {
    /// The code coverage of test case `tc` from its activity coverage `info`.
    pub fn of_case(prog: &Program, tc: &TestCase, tables: &CoverageTables, info: &CoverageInfo) -> CodeCoverage {
        let (module, main) = prog.test_case_module(tc);
        let mut machines = vec![false; prog.machines.len()];
        let included =
            module.interface_def.iter().map(|(_, m)| *m).chain(module.monitor_map.iter().map(|(m, _)| *m)).chain(main);
        for m in included {
            machines[m.idx()] = true;
        }
        let mut funs = vec![false; prog.funs.len()];
        let mut work: Vec<FunId> = Vec::new();
        for (i, f) in prog.funs.iter().enumerate() {
            if f.owner.is_some_and(|m| machines[m.idx()]) {
                funs[i] = true;
                work.push(FunId(i as u32));
            }
        }
        while let Some(f) = work.pop() {
            for c in &prog.fun(f).callees {
                if !funs[c.idx()] {
                    funs[c.idx()] = true;
                    work.push(*c);
                }
            }
        }
        let mut handled = FxHashSet::default();
        for (s, e) in handlers(prog, &machines) {
            if info.received.contains(&(tables.plain(s), e.0)) {
                handled.insert((s, e));
            }
        }
        CodeCoverage { hit: info.probes.clone(), handled, machines, funs }
    }

    /// Adds another test case's coverage.
    pub fn merge(&mut self, other: &CodeCoverage) {
        self.hit.union(&other.hit);
        self.handled.extend(other.handled.iter().copied());
        merge_flags(&mut self.machines, &other.machines);
        merge_flags(&mut self.funs, &other.funs);
    }

    fn includes(&self, f: FunId) -> bool {
        self.funs.get(f.idx()).copied().unwrap_or(false)
    }

    /// Covered and total statements, branches and event handlers. Code that must never
    /// run (`assert false`) is left out.
    pub fn summary(&self, prog: &Program, probes: &[Probe]) -> Summary {
        let mut s = Summary::default();
        for (id, p) in probes.iter().enumerate() {
            if !self.includes(p.fun) || must_not_run(probes, id) {
                continue;
            }
            let hit = self.hit.contains(id as u32);
            match p.kind {
                ProbeKind::Stmt => s.statements.add(hit),
                ProbeKind::Then | ProbeKind::Else => s.branches.add(hit),
                ProbeKind::Entry => {}
            }
        }
        let entries = entries(probes);
        for h in handlers(prog, &self.machines) {
            if !handler_must_not_run(prog, probes, &entries, h) {
                s.handlers.add(self.handled.contains(&h));
            }
        }
        s
    }

    /// What never ran, by file and line: for each piece of code that did not run, the
    /// outermost one (a handler that never ran, not each of its statements).
    pub fn uncovered(&self, prog: &Program, sm: &SourceMap, probes: &[Probe]) -> Vec<Uncovered> {
        let mut out = Vec::new();
        // functions explained by a handler that never ran
        let mut explained: FxHashSet<FunId> = FxHashSet::default();
        let entries = entries(probes);
        for (s, e) in handlers(prog, &self.machines) {
            if self.handled.contains(&(s, e)) || handler_must_not_run(prog, probes, &entries, (s, e)) {
                continue;
            }
            let st = prog.state(s);
            let Some(h) = st.handler_for(e) else { continue };
            match h.action {
                HandlerAction::Do(f) | HandlerAction::Goto { with: Some(f), .. } => {
                    explained.insert(f);
                }
                _ => {}
            }
            let what = match h.action {
                HandlerAction::Goto { target, .. } => format!("goto {} never taken", prog.state(target).name),
                _ => "never handled".to_string(),
            };
            let context = format!("{}.{}, on {}", prog.machine(st.machine).name, st.name, prog.event(e).name);
            push(&mut out, sm, h.span, None, context, what);
        }
        let hit = |id: Option<u32>| id.is_none_or(|i| self.hit.contains(i));
        let mut id = 0;
        while id < probes.len() {
            let p = &probes[id];
            let this = id as u32;
            id += 1;
            if !self.includes(p.fun) || hit(Some(this)) || !hit(p.parent) || must_not_run(probes, this as usize) {
                continue;
            }
            let context = fun_context(prog, p.fun);
            match p.kind {
                ProbeKind::Entry if p.parent.is_none() && explained.contains(&p.fun) => {}
                ProbeKind::Entry => {
                    let f = prog.fun(p.fun);
                    let what = if f.parent.is_some() { "receive case never ran" } else { "never ran" };
                    push(&mut out, sm, p.span, None, context, what.to_string());
                }
                ProbeKind::Then | ProbeKind::Else => {
                    let never = if p.kind == ProbeKind::Then { "never true" } else { "never false" };
                    push(&mut out, sm, p.span, None, context, format!("{} {}", condition(sm, p.span), never));
                }
                ProbeKind::Stmt => {
                    // the run of unreached statements that starts here
                    let mut last = p.span;
                    while let Some(q) = probes.get(id) {
                        let reported = q.kind == ProbeKind::Stmt && q.parent == p.parent;
                        let nested = q.parent.is_some_and(|par| !self.hit.contains(par) && par >= this);
                        if !(reported || nested) || self.hit.contains(id as u32) {
                            break;
                        }
                        if reported {
                            last = q.span;
                        }
                        id += 1;
                    }
                    let what = format!("not reached: {}", snippet(sm, p.span));
                    push(&mut out, sm, p.span, Some(last), context, what);
                }
            }
        }
        out.sort_by(|a, b| (&a.file, a.line, &a.what).cmp(&(&b.file, b.line, &b.what)));
        out.dedup();
        out
    }
}

/// Whether probe `id` marks code that must never run: an `assert false` statement, or a
/// branch or function that starts with one.
fn must_not_run(probes: &[Probe], id: usize) -> bool {
    let first_child = probes.get(id + 1).filter(|c| c.kind == ProbeKind::Stmt && c.parent == Some(id as u32));
    probes[id].asserts_false || first_child.is_some_and(|c| c.asserts_false)
}

/// The entry probe of each root function, by [`FunId`].
fn entries(probes: &[Probe]) -> rustc_hash::FxHashMap<FunId, usize> {
    let roots = probes.iter().enumerate().filter(|(_, p)| p.kind == ProbeKind::Entry && p.parent.is_none());
    roots.map(|(id, p)| (p.fun, id)).collect()
}

/// Whether handler `h` runs a function that starts with `assert false`.
fn handler_must_not_run(
    prog: &Program,
    probes: &[Probe],
    entries: &rustc_hash::FxHashMap<FunId, usize>,
    (s, e): (StateId, EventId),
) -> bool {
    let fun = match prog.state(s).handler_for(e).map(|h| &h.action) {
        Some(HandlerAction::Do(f)) | Some(HandlerAction::Goto { with: Some(f), .. }) => *f,
        _ => return false,
    };
    entries.get(&fun).is_some_and(|id| must_not_run(probes, *id))
}

fn merge_flags(a: &mut Vec<bool>, b: &[bool]) {
    if a.len() < b.len() {
        a.resize(b.len(), false);
    }
    for (x, y) in a.iter_mut().zip(b) {
        *x |= *y;
    }
}

/// The `(state, event)` handlers that do or goto, of the states of `machines`.
fn handlers(prog: &Program, machines: &[bool]) -> Vec<(StateId, EventId)> {
    let mut out = Vec::new();
    for (i, m) in prog.machines.iter().enumerate() {
        if !machines.get(i).copied().unwrap_or(false) {
            continue;
        }
        for s in &m.states {
            for h in &prog.state(*s).handlers {
                if matches!(h.action, HandlerAction::Do(_) | HandlerAction::Goto { .. }) {
                    out.push((*s, h.event));
                }
            }
        }
    }
    out
}

/// Where function `f` runs, e.g. `Server.Serving, on eRequest`, `Server.Init entry`,
/// `Server.helper` or `helper`.
fn fun_context(prog: &Program, f: FunId) -> String {
    let decl = prog.fun(f);
    if let Some(parent) = decl.parent {
        return fun_context(prog, parent);
    }
    let Some(m) = decl.owner else { return decl.name.clone() };
    let machine = prog.machine(m);
    for s in &machine.states {
        let st = prog.state(*s);
        if st.entry == Some(f) {
            return format!("{}.{} entry", machine.name, st.name);
        }
        if st.exit == Some(f) {
            return format!("{}.{} exit", machine.name, st.name);
        }
        for h in &st.handlers {
            if matches!(h.action, HandlerAction::Do(g) | HandlerAction::Goto { with: Some(g), .. } if g == f) {
                return format!("{}.{}, on {}", machine.name, st.name, prog.event(h.event).name);
            }
        }
    }
    if decl.name.is_empty() {
        machine.name.clone()
    } else {
        format!("{}.{}", machine.name, decl.name)
    }
}

/// `if (cond)` of the `if` statement at `span`, shortened.
fn condition(sm: &SourceMap, span: Span) -> String {
    if span.is_dummy() {
        return String::new();
    }
    let text = sm.text(span);
    let mut depth = 0;
    for (i, c) in text.char_indices() {
        match c {
            '(' => depth += 1,
            ')' if depth == 1 => return shorten(&text[..=i].split_whitespace().collect::<Vec<_>>().join(" ")),
            ')' => depth -= 1,
            _ => {}
        }
    }
    snippet(sm, span)
}

fn shorten(line: &str) -> String {
    const MAX: usize = 60;
    if line.chars().count() > MAX {
        format!("{}...", line.chars().take(MAX - 3).collect::<String>())
    } else {
        line.to_string()
    }
}

/// The first line of the source of `span`, shortened.
fn snippet(sm: &SourceMap, span: Span) -> String {
    if span.is_dummy() {
        return String::new();
    }
    shorten(sm.text(span).lines().next().unwrap_or("").trim().trim_end_matches('{').trim_end())
}

fn push(out: &mut Vec<Uncovered>, sm: &SourceMap, span: Span, last: Option<Span>, context: String, what: String) {
    if span.is_dummy() {
        return;
    }
    let loc = sm.location(span);
    let line = loc.line.max(0) as u32;
    let end_line = last.filter(|l| !l.is_dummy()).map_or(line, |l| sm.location(l).line.max(0) as u32);
    out.push(Uncovered { file: loc.file.unwrap_or_default(), line, end_line: end_line.max(line), context, what });
}
