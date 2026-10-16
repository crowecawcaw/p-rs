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
    /// See [`Config::max_work_per_step`](crate::runtime::Config::max_work_per_step).
    pub max_work_per_step: u64,
    /// Write `--test-method:<name>` into the reproducible trace (upstream does so only when
    /// a test case was named with `--testcase`).
    pub record_test_name: bool,
    /// Stream every schedule's runtime log to the output instead of keeping it in memory
    /// (`--verbose`); no readable trace is produced then.
    pub verbose: bool,
    /// Keep the runtime log of every schedule (`CheckResult::last_log` is then the last
    /// schedule's log). Off by default: exploring without logging is much faster, and the
    /// first buggy schedule is replayed to produce its log.
    pub log_every_schedule: bool,
    /// Collect activity coverage (`CheckResult::coverage`), as upstream does by default.
    pub activity_coverage: bool,
    /// Produce the JSON trace of the first buggy schedule (by replaying it with JSON
    /// logging on, so exploration itself pays nothing).
    pub json_trace: bool,
    /// Produce the XML trace of the first buggy schedule (`--xml-trace`), also by replaying it.
    pub xml_trace: bool,
    /// Name of the compiled project, the source upstream reports for exceptions raised by
    /// the model's own code.
    pub assembly_name: String,
    /// Latency profiles (`--latency`); see `design/time/PROPOSAL.md`.
    pub latency: Option<Rc<crate::time::ProfileSet>>,
    /// `--latency-profile`: the profile to use instead of the `[tests]` entry.
    pub latency_profile: Option<String>,
    /// `--timed`: time-aware scheduling.
    pub timed: bool,
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
            max_work_per_step: crate::runtime::DEFAULT_MAX_WORK_PER_STEP,
            record_test_name: true,
            verbose: false,
            log_every_schedule: false,
            activity_coverage: false,
            json_trace: false,
            xml_trace: false,
            assembly_name: "PImplementation".into(),
            latency: None,
            latency_profile: None,
            timed: false,
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
    /// Scenario coverage, per scenario (in attachment order).
    pub scenarios: Vec<ScenarioCoverage>,
}

/// Coverage of one scenario monitor across a run (`TestReport.Scenario*`).
#[derive(Debug, Clone, Default)]
pub struct ScenarioCoverage {
    pub name: String,
    /// Schedules that satisfied the scenario.
    pub trigger_count: usize,
    /// Distinct abstract timelines of the satisfying schedules.
    pub satisfying_timelines: BTreeSet<String>,
    /// Most distinct states reached in one schedule.
    pub max_states_reached: usize,
    pub total_states: usize,
}

/// String ordering of .NET's default comparer (`OrderBy(k => k)`, `List<string>.Sort()`) as
/// the P tool runs it: with `InvariantGlobalization`, comparisons are ordinal.
pub fn culture_order(a: &str, b: &str) -> std::cmp::Ordering {
    a.cmp(b)
}

impl TestReport {
    fn scenario(&mut self, name: &str) -> &mut ScenarioCoverage {
        if let Some(i) = self.scenarios.iter().position(|s| s.name == name) {
            return &mut self.scenarios[i];
        }
        self.scenarios.push(ScenarioCoverage { name: name.to_string(), ..Default::default() });
        self.scenarios.last_mut().unwrap()
    }

    /// Scenarios sorted the way upstream lists them.
    pub fn sorted_scenarios(&self) -> Vec<&ScenarioCoverage> {
        let mut v: Vec<&ScenarioCoverage> = self.scenarios.iter().collect();
        v.sort_by(|a, b| culture_order(&a.name, &b.name));
        v
    }
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
        if !self.scenarios.is_empty() {
            let names = self.sorted_scenarios();
            let covered: Vec<&&ScenarioCoverage> = names.iter().filter(|s| s.trigger_count > 0).collect();
            let gaps: Vec<&&ScenarioCoverage> = names.iter().filter(|s| s.trigger_count == 0).collect();
            let gap_text = if gaps.is_empty() {
                String::new()
            } else {
                format!(" ({} gap{})", gaps.len(), plural(gaps.len()))
            };
            out.push_str(&format!(
                "\n{} Scenario coverage: {}/{} scenarios covered{}",
                sub,
                covered.len(),
                names.len(),
                gap_text
            ));
            for s in covered {
                let u = s.satisfying_timelines.len();
                out.push_str(&format!(
                    "\n{}   [covered] {}: triggered in {} schedule{}, {} unique satisfying timeline{}",
                    sub,
                    s.name,
                    s.trigger_count,
                    plural(s.trigger_count),
                    u,
                    plural(u)
                ));
            }
            for s in gaps {
                out.push_str(&format!("\n{}   [  GAP  ] {}: not covered", sub, s.name));
                if s.total_states > 0 {
                    out.push_str(&format!(" (best partial progress: {}/{} states)", s.max_states_reached, s.total_states));
                }
            }
        }
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
    /// Polled before every schedule; returning true stops exploration (timeouts).
    fn cancelled(&mut self) -> bool {
        false
    }
}

impl Output for Vec<String> {
    fn line(&mut self, s: &str) {
        self.push(s.to_string());
    }
    fn error(&mut self, s: &str) {
        self.push(format!("Error: {}", s));
    }
}

/// The first buggy schedule, kept for the logged replay.
struct BugSchedule {
    trace: ScheduleTrace,
    bound_hit_at: Option<usize>,
    strategy_description: Option<String>,
    bug: Option<String>,
    report_text: String,
    /// `Task.CurrentId` of the buggy schedule's test task.
    test_task_id: u64,
}

/// What a checking run produced.
#[derive(Debug, Clone, Default)]
pub struct CheckResult {
    pub report: TestReport,
    /// The runtime log of the first buggy schedule plus the report (`ReadableTrace`).
    pub readable_trace: Option<String>,
    /// The reproducible schedule of the first buggy schedule (`.schedule` file).
    pub schedule_trace: Option<String>,
    /// The runtime log of the first buggy schedule; with `log_every_schedule` (or
    /// `verbose`), the log of the last schedule explored.
    pub last_log: Vec<String>,
    pub strategy_description: String,
    /// Set when a replay could not follow its trace.
    pub replay_error: Option<String>,
    /// Exploration was stopped by [`Output::cancelled`].
    pub cancelled: bool,
    /// The `.trace.json` contents of the first buggy schedule (see `json_trace`).
    pub json_trace: Option<String>,
    /// The `.trace.xml` contents of the first buggy schedule (see `xml_trace`).
    pub xml_trace: Option<String>,
    /// With `verbose` and `json_trace`: the JSON logs of every schedule explored, as one
    /// array of arrays (upstream's `_verbose.trace.json`).
    pub verbose_json_trace: Option<String>,
    /// Activity coverage over all schedules (see `activity_coverage`).
    pub coverage: Option<crate::coverage::CoverageInfo>,
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
        "pos" => Box::new(crate::feedback::ScheduleAndInputStrategy::new(
            opts.max_unfair_steps,
            rng.clone(),
            Box::new(crate::feedback::PosScheduler::new(crate::feedback::Source::Shared(rng.clone()))),
        )),
        "feedback" | "feedbackpct" | "feedbackpos" => {
            use crate::feedback::{ControlledRandom, PctScheduler, PosScheduler, RandomScheduler, Source};
            let src = Source::Controlled(ControlledRandom::new(opts.seed));
            let scheduler: Box<dyn crate::feedback::Scheduler> = match opts.strategy.as_str() {
                "feedback" => Box::new(RandomScheduler::new(src)),
                "feedbackpct" => Box::new(PctScheduler::new(opts.strategy_bound, 0, src)),
                _ => Box::new(PosScheduler::new(src)),
            };
            Box::new(crate::feedback::FeedbackGuidedStrategy::new(opts.max_unfair_steps, opts.seed, scheduler))
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
    /// Implementations of the program's foreign functions.
    pub foreign: Option<Rc<dyn crate::foreign::ForeignFunctions>>,
    coverage_tables: std::cell::OnceCell<Rc<crate::coverage::CoverageTables>>,
}

impl<'p> Checker<'p> {
    pub fn new(prog: &'p Program) -> Checker<'p> {
        Checker { prog, code: compile_program(prog), foreign: None, coverage_tables: std::cell::OnceCell::new() }
    }

    /// Node/event tables for activity coverage (built on first use).
    pub fn coverage_tables(&self) -> Rc<crate::coverage::CoverageTables> {
        self.coverage_tables
            .get_or_init(|| Rc::new(crate::coverage::CoverageTables::new(self.prog)))
            .clone()
    }

    /// The `.coverage.txt` report of a run's activity coverage.
    pub fn coverage_report(&self, info: &crate::coverage::CoverageInfo) -> String {
        crate::coverage::coverage_report(self.prog, &self.coverage_tables(), info)
    }

    /// Registers implementations for the program's foreign functions.
    pub fn with_foreign(mut self, f: Rc<dyn crate::foreign::ForeignFunctions>) -> Checker<'p> {
        self.foreign = Some(f);
        self
    }

    /// Time tracking for `tc`: active when the program declares one of the time
    /// functions or a latency file or `--timed` is given. Checks the profile's names.
    /// Whether the program declares time functions; errors on a wrong declaration.
    fn declares_time(&self) -> Result<bool, String> {
        use pnext_sema::PType;
        let prog = self.prog;
        let mut declares = false;
        for f in prog.funs.iter().filter(|f| f.owner.is_none() && f.is_foreign()) {
            let Some(sig) = crate::time::time_signature(&f.name) else { continue };
            declares = true;
            let params: Vec<&PType> = f.params.iter().map(|v| &prog.var(*v).ty).collect();
            let want = |t: &str| match t {
                "int" => PType::Int,
                "string" => PType::String,
                _ => PType::Null,
            };
            let (want_params, want_ret) = sig;
            let ok = params.len() == want_params.len()
                && params.iter().zip(want_params).all(|(p, w)| **p == want(w))
                && f.ret == want(want_ret);
            if !ok {
                return Err(format!(
                    "foreign function '{}' is a time function and must be declared as {}",
                    f.name,
                    crate::time::time_declaration(&f.name)
                ));
            }
        }
        Ok(declares)
    }

    fn time_options(&self, tc: &TestCase, opts: &CheckerOptions) -> Result<Option<crate::time::TimeOptions>, String> {
        let declares = self.declares_time()?;
        if !declares && opts.latency.is_none() && !opts.timed {
            return Ok(None);
        }
        let profile = match &opts.latency {
            Some(set) => match set.for_test(&tc.name, opts.latency_profile.as_deref())? {
                Some(p) => Some(p.clone()),
                None => {
                    return Err(format!(
                        "test case '{}' has no latency profile: add it to [tests] in the latency file or pass --latency-profile",
                        tc.name
                    ))
                }
            },
            None if opts.latency_profile.is_some() => return Err("--latency-profile needs --latency".into()),
            None => None,
        };
        self.check_profile(profile.as_ref())?;
        Ok(Some(crate::time::TimeOptions { profile, timed: opts.timed }))
    }

    fn check_profile(&self, profile: Option<&crate::time::Profile>) -> Result<(), String> {
        let Some(p) = profile else { return Ok(()) };
        let machines: Vec<&str> = self.prog.machines.iter().map(|m| m.name.as_str()).collect();
        let events: Vec<&str> = self.prog.events.iter().map(|e| e.name.as_str()).collect();
        p.check_names(&machines, &events)
    }

    /// Replays a buggy schedule with text (and optionally JSON) logging enabled; returns
    /// the text log and the JSON trace.
    #[allow(clippy::too_many_arguments)]
    fn logged_replay(
        &self,
        tc: &TestCase,
        main: pnext_sema::MachineId,
        module: &pnext_sema::ModuleInfo,
        assignment: Option<&pnext_sema::ParamAssignment>,
        config: &Config,
        bug: &BugSchedule,
        json: bool,
        xml: bool,
        time: Option<&crate::time::TimeOptions>,
    ) -> (Vec<String>, Option<String>, Option<String>) {
        let mut strategy = ReplayStrategy::new(bug.trace.clone());
        strategy.stop_at = bug.bound_hit_at;
        let mut config = config.clone();
        config.strategy_description = bug.strategy_description.clone();
        config.logging = true;
        let mut rt = Runtime::new(self.prog, &self.code, module, &config, &mut strategy, assignment);
        rt.foreign = self.foreign.clone();
        rt.time = time.map(|o| Box::new(crate::time::TimeState::new(o.clone())));
        if json {
            rt.json = Some(crate::jsonlog::JsonLog::new());
        }
        if xml {
            rt.xml = Some(crate::xmllog::XmlLog::new(bug.test_task_id));
        }
        rt.run_test(main, &tc.name);
        if rt.bug.is_none() {
            rt.check_hot_monitors_at_termination();
        }
        debug_assert_eq!(rt.bug, bug.bug, "replaying a buggy schedule must reproduce its bug");
        (std::mem::take(&mut rt.log), rt.json.take().map(|j| j.to_json_text()), rt.xml.take().map(|x| x.finish()))
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
        // a replay takes its time settings from the schedule file
        let time = match opts.replay.as_ref().filter(|_| is_replay).and_then(|t| t.time.as_ref().map(|h| (h, t))) {
            Some((header, t)) => {
                let o = crate::time::TimeOptions::from_schedule(header, &t.latency)
                    .map_err(|e| format!("the schedule's time settings: {}", e))?;
                self.declares_time()?;
                self.check_profile(o.profile.as_ref())?;
                Some(o)
            }
            // a schedule without time settings was recorded untimed, without a profile
            None if is_replay => {
                let plain = CheckerOptions { latency: None, latency_profile: None, timed: false, ..opts.clone() };
                self.time_options(tc, &plain)?
            }
            None => self.time_options(tc, opts)?,
        };
        let seed_opt = match opts.strategy.as_str() {
            "random" | "pct" | "pos" | "feedbackpct" | "feedbackpos" | "fairpct" | "probabilistic" | "rl" => {
                format!(" (seed:{})", opts.seed)
            }
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
            max_work_per_step: opts.max_work_per_step,
            strategy_description: None,
            // buggy schedules are replayed with logging on afterwards
            logging: is_replay || opts.verbose || opts.log_every_schedule,
            assembly_name: opts.assembly_name.clone(),
        };

        let mut result = CheckResult { strategy_description: strategy.description(), ..Default::default() };
        let mut guard = PrintGuard(1);
        let max_iterations = if is_replay { 1 } else { opts.iterations };
        let mut i = 0usize;
        let mut first_bug: Option<BugSchedule> = None;
        // `Task.Id`s are handed out in order: one for the engine, then per schedule one for
        // the test task and one per machine event handler run
        let mut tasks_used = 1u64;
        let coverage_tables = opts.activity_coverage.then(|| self.coverage_tables());
        let mut verbose_json = (opts.verbose && opts.json_trace && !is_replay).then(Vec::new);
        while max_iterations == 0 || i < max_iterations {
            if out.cancelled() {
                result.cancelled = true;
                break;
            }
            if !is_replay && guard.should_print(i + 1) {
                out.line(&format!("..... Schedule #{}", i + 1));
            }
            let bug_found;
            {
                let mut rt = Runtime::new(prog, &self.code, module, &config, &mut *strategy, assignment);
                rt.foreign = self.foreign.clone();
                rt.time = time.as_ref().map(|o| Box::new(crate::time::TimeState::new(o.clone())));
                if let Some(t) = &coverage_tables {
                    rt.coverage_tables = Some(t.clone());
                    rt.coverage = Some(result.coverage.take().unwrap_or_default());
                    rt.coverage_run = Some(Default::default());
                }
                if verbose_json.is_some() {
                    rt.json = Some(crate::jsonlog::JsonLog::new());
                }
                rt.run_test(main, &tc.name);
                if rt.strategy.wants_feedback() {
                    let tokens = rt.timeline.tokens();
                    let timeline = tokens.join(";");
                    let minhash = crate::feedback::timeline_minhash(tokens.iter().map(String::as_str));
                    let reached: Vec<(String, usize)> =
                        rt.scenarios.iter().map(|s| (s.name.clone(), s.reached.len())).collect();
                    let satisfied: Vec<String> =
                        rt.scenarios.iter().filter(|s| s.satisfied).map(|s| s.name.clone()).collect();
                    rt.strategy.observe_timeline(&timeline, &minhash, &reached, &satisfied);
                }
                if rt.bug.is_none() {
                    rt.check_hot_monitors_at_termination();
                }
                bug_found = rt.bug.is_some();
                let test_task_id = tasks_used + 1;
                tasks_used += 1 + rt.handler_starts;
                if let (Some(all), Some(j)) = (verbose_json.as_mut(), rt.json.take()) {
                    all.push(j.to_json(false));
                }
                if opts.verbose {
                    for l in &rt.log {
                        out.line(l);
                    }
                }
                // upstream reports the bug through a console that is redirected to a null
                // writer while a schedule runs, so it only shows in verbose mode
                if let Some(b) = rt.bug.as_ref().filter(|_| opts.verbose) {
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
                let abstract_timeline = rt.timeline.abstract_timeline();
                // scenario coverage (all attached scenarios are tracked, even if never reached)
                for sc in &rt.scenarios {
                    let cov = r.scenario(&sc.name);
                    cov.max_states_reached = cov.max_states_reached.max(sc.reached.len());
                    if sc.total_states > 0 {
                        cov.total_states = sc.total_states;
                    }
                    if sc.satisfied {
                        cov.trigger_count += 1;
                        cov.satisfying_timelines.insert(abstract_timeline.clone());
                    }
                }
                r.timelines.insert(abstract_timeline);
                if let Some(e) = rt.strategy.replay_error() {
                    result.replay_error = Some(e);
                }
                if bug_found && !is_replay && result.schedule_trace.is_none() {
                    let mut trace = rt.trace.clone();
                    trace.is_fair = fair;
                    trace.liveness_temperature_threshold = Some(config.liveness_temperature_threshold);
                    trace.test_name = if opts.record_test_name { Some(tc.name.clone()) } else { None };
                    trace.time = time.as_ref().map(|t| t.schedule_header());
                    trace.latency = time.as_ref().and_then(|t| t.profile.as_ref()).map(|p| p.entries()).unwrap_or_default();
                    result.schedule_trace = Some(trace.to_text());
                    first_bug = Some(BugSchedule {
                        trace,
                        bound_hit_at: rt.bound_hit_at,
                        strategy_description: rt.bug_strategy_description.clone(),
                        bug: rt.bug.clone(),
                        report_text: result.report.text(opts, "<StrategyLog>"),
                        test_task_id,
                    });
                }
                if bug_found && !is_replay && opts.full_exploration {
                    out.line(&format!("..... Schedule #{} triggered bug #{} [task-0]", i + 1, result.report.num_bugs));
                }
                result.last_log = std::mem::take(&mut rt.log);
                if rt.coverage.is_some() {
                    result.coverage = rt.coverage.take();
                }
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
        result.verbose_json_trace = verbose_json.map(|v| crate::jsonlog::Json::Arr(v).to_pretty());
        if let Some(bug) = first_bug {
            let (log, json, xml) =
                self.logged_replay(tc, main, module, assignment, &config, &bug, opts.json_trace, opts.xml_trace, time.as_ref());
            if !opts.verbose {
                // `ReadableTrace`: the runtime log followed by the report so far
                let mut text = log.join("\n");
                text.push('\n');
                text.push_str(&bug.report_text);
                result.readable_trace = Some(text);
            }
            result.json_trace = json;
            result.xml_trace = xml;
            if !config.logging {
                result.last_log = log;
            }
        }
        Ok(result)
    }
}
