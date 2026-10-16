//! The testing engine: runs a test case for a number of schedules under a strategy and
//! collects the statistics, bug reports and traces (`TestingEngine` + `TestReport`).

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

use pnext_sema::{Program, TestCase};

use crate::compile::{compile_program, Code};
use crate::random::RandomValueGenerator;
use crate::runtime::{Config, Runtime};
use crate::strategy::{
    ComboStrategy, PctStrategy, ProbabilisticRandomStrategy, RandomStrategy, ReplayStrategy, ScheduleTrace, SharedRandom,
    Strategy,
};

/// Options of a checking run (the subset of `CheckerConfiguration` that affects results).
#[derive(Debug, Clone)]
pub struct CheckerOptions {
    /// `random`, `pct`, `fairpct`, `probabilistic` or `replay`.
    pub strategy: String,
    /// Seed of the random value generator (`--seed`). Must be given: the core crate has no
    /// source of entropy.
    pub seed: u32,
    /// Number of schedules to explore (`--schedules`); 0 = unbounded.
    pub iterations: usize,
    pub max_unfair_steps: usize,
    pub max_fair_steps: usize,
    /// Whether the user set `--max-steps` explicitly (affects the report text).
    pub user_set_max_fair_steps: bool,
    /// `None` = the upstream default, half of the fair step bound.
    pub liveness_temperature_threshold: Option<usize>,
    pub fail_on_max_steps: bool,
    /// Priority switch points for `pct`/`fairpct`, coin flips for `probabilistic`.
    pub strategy_bound: usize,
    /// Keep exploring after the first bug (`--explore`).
    pub full_exploration: bool,
    pub incremental_seed: bool,
    /// The schedule to replay when `strategy == "replay"`.
    pub replay: Option<ScheduleTrace>,
}

impl Default for CheckerOptions {
    fn default() -> Self {
        CheckerOptions {
            strategy: "random".into(),
            seed: 0,
            iterations: 1,
            max_unfair_steps: 10000,
            max_fair_steps: 100000,
            user_set_max_fair_steps: false,
            liveness_temperature_threshold: None,
            fail_on_max_steps: false,
            strategy_bound: 0,
            full_exploration: false,
            incremental_seed: false,
            replay: None,
        }
    }
}

impl CheckerOptions {
    pub fn effective_liveness_threshold(&self) -> usize {
        self.liveness_temperature_threshold.unwrap_or(self.max_fair_steps / 2)
    }
}

/// Accumulated statistics of a run (`TestReport`).
#[derive(Debug, Clone, Default)]
pub struct TestReport {
    pub num_bugs: usize,
    pub bug_reports: Vec<String>,
    pub explored_fair_schedules: usize,
    pub explored_unfair_schedules: usize,
    pub total_fair_steps: usize,
    pub min_fair_steps: Option<usize>,
    pub max_fair_steps: Option<usize>,
    pub max_fair_steps_hit_in_fair: usize,
    pub max_unfair_steps_hit_in_fair: usize,
    pub max_unfair_steps_hit_in_unfair: usize,
    pub timelines: BTreeSet<String>,
}

impl TestReport {
    /// `TestReport.GetText`.
    pub fn text(&self, opts: &CheckerOptions, prefix: &str) -> String {
        let sub = if prefix == "..." { "....." } else { prefix };
        let mut out = String::new();
        out.push_str(&format!("{} Checking statistics:", prefix));
        out.push_str(&format!("\n{} Found {} bug{}.", sub, self.num_bugs, plural(self.num_bugs)));
        out.push_str(&format!("\n{} Scheduling statistics:", prefix));
        let total = self.explored_fair_schedules + self.explored_unfair_schedules;
        out.push_str(&format!("\n{} Explored {} schedule{}", sub, total, plural(total)));
        let tl = self.timelines.len();
        out.push_str(&format!("\n{} Explored {} timeline{}", sub, tl, plural(tl)));
        if total > 0 && self.num_bugs > 0 {
            out.push_str(&format!("\n{} Found {} buggy schedules.", sub, percent(self.num_bugs as f64 * 100.0 / total as f64)));
        }
        if self.explored_fair_schedules > 0 {
            let avg = self.total_fair_steps / self.explored_fair_schedules;
            out.push_str(&format!(
                "\n{} Number of scheduling points in terminating schedules: {} (min), {} (avg), {} (max).",
                sub,
                self.min_fair_steps.unwrap_or(0),
                avg,
                self.max_fair_steps.unwrap_or(0)
            ));
            if opts.max_unfair_steps > 0 && self.max_unfair_steps_hit_in_fair > 0 {
                out.push_str(&format!(
                    "\n{} Exceeded the max-steps bound of '{}' in {} of the fair schedules.",
                    sub,
                    opts.max_unfair_steps,
                    percent(self.max_unfair_steps_hit_in_fair as f64 / self.explored_fair_schedules as f64 * 100.0)
                ));
            }
            if opts.user_set_max_fair_steps && opts.max_fair_steps > 0 && self.max_fair_steps_hit_in_fair > 0 {
                out.push_str(&format!(
                    "\n{} Hit the max-steps bound of '{}' in {} of schedules.",
                    sub,
                    opts.max_fair_steps,
                    percent(self.max_fair_steps_hit_in_fair as f64 / self.explored_fair_schedules as f64 * 100.0)
                ));
            }
        }
        if self.explored_unfair_schedules > 0 && opts.max_unfair_steps > 0 && self.max_unfair_steps_hit_in_unfair > 0 {
            out.push_str(&format!(
                "\n{} Hit the max-steps bound of '{}' in {} of the unfair schedules.",
                sub,
                opts.max_unfair_steps,
                percent(self.max_unfair_steps_hit_in_unfair as f64 / self.explored_unfair_schedules as f64 * 100.0)
            ));
        }
        out
    }
}

fn plural(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}

/// `{0:F2}%`.
fn percent(v: f64) -> String {
    format!("{:.2}%", v)
}

/// Receives the console output of a run as it is produced.
pub trait Output {
    /// A normal line (`Logger.WriteLine`).
    fn line(&mut self, s: &str);
    /// A bug report (`ErrorReporter.WriteErrorLine`).
    fn error(&mut self, s: &str);
}

impl Output for Vec<String> {
    fn line(&mut self, s: &str) {
        self.push(s.to_string());
    }
    fn error(&mut self, s: &str) {
        self.push(format!("Error: {}", s));
    }
}

/// What a checking run produced.
#[derive(Debug, Clone, Default)]
pub struct CheckResult {
    pub report: TestReport,
    /// The runtime log of the first buggy schedule plus the report (`ReadableTrace`).
    pub readable_trace: Option<String>,
    /// The reproducible schedule of the first buggy schedule (`.schedule` file).
    pub schedule_trace: Option<String>,
    /// The runtime log of the last schedule (useful when no bug was found).
    pub last_log: Vec<String>,
    pub strategy_description: String,
    /// Set when a replay could not follow its trace.
    pub replay_error: Option<String>,
}

fn build_strategy(opts: &CheckerOptions, rng: &SharedRandom) -> Result<Box<dyn Strategy>, String> {
    Ok(match opts.strategy.as_str() {
        "random" => Box::new(RandomStrategy::new(opts.max_fair_steps, rng.clone())),
        "pct" => Box::new(PctStrategy::new(opts.max_unfair_steps, opts.strategy_bound, rng.clone())),
        "fairpct" => Box::new(ComboStrategy {
            prefix: Box::new(PctStrategy::new(opts.max_unfair_steps, opts.strategy_bound, rng.clone())),
            suffix: Box::new(RandomStrategy::new(opts.max_fair_steps, rng.clone())),
        }),
        "probabilistic" => {
            Box::new(ProbabilisticRandomStrategy::new(opts.max_fair_steps, opts.strategy_bound, rng.clone()))
        }
        "replay" => match &opts.replay {
            Some(t) => Box::new(ReplayStrategy::new(t.clone())),
            None => return Err("replay strategy needs a schedule".into()),
        },
        other => return Err(format!("unknown scheduling strategy '{}'", other)),
    })
}

/// `ShouldPrintIteration`.
struct PrintGuard(usize);

impl PrintGuard {
    fn should_print(&mut self, schedule: usize) -> bool {
        if schedule > self.0 * 10 {
            let digits = schedule.to_string().len() - 1;
            self.0 = 10usize.pow(digits as u32);
        }
        schedule % self.0 == 0
    }
}

/// A compiled program ready to be checked.
pub struct Checker<'p> {
    pub prog: &'p Program,
    pub code: Code,
}

impl<'p> Checker<'p> {
    pub fn new(prog: &'p Program) -> Checker<'p> {
        Checker { prog, code: compile_program(prog) }
    }

    /// Runs one test case (`TestingEngine.Run`).
    pub fn check(&self, tc: &TestCase, opts: &CheckerOptions, out: &mut dyn Output) -> Result<CheckResult, String> {
        let prog = self.prog;
        let (module, main) = prog.test_case_module(tc);
        let Some(main) = main else {
            return Err(format!("test case '{}' has no main machine", tc.name));
        };
        let assignment = prog.test_case_assignment(tc);
        let rng: SharedRandom = Rc::new(RefCell::new(RandomValueGenerator::new(opts.seed)));
        let mut strategy = build_strategy(opts, &rng)?;
        let is_replay = opts.strategy == "replay";
        let seed_opt = match opts.strategy.as_str() {
            "random" | "pct" | "fairpct" | "probabilistic" => format!(" (seed:{})", opts.seed),
            _ => String::new(),
        };
        out.line(&format!("... Checker is using '{}' strategy{}.", opts.strategy, seed_opt));

        let config = Config {
            max_unfair_steps: opts.max_unfair_steps,
            max_fair_steps: opts.max_fair_steps,
            liveness_enabled: true,
            liveness_temperature_threshold: opts.effective_liveness_threshold(),
            fail_on_max_steps: opts.fail_on_max_steps,
            strategy_name: opts.strategy.clone(),
        };

        let mut result = CheckResult { strategy_description: strategy.description(), ..Default::default() };
        let mut guard = PrintGuard(1);
        let max_iterations = if is_replay { 1 } else { opts.iterations };
        let mut i = 0usize;
        while max_iterations == 0 || i < max_iterations {
            if !is_replay && guard.should_print(i + 1) {
                out.line(&format!("..... Schedule #{}", i + 1));
            }
            let bug_found;
            {
                let mut rt = Runtime::new(prog, &self.code, module, &config, &mut *strategy, assignment);
                rt.run_test(main, &tc.name);
                if rt.bug.is_none() {
                    rt.check_hot_monitors_at_termination();
                }
                bug_found = rt.bug.is_some();
                if let Some(b) = &rt.bug {
                    out.error(b);
                }
                // GatherTestingStatistics
                let steps = rt.strategy.scheduled_steps();
                let reached_max = rt.strategy.has_reached_max_steps();
                let fair = rt.strategy.is_fair();
                let r = &mut result.report;
                if let Some(b) = &rt.bug {
                    r.num_bugs += 1;
                    r.bug_reports.push(b.clone());
                }
                if fair {
                    r.explored_fair_schedules += 1;
                    r.total_fair_steps += steps;
                    r.min_fair_steps = Some(r.min_fair_steps.map_or(steps, |m| m.min(steps)));
                    r.max_fair_steps = Some(r.max_fair_steps.map_or(steps, |m| m.max(steps)));
                    if reached_max {
                        r.max_fair_steps_hit_in_fair += 1;
                    }
                    if opts.max_unfair_steps > 0 && steps >= opts.max_unfair_steps {
                        r.max_unfair_steps_hit_in_fair += 1;
                    }
                } else {
                    r.explored_unfair_schedules += 1;
                    if reached_max {
                        r.max_unfair_steps_hit_in_unfair += 1;
                    }
                }
                r.timelines.insert(rt.timeline.abstract_timeline());
                if let Some(e) = rt.strategy.replay_error() {
                    result.replay_error = Some(e);
                }
                if bug_found && !is_replay && result.readable_trace.is_none() {
                    let mut text = rt.log.join("\n");
                    text.push('\n');
                    text.push_str(&result.report.text(opts, "<StrategyLog>"));
                    result.readable_trace = Some(text);
                    let mut trace = rt.trace.clone();
                    trace.is_fair = fair;
                    trace.liveness_temperature_threshold = Some(config.liveness_temperature_threshold);
                    trace.test_name = Some(tc.name.clone());
                    result.schedule_trace = Some(trace.to_text());
                }
                if bug_found && !is_replay && opts.full_exploration {
                    out.line(&format!("..... Schedule #{} triggered bug #{} [task-0]", i + 1, result.report.num_bugs));
                }
                result.last_log = std::mem::take(&mut rt.log);
            }
            if is_replay || (!opts.full_exploration && result.report.num_bugs > 0) || !strategy.prepare_next_iteration() {
                break;
            }
            if opts.incremental_seed {
                let s = rng.borrow().seed.wrapping_add(1);
                rng.borrow_mut().set_seed(s);
            }
            i += 1;
        }
        Ok(result)
    }
}
