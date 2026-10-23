//! Mutation testing for `snappea check`: runs the passing test cases against mutants of
//! the model (see [`crate::checker::mutation`]) and reports the mutants no test case
//! detects. Each mutant runs only the test cases that ran the code it changes, for a
//! fixed number of schedules with a fixed seed, so a mutant's verdict does not depend on
//! the machine; the time limit only decides how many mutants are tested. Mutants are
//! tested in a fixed, shuffled order, so a run the time limit stops samples all the code.

use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::time::Duration;

use serde_json::{json, Value};

use crate::checker::code_coverage::Hits;
use crate::checker::engine::{Checker, CheckerOptions, Output};
use crate::checker::mutation::{self, Mutant};
use crate::checker::strategy::StrategyKind;
use crate::driver::check::{engine_options, CheckConfig};
use crate::host::{Host, Job};
use crate::sema::{Program, TestCase};

/// Schedules explored per mutant, shared by the test cases that run its code.
pub const SCHEDULES: usize = 1000;
/// Schedules each of those test cases explores at least.
pub const MIN_SCHEDULES: usize = 100;

/// What the test cases found for one mutant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// A test case failed with the change.
    Detected { test: String },
    /// Every test case that ran the changed code passed.
    Undetected {
        tests: Vec<String>,
        /// Every test case explored all of its schedules.
        search_complete: bool,
    },
    /// The time limit came first.
    NotTested,
}

/// The mutants of a suite and their verdicts.
#[derive(Debug, Clone, Default)]
pub struct Report {
    pub mutants: Vec<Mutant>,
    pub verdicts: Vec<Verdict>,
    pub workers: usize,
    pub seconds: f64,
}

impl Report {
    fn count(&self, f: impl Fn(&Verdict) -> bool) -> usize {
        self.verdicts.iter().filter(|v| f(v)).count()
    }

    pub fn detected(&self) -> usize {
        self.count(|v| matches!(v, Verdict::Detected { .. }))
    }

    pub fn not_tested(&self) -> usize {
        self.count(|v| *v == Verdict::NotTested)
    }

    /// The mutants no test case detected, by file and line.
    pub fn undetected(&self) -> Vec<(&Mutant, &[String], bool)> {
        let mut out: Vec<_> = self
            .mutants
            .iter()
            .zip(&self.verdicts)
            .filter_map(|(m, v)| match v {
                Verdict::Undetected { tests, search_complete } => Some((m, tests.as_slice(), *search_complete)),
                _ => None,
            })
            .collect();
        out.sort_by(|a, b| (&a.0.file, a.0.line).cmp(&(&b.0.file, b.0.line)));
        out
    }

    /// The console lines: the counts, then the first `shown` undetected mutants.
    pub fn lines(&self, shown: usize) -> Vec<String> {
        let tested = self.mutants.len() - self.not_tested();
        let mut out = vec![format!(
            "Mutation testing: {} of {} mutant{} detected ({:.1}s)",
            self.detected(),
            tested,
            plural(tested),
            self.seconds
        )];
        let undetected = self.undetected();
        if !undetected.is_empty() {
            out.push("Not detected by any test case (every test case still passes with the change):".into());
            let location = |m: &Mutant| format!("{}:{}", m.file, m.line);
            let list = &undetected[..undetected.len().min(shown)];
            let width = list.iter().map(|(m, ..)| location(m).len()).max().unwrap_or(0);
            for (m, ..) in list {
                out.push(format!("  {:width$}  {}: {}", location(m), m.context, m.description, width = width));
            }
            if undetected.len() > list.len() {
                out.push(format!("  ... {} more (--json lists all)", undetected.len() - list.len()));
            }
        }
        let n = self.not_tested();
        if n > 0 {
            out.push(format!(
                "Mutation testing reached its time limit: {} mutant{} not tested (--mutation-timeout raises it).",
                n,
                plural(n)
            ));
        }
        out
    }

    pub fn to_json(&self) -> Value {
        json!({
            "mutants": self.mutants.len() - self.not_tested(),
            "detected": self.detected(),
            "undetected": self.undetected().iter().map(|(m, tests, complete)| json!({
                "file": m.file,
                "line": m.line,
                "context": m.context,
                "mutation": m.change.name(),
                "change": m.description,
                "test_cases": tests,
                "search_complete": complete,
            })).collect::<Vec<_>>(),
            "not_tested": self.not_tested(),
            "time_limit_reached": self.not_tested() > 0,
            "schedules_per_mutant": SCHEDULES,
            "seconds": (self.seconds * 1000.0).round() / 1000.0,
        })
    }
}

/// Runs the passing test cases `passed` (each with the code it ran) against the mutants
/// of the code they ran, for at most `time_limit` seconds.
pub fn run(
    host: &mut dyn Host,
    prog: &Program,
    cfg: &CheckConfig,
    passed: &[(TestCase, Hits)],
    time_limit: f64,
) -> Report {
    let start = host.now();
    let tests: Vec<TestCase> = passed.iter().map(|(tc, _)| tc.clone()).collect();
    let probes = crate::checker::compile::compile_program_with_probes(prog).probes;
    let funs = mutation::mutable_funs(prog, &tests);
    // only mutants of code a passing test case ran: coverage reports the rest
    let mutants: Vec<Mutant> = mutation::mutants(prog, &funs, &probes)
        .into_iter()
        .filter(|m| passed.iter().any(|(_, h)| h.ran(m.probe)))
        .collect();
    // the order mutants are tested in: shuffled, the same in every run
    let mut order: Vec<usize> = (0..mutants.len()).collect();
    order.sort_by_cached_key(|&i| {
        let m = &mutants[i];
        let mut h = rustc_hash::FxHasher::default();
        (&m.file, m.line, &m.description).hash(&mut h);
        (h.finish(), i)
    });
    let opts = CheckerOptions {
        strategy: StrategyKind::Auto,
        iterations: SCHEDULES,
        json_trace: false,
        xml_trace: false,
        activity_coverage: false,
        verbose: false,
        full_exploration: false,
        ..engine_options(cfg, cfg.seed.unwrap_or(0))
    };
    let job = Mutants { prog, mutants: &mutants, order: &order, passed, opts: &opts };
    let deadline = start + time_limit;
    let mut verdicts = vec![Verdict::NotTested; mutants.len()];
    let threads = host.threads();
    let workers = threads.map_or(1, |t| cfg.workers.unwrap_or(t.available)).clamp(1, mutants.len().max(1));
    match threads {
        Some(threads) if workers > 1 => job.run_threads(host, threads, workers, deadline, &mut verdicts),
        _ => {
            let mut out = Deadline { host, deadline };
            for &i in &order {
                match job.test(i, &mut out) {
                    Some(verdict) => verdicts[i] = verdict,
                    None => break,
                }
            }
        }
    }
    Report { mutants, verdicts, workers, seconds: host.now() - start }
}

struct Mutants<'a> {
    prog: &'a Program,
    mutants: &'a [Mutant],
    order: &'a [usize],
    passed: &'a [(TestCase, Hits)],
    opts: &'a CheckerOptions,
}

impl Mutants<'_> {
    /// Runs the test cases that ran mutant `i`'s code against it, until one fails; `None`
    /// when `out` was cancelled first.
    fn test(&self, i: usize, out: &mut dyn Output) -> Option<Verdict> {
        let m = &self.mutants[i];
        let prog = mutation::apply(self.prog, m);
        let checker = Checker::new(&prog);
        let mut tests = Vec::new();
        let mut complete = true;
        let covering: Vec<_> = self.passed.iter().filter(|(_, h)| h.ran(m.probe)).collect();
        // the test cases share the schedules: code many test cases run costs no more to
        // test than code one runs
        let iterations = (SCHEDULES / covering.len().max(1)).max(MIN_SCHEDULES);
        let opts = CheckerOptions { iterations, ..self.opts.clone() };
        for (tc, _) in covering {
            if out.cancelled() {
                return None;
            }
            let res = match checker.check(tc, &opts, out) {
                Ok(res) => res,
                // a change the checker cannot run is one a test case notices
                Err(_) => return Some(Verdict::Detected { test: tc.name.clone() }),
            };
            if res.report.num_bugs > 0 {
                return Some(Verdict::Detected { test: tc.name.clone() });
            }
            if res.cancelled {
                return None;
            }
            complete &= res.exhaustive.as_ref().is_some_and(|x| x.finished);
            tests.push(tc.name.clone());
        }
        Some(Verdict::Undetected { tests, search_complete: complete })
    }

    /// Tests the mutants on `workers` threads, until `deadline`.
    fn run_threads(
        &self,
        host: &mut dyn Host,
        threads: crate::host::Threads,
        workers: usize,
        deadline: f64,
        verdicts: &mut [Verdict],
    ) {
        let next = AtomicUsize::new(0);
        let cancelled = AtomicBool::new(false);
        let (tx, rx) = mpsc::channel::<(usize, Verdict)>();
        let jobs: Vec<Job<'_>> = (0..workers)
            .map(|_| {
                let (tx, next, cancelled) = (tx.clone(), &next, &cancelled);
                Box::new(move || {
                    let mut out = Flag(cancelled);
                    loop {
                        let k = next.fetch_add(1, Ordering::Relaxed);
                        if k >= self.order.len() || cancelled.load(Ordering::Relaxed) {
                            break;
                        }
                        let i = self.order[k];
                        match self.test(i, &mut out) {
                            Some(v) => {
                                let _ = tx.send((i, v));
                            }
                            None => break,
                        }
                    }
                }) as Job<'_>
            })
            .collect();
        drop(tx);
        let mut main = || loop {
            match rx.recv_timeout(Duration::from_millis(20)) {
                Ok((i, v)) => verdicts[i] = v,
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
            if host.now() >= deadline {
                cancelled.store(true, Ordering::Relaxed);
            }
        };
        (threads.run)(jobs, &mut main);
    }
}

/// Engine output that is discarded, cancelled by a flag.
struct Flag<'a>(&'a AtomicBool);

impl Output for Flag<'_> {
    fn line(&mut self, _: &str) {}
    fn error(&mut self, _: &str) {}
    fn cancelled(&mut self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

/// Engine output that is discarded, cancelled at a deadline.
struct Deadline<'h> {
    host: &'h dyn Host,
    deadline: f64,
}

impl Output for Deadline<'_> {
    fn line(&mut self, _: &str) {}
    fn error(&mut self, _: &str) {}
    fn cancelled(&mut self) -> bool {
        self.host.now() >= self.deadline
    }
}

fn plural(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}
