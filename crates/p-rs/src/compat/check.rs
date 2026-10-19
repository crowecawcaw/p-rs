//! `p check` (`PCheckerOptions`, `Checker.Run`, `TestingProcess`, `TestingEngine` I/O).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::Duration;

use crate::checker::engine::{CheckResult, Checker, CheckerOptions, Exploration, Output, WorkerPool};
use crate::checker::strategy::{ScheduleTrace, StrategyKind};

use crate::compat::argparse::{ArgParser, DataType, ParseError};
use crate::compat::host::{full_path, Color, Host, HostExt, Job, Stream, Threads};
use crate::compat::project::{self, Sources};
use crate::compat::{format_0_00, report_and_exit, write_output, Exit};

/// `PCheckerOptions` parser definition.
pub fn parser() -> ArgParser {
    let mut p = ArgParser::new(
        "p check",
        "The P checker enables systematic exploration of a specified P test case, it generates a reproducible bug-trace if a bug is found, and also allows replaying a bug-trace.",
    );
    let b = Some("Basic");
    p.group("Basic", "Basic options");
    p.add_positional(
        b,
        "path",
        "P project file (*.pproj) or P file (*.p) to check. If no sources are given, the checker uses the *.pproj in the current folder, or else the *.p files in it and its immediate subfolders",
    )
    .optional();
    p.add(b, "pproj", Some("pp"), "P project file to check (*.pproj)", DataType::Str);
    p.add(b, "pfiles", Some("pf"), "List of P files to check", DataType::Str).multi();
    p.add(b, "projname", Some("pn"), "Project name (names the output files)", DataType::Str);
    p.add(
        b,
        "mode",
        Some("md"),
        "Checker mode to use. Can be bugfinding or pex. If this option is not passed, bugfinding mode is used as default",
        DataType::Str,
    )
    .allowed(&["bugfinding", "pex"]);
    p.add(b, "testcase", Some("tc"), "Test case to explore", DataType::Str);
    p.add(b, "list-tests", None, "List all test cases and exit.", DataType::Bool);
    p.add(b, "timeout", Some("t"), "Timeout in seconds (disabled by default)", DataType::UInt);
    p.add(b, "memout", Some("m"), "Memory limit in Giga bytes (disabled by default)", DataType::Double);
    p.add(b, "outdir", Some("o"), "Dump output to directory (absolute or relative path)", DataType::Str);
    p.add(b, "verbose", Some("v"), "Enable verbose log output during exploration", DataType::Bool);
    p.add(b, "debug", Some("d"), "Enable debugging", DataType::Bool).hidden();

    let e = Some("explore");
    p.group("explore", "Systematic exploration options");
    p.add(e, "iterations", Some("i"), "Number of schedules to explore", DataType::UInt).hidden();
    p.add(e, "schedules", Some("s"), "Number of schedules to explore", DataType::UInt);
    p.add(
        e,
        "max-steps",
        Some("ms"),
        "Max scheduling steps to be explored during systematic exploration (by default 10,000 unfair and 100,000 fair steps). You can provide one or two unsigned integer values",
        DataType::UInt,
    )
    .multi();
    p.add(e, "fail-on-maxsteps", None, "Consider it a bug if the test hits the specified max-steps", DataType::Bool);
    p.add(
        e,
        "liveness-temperature-threshold",
        None,
        "Specify the liveness temperature threshold is the liveness temperature value that triggers a liveness bug",
        DataType::UInt,
    )
    .hidden();

    let s = Some("scheduling");
    p.group("scheduling", "Search prioritization options");
    p.add(s, "sch-random", None, "Choose the random scheduling strategy (this is the default)", DataType::Bool);
    p.add(s, "sch-feedback", None, "Choose the random scheduling strategy with feedback mutation", DataType::Bool);
    p.add(
        s,
        "sch-feedbackpct",
        None,
        "Choose the PCT scheduling strategy with feedback mutation (recommended feedback strategy: best rare-behavior coverage; takes the max number of priority switch points, e.g. --sch-feedbackpct 10)",
        DataType::UInt,
    );
    p.add(s, "sch-feedbackpos", None, "Choose the POS scheduling strategy with feedback mutation", DataType::Bool);
    p.add(
        s,
        "sch-probabilistic",
        Some("sp"),
        "Choose the probabilistic scheduling strategy with given probability for each scheduling decision where the probability is specified as the integer N in the equation 0.5 to the power of N.  So for N=1, the probability is 0.5, for N=2 the probability is 0.25, N=3 you get 0.125, etc.",
        DataType::UInt,
    );
    p.add(
        s,
        "sch-pct",
        None,
        "Choose the PCT scheduling strategy with given maximum number of priority switch points",
        DataType::UInt,
    );
    p.add(s, "sch-pos", None, "Choose the POS scheduling strategy", DataType::Bool);
    p.add(
        s,
        "sch-fairpct",
        None,
        "Choose the fair PCT scheduling strategy with given maximum number of priority switch points",
        DataType::UInt,
    );
    p.add(s, "sch-rl", None, "Choose the reinforcement learning (RL) scheduling strategy", DataType::Bool).hidden();
    p.add(
        s,
        "sch-pex",
        None,
        "Choose the scheduling strategy for PEx mode (options: random, dfs). (default: random)",
        DataType::Str,
    )
    .allowed(&["random", "dfs", "astar"]);
    p.add(
        s,
        "timeline-repr",
        None,
        "How the feedback-guided search represents a schedule's timeline (the diversity signal): pairwise (default), kgram, causal, hybrid",
        DataType::Str,
    )
    .allowed(&["pairwise", "kgram", "causal", "hybrid"]);
    p.add(
        s,
        "timeline-kgram",
        None,
        "Gram length for the kgram/hybrid timeline representation (default: 3)",
        DataType::UInt,
    );
    p.add(
        s,
        "timeline-payload",
        None,
        "Enrich timeline event labels with a stable hash of the event payload",
        DataType::Bool,
    );

    p.group("replay", "Replay and debug options");
    p.add(Some("replay"), "replay", Some("r"), "Schedule file to replay", DataType::Str);

    let a = Some("advanced");
    p.group("advanced", "Advanced options");
    p.add(a, "explore", None, "Keep testing until the bound (e.g. schedule or time) is reached", DataType::Bool);
    p.add(a, "seed", None, "Specify the random value generator seed", DataType::UInt);
    p.add(a, "graph-bug", None, "Output a DGML graph of the schedule that found a bug", DataType::Bool);
    p.add(a, "graph", None, "Output a DGML graph of all test schedules whether a bug was found or not", DataType::Bool);
    p.add(a, "xml-trace", None, "Specify a filename for XML runtime log output to be written to", DataType::Bool);
    p.add(
        a,
        "jvm-args",
        None,
        "Specify a concatenated list of JVM arguments to pass, each starting with a colon",
        DataType::Str,
    )
    .hidden();
    p.add(
        a,
        "checker-args",
        None,
        "Specify a concatenated list of additional checker arguments to pass, each starting with a colon",
        DataType::Str,
    )
    .hidden();
    p.add(
        a,
        "psym-args",
        None,
        "Specify a concatenated list of additional PSym-specific arguments to pass, each starting with a colon",
        DataType::Str,
    )
    .hidden();
    p
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    BugFinding,
    PEx,
}

impl Mode {
    fn name(self) -> &'static str {
        match self {
            Mode::BugFinding => "BugFinding",
            Mode::PEx => "PEx",
        }
    }
}

/// `CheckerConfiguration` (the parts prs uses).
#[derive(Debug, Clone)]
pub struct CheckConfig {
    /// Where the P sources come from.
    pub input: project::Input,
    /// The project, resolved from `input`.
    pub project_name: String,
    pub p_files: Vec<PathBuf>,
    pub foreign_files: Vec<PathBuf>,
    pub output_path: String,
    /// Output directory with a trailing separator, as upstream builds it.
    pub output_directory: String,
    pub mode: Mode,
    pub test_case_name: String,
    pub list_test_cases: bool,
    pub strategy: StrategyKind,
    pub strategy_bound: usize,
    pub iterations: usize,
    /// Workers exploring schedules in parallel; `None` = one per CPU core. Upstream has
    /// one, so `p check` keeps the default of 1.
    pub workers: Option<usize>,
    /// `--schedules` (or `--timeout`) was given.
    pub user_set_iterations: bool,
    pub seed: Option<u32>,
    pub full_exploration: bool,
    pub max_unfair_steps: usize,
    pub max_fair_steps: usize,
    pub user_set_max_fair_steps: bool,
    pub fail_on_max_steps: bool,
    pub liveness_threshold: usize,
    /// Timeout in seconds; `None` disables it.
    pub timeout: Option<u32>,
    pub verbose: bool,
    /// `--xml-trace`: also write the buggy schedule's `.trace.xml`.
    pub xml_trace: bool,
    pub timeline: crate::checker::timeline::TimelineConfig,
    /// `--replay`: the schedule file to replay.
    pub schedule_file: Option<String>,
    /// Latency profiles and the profile to use (time tracking, see `design/time`).
    pub latency: Option<std::sync::Arc<crate::checker::time::ProfileSet>>,
    pub latency_profile: Option<String>,
    /// Time-aware scheduling.
    pub timed: bool,
}

impl Default for CheckConfig {
    fn default() -> CheckConfig {
        CheckConfig {
            input: Default::default(),
            project_name: project::DEFAULT_NAME.into(),
            p_files: Vec::new(),
            foreign_files: Vec::new(),
            output_path: "PCheckerOutput".into(),
            output_directory: String::new(),
            mode: Mode::BugFinding,
            test_case_name: String::new(),
            list_test_cases: false,
            strategy: StrategyKind::Random,
            strategy_bound: 0,
            iterations: 1,
            workers: Some(1),
            user_set_iterations: false,
            seed: None,
            full_exploration: false,
            max_unfair_steps: 10000,
            max_fair_steps: 100000,
            user_set_max_fair_steps: false,
            fail_on_max_steps: false,
            liveness_threshold: 0,
            timeout: None,
            verbose: false,
            xml_trace: false,
            timeline: Default::default(),
            schedule_file: None,
            latency: None,
            latency_profile: None,
            timed: false,
        }
    }
}

/// Runs `p check`; returns the exit code.
pub fn run(args: &[String], host: &mut dyn Host) -> i32 {
    let cfg = match parse(args, host) {
        Ok(c) => c,
        Err(Exit(code)) => return code,
    };
    match run_checker(cfg, host) {
        Ok(code) => code,
        Err(Exit(code)) => code,
    }
}

fn help_and_exit(p: &ArgParser, host: &mut dyn Host, msg: &str) -> Exit {
    host.write(Stream::Out, &p.help(), Some(Color::Cyan));
    report_and_exit(host, msg)
}

fn checker_report_and_exit(host: &mut dyn Host, msg: &str) -> Exit {
    report_and_exit(host, &format!("{} Run `p check --help` for usage help.", msg))
}

/// `PCheckerOptions.Parse`.
pub fn parse(args: &[String], host: &mut dyn Host) -> Result<CheckConfig, Exit> {
    let p = parser();
    let result = match p.parse(args) {
        Ok(r) => r,
        Err(ParseError::Help) => {
            host.write(Stream::Out, &p.help(), Some(Color::Cyan));
            return Err(Exit(1));
        }
        Err(ParseError::Invalid(m)) => return Err(help_and_exit(&p, host, &m)),
    };
    let mut cfg = CheckConfig::default();
    for a in &result {
        let v = a.value.clone();
        let uint = || v.as_ref().map(|v| v.as_uint()).unwrap_or(0);
        let string = || v.as_ref().map(|v| v.as_str().to_string()).unwrap_or_default();
        match a.long_name.as_str() {
            "outdir" => cfg.output_path = string(),
            "verbose" => cfg.verbose = true,
            "xml-trace" => cfg.xml_trace = true,
            "debug" | "memout" | "graph" | "graph-bug" | "jvm-args" | "checker-args" | "psym-args" => {}
            "timeline-repr" => cfg.timeline.repr = crate::checker::timeline::TimelineRepr::parse(&string()),
            "timeline-kgram" => cfg.timeline.kgram = uint() as usize,
            "timeline-payload" => cfg.timeline.payload = true,
            // as upstream, 0 is no timeout, and without --schedules no schedule bound either
            "timeout" => {
                let t = uint();
                cfg.timeout = (t > 0).then_some(t);
                cfg.user_set_iterations = true;
                if cfg.iterations == 1 {
                    cfg.iterations = 0;
                }
            }
            "path" => {
                let f = string();
                let l = f.to_lowercase();
                if l.ends_with(".pproj") {
                    cfg.input.pproj = Some(f);
                } else if l.ends_with(".p") {
                    cfg.input.pfiles.push(f);
                } else {
                    return Err(checker_report_and_exit(
                        host,
                        &format!("'{}' is not a P project file (*.pproj) or P file (*.p). prs checks P sources directly; there is no compiled file.", f),
                    ));
                }
            }
            "pproj" => cfg.input.pproj = Some(string()),
            "pfiles" => cfg.input.pfiles.extend(v.as_ref().map(|v| v.as_str_list()).unwrap_or_default()),
            "projname" => cfg.input.project_name = Some(string()),
            "mode" => cfg.mode = if string().to_lowercase() == "pex" { Mode::PEx } else { Mode::BugFinding },
            "testcase" => cfg.test_case_name = string(),
            "list-tests" => cfg.list_test_cases = true,
            "seed" => cfg.seed = Some(uint()),
            "sch-random" => cfg.strategy = StrategyKind::Random,
            "sch-pos" => cfg.strategy = StrategyKind::Pos,
            "sch-feedback" => cfg.strategy = StrategyKind::Feedback,
            "sch-feedbackpos" => cfg.strategy = StrategyKind::FeedbackPos,
            "sch-rl" => cfg.strategy = StrategyKind::Rl,
            "sch-probabilistic" => (cfg.strategy, cfg.strategy_bound) = (StrategyKind::Probabilistic, uint() as usize),
            "sch-pct" => (cfg.strategy, cfg.strategy_bound) = (StrategyKind::Pct, uint() as usize),
            "sch-fairpct" => (cfg.strategy, cfg.strategy_bound) = (StrategyKind::FairPct, uint() as usize),
            "sch-feedbackpct" => (cfg.strategy, cfg.strategy_bound) = (StrategyKind::FeedbackPct, uint() as usize),
            "sch-pex" => {
                cfg.strategy = match string().as_str() {
                    "random" => StrategyKind::Random,
                    "dfs" => StrategyKind::Dfs,
                    "astar" => {
                        return Err(checker_report_and_exit(
                            host,
                            "The 'astar' scheduling strategy is not supported by prs.",
                        ))
                    }
                    other => unreachable!("--sch-pex does not allow '{}'", other),
                }
            }
            "replay" => {
                let f = string();
                if Path::new(&f).extension().map(|e| e != "schedule").unwrap_or(true) {
                    return Err(checker_report_and_exit(
                        host,
                        "Please give a valid schedule file '--replay x', where 'x' has extension '.schedule'.",
                    ));
                }
                cfg.schedule_file = Some(f);
                cfg.strategy = StrategyKind::Replay;
            }
            "iterations" | "schedules" => {
                cfg.iterations = uint() as usize;
                cfg.user_set_iterations = true;
            }
            "explore" => cfg.full_exploration = true,
            "max-steps" => {
                let vals = v.as_ref().map(|v| v.as_uint_list()).unwrap_or_default();
                if vals.len() > 2 {
                    return Err(checker_report_and_exit(host, "Invalid number of options supplied via '--max-steps'."));
                }
                let i = vals[0];
                let j = if vals.len() == 2 {
                    cfg.user_set_max_fair_steps = true;
                    vals[1]
                } else {
                    i.wrapping_mul(10)
                };
                cfg.max_unfair_steps = i as usize;
                cfg.max_fair_steps = j as usize;
            }
            "fail-on-maxsteps" => cfg.fail_on_max_steps = true,
            // also `--liveness-temperature-threshold`: upstream parses it but has no case for it
            other => return Err(help_and_exit(&p, host, &format!("Unhandled parsed argument: '{}'", other))),
        }
    }
    resolve_sources(host, &mut cfg)?;
    finish_options(&mut cfg);
    if cfg.max_fair_steps < cfg.max_unfair_steps {
        return Err(checker_report_and_exit(host, "For the option '--max-steps N[,M]', please make sure that M >= N."));
    }
    set_output_directory(host, &mut cfg)?;
    Ok(cfg)
}

/// Resolves `cfg.input` to the project name and files.
pub fn resolve_sources(host: &mut dyn Host, cfg: &mut CheckConfig) -> Result<(), Exit> {
    let resolved = project::resolve(host, &cfg.input)?;
    cfg.project_name = resolved.project_name;
    cfg.p_files = resolved.p_files;
    cfg.foreign_files = resolved.foreign_files;
    Ok(())
}

/// The strategy and limits implied by the mode and step bounds (`SanitizeConfiguration`).
pub fn finish_options(cfg: &mut CheckConfig) {
    // PEx mode searches exhaustively: `--sch-pex dfs` in order, otherwise (`random`, the
    // default) trying the alternatives at each choice point in a random order, with no
    // schedule limit unless one is given
    if cfg.mode == Mode::PEx && cfg.strategy != StrategyKind::Replay {
        if cfg.strategy != StrategyKind::Dfs {
            cfg.strategy = StrategyKind::RandomDfs;
        }
        if !cfg.user_set_iterations {
            cfg.iterations = 0;
        }
    }
    if cfg.liveness_threshold == 0 && cfg.max_fair_steps > 0 {
        cfg.liveness_threshold = cfg.max_fair_steps / 2;
    }
}

/// Reads and parses a latency profile file (time tracking).
pub fn load_latency(host: &mut dyn Host, file: &str) -> Result<std::sync::Arc<crate::checker::time::ProfileSet>, Exit> {
    let path = full_path(&host.cwd(), Path::new(file));
    let text = match host.read_to_string(&path) {
        Ok(t) => t,
        Err(e) => return Err(report_and_exit(host, &format!("Cannot read latency file '{}': {}", file, e))),
    };
    match crate::checker::time::ProfileSet::parse(&text) {
        Ok(set) => Ok(std::sync::Arc::new(set)),
        Err(e) => Err(report_and_exit(host, &format!("Latency file '{}': {}", file, e))),
    }
}

fn sanitize_segment(name: &str) -> String {
    name.chars().map(|c| if c.is_alphanumeric() || matches!(c, '.' | '-' | '_') { c } else { '_' }).collect()
}

/// `CheckerConfiguration.SetOutputDirectory`: picks `<outdir>/[<testcase>/]<Mode>/` and
/// rotates up to ten previous runs to `<Mode>0` .. `<Mode>9`.
pub fn set_output_directory(host: &mut dyn Host, cfg: &mut CheckConfig) -> Result<(), Exit> {
    let suffix = if cfg.test_case_name.is_empty() {
        cfg.mode.name().to_string()
    } else {
        format!("{}/{}", sanitize_segment(&cfg.test_case_name), cfg.mode.name())
    };
    let base = if cfg.output_path.is_empty() {
        format!("./Output/{}/", cfg.project_name)
    } else {
        format!("{}/", cfg.output_path)
    };
    cfg.output_directory = format!("{}{}/", base, suffix);
    let cwd = host.cwd();
    let abs = |s: &str| full_path(&cwd, Path::new(s));
    let stem = &cfg.output_directory[..cfg.output_directory.len() - 1];
    let history = |h: usize| format!("{}{}", stem, h);
    const MAX_HISTORY: usize = 10;
    let fail = |host: &mut dyn Host, path: &str, e: std::io::Error| {
        report_and_exit(host, &format!("Cannot prepare output directory '{}': {}", path, e))
    };
    let mut older = history(MAX_HISTORY - 1);
    if host.is_dir(&abs(&older)) {
        host.remove_dir_all(&abs(&older)).map_err(|e| fail(host, &older, e))?;
    }
    if cfg.strategy != StrategyKind::Replay {
        for h in (0..MAX_HISTORY - 1).rev() {
            let newer = history(h);
            if host.is_dir(&abs(&newer)) {
                host.rename(&abs(&newer), &abs(&older)).map_err(|e| fail(host, &newer, e))?;
            }
            older = newer;
        }
        if host.is_dir(&abs(&cfg.output_directory)) {
            host.rename(&abs(&cfg.output_directory), &abs(&older)).map_err(|e| fail(host, &cfg.output_directory, e))?;
        }
    }
    host.create_dir_all(&abs(&cfg.output_directory)).map_err(|e| fail(host, &cfg.output_directory, e))
}

/// Type-checked sources.
pub struct Loaded {
    pub sources: Sources,
    pub analysis: crate::sema::Analysis,
}

/// Reads and type-checks the project's sources (upstream: `TestingEngine.LoadAssembly`);
/// type errors are reported as upstream's compiler reports them.
pub fn load(host: &mut dyn Host, cfg: &CheckConfig) -> Result<Loaded, Exit> {
    let sources = match project::read(host, &cfg.project_name, &cfg.p_files, &host.cwd()) {
        Ok(s) => s,
        Err(e) => return Err(report_and_exit(host, &e)),
    };
    let sm = sources.source_map();
    let r = crate::sema::compile(&sm, &crate::sema::Options::default());
    if !project::report_result(host, &sources, &sm, &r) {
        return Err(Exit(1));
    }
    for f in &cfg.foreign_files {
        host.out_colored_line(
            &format!(
                "Warning: foreign file {} is ignored; prs runs P models directly, so foreign functions must be provided to the checker in Rust.",
                f.display()
            ),
            Color::Yellow,
        );
    }
    let analysis = r.map_err(|_| Exit(1))?;
    Ok(Loaded { sources, analysis })
}

/// `Checker.Run`.
pub fn run_checker(mut cfg: CheckConfig, host: &mut dyn Host) -> Result<i32, Exit> {
    if let Some(schedule_file) = cfg.schedule_file.clone() {
        host.out_line(&format!(
            "Replay option is used, checker is ignoring all other parameters and using the {} to replay the schedule",
            schedule_file
        ));
        host.out_line(&format!("... Replaying {}", schedule_file));
        let loaded = load(host, &cfg)?;
        let tc = select_test_case(host, &loaded, &cfg.test_case_name)?;
        let trace = read_schedule(host, &schedule_file)?;
        replay(host, &loaded, &tc, &mut cfg, trace)?;
        return Ok(0);
    }

    host.out_line(&format!(".. Checking {}", cfg.project_name));
    let mut exit_code = 0;
    // both modes run the same engine; PEx mode with an exhaustive search strategy
    let loaded = load(host, &cfg)?;
    let cases = fetch_test_cases(host, &loaded, &cfg);
    if !cfg.list_test_cases {
        for tc in cases {
            cfg.test_case_name = tc;
            // as upstream, the last test case sets the exit code
            exit_code = testing_process(host, &loaded, &cfg)?;
        }
    }
    host.out_line(". Done");
    Ok(exit_code)
}

/// Reads a `.schedule` file.
pub fn read_schedule(host: &mut dyn Host, schedule_file: &str) -> Result<ScheduleTrace, Exit> {
    let text = match host.read_to_string(&full_path(&host.cwd(), Path::new(schedule_file))) {
        Ok(t) => t,
        Err(e) => return Err(report_and_exit(host, &format!("Cannot read schedule file '{}': {}", schedule_file, e))),
    };
    ScheduleTrace::parse(&text).map_err(|e| report_and_exit(host, &e))
}

/// Replays `trace` on `tc`; returns the number of bugs reproduced.
pub fn replay(
    host: &mut dyn Host,
    loaded: &Loaded,
    tc: &crate::sema::TestCase,
    cfg: &mut CheckConfig,
    trace: ScheduleTrace,
) -> Result<usize, Exit> {
    if let Some(t) = trace.liveness_temperature_threshold {
        cfg.liveness_threshold = t;
    }
    let mut opts = engine_options(cfg, 0);
    opts.replay = Some(trace);
    let start = host.now();
    let res = run_engine(host, loaded, tc, &opts, None, Some(1))?;
    // `ReplayStrategy` reports a schedule it cannot follow with `Error.ReportAndExit`
    if let Some(e) = &res.replay_error {
        return Err(report_and_exit(host, e));
    }
    let elapsed = host.now() - start;
    let n = res.report.num_bugs;
    let text = format!(
        "... Reproduced {} bug{}.\n... Elapsed {} sec and used {} GB.",
        n,
        if n == 1 { "" } else { "s" },
        format_0_00(elapsed),
        format_0_00(host.max_memory_gb())
    );
    host.write(Stream::Out, &text, Some(Color::Yellow));
    Ok(n)
}

/// `TestingProcess.FetchTestCases`.
fn fetch_test_cases(host: &mut dyn Host, loaded: &Loaded, cfg: &CheckConfig) -> Vec<String> {
    let names: Vec<String> = loaded.analysis.program.test_cases().into_iter().map(|t| t.name).collect();
    if cfg.list_test_cases {
        host.out_line(&format!(".. List of test cases (total {})", names.len()));
    } else if cfg.test_case_name.is_empty() {
        return vec![String::new()];
    } else {
        host.out_line(&format!(".. Running test cases with prefix '{}':", cfg.test_case_name));
    }
    let mut out = Vec::new();
    for n in names {
        if cfg.list_test_cases {
            host.out_line(&n);
        } else if n.starts_with(&cfg.test_case_name) {
            host.out_line(&n);
            out.push(n);
        }
    }
    out
}

/// `TestMethodInfo.GetTestMethod`: exact name, else the unique test case containing it.
pub fn select_test_case(host: &mut dyn Host, loaded: &Loaded, name: &str) -> Result<crate::sema::TestCase, Exit> {
    let all = loaded.analysis.program.test_cases();
    let exact: Vec<_> = all.iter().filter(|t| t.name == name).cloned().collect();
    let filtered: Vec<_> =
        if !exact.is_empty() { exact } else { all.iter().filter(|t| t.name.contains(name)).cloned().collect() };
    let options: String = all.iter().map(|t| format!("{}\n", t.name)).collect();
    match filtered.as_slice() {
        [tc] => {
            host.out_line(&format!(".. Test case :: {}", tc.name));
            Ok(tc.clone())
        }
        [_, _, ..] => Err(report_and_exit(
            host,
            &format!(
                "We found '{}' test cases. Please provide a more precise name of the test case you wish to check using (--testcase | -tc).\nPossible options are: \n{}",
                all.len(),
                options
            ),
        )),
        [] if !all.is_empty() => Err(report_and_exit(
            host,
            &format!("Cannot find a P test case that contains {}. Possible options are: \n{}", name, options),
        )),
        [] => Err(report_and_exit(
            host,
            "Cannot detect a P test case to check.\n -- If you have a test case specified and P checker is not finding it, please report this error to the P team --",
        )),
    }
}

fn engine_options(cfg: &CheckConfig, seed: u32) -> CheckerOptions {
    CheckerOptions {
        strategy: cfg.strategy,
        seed,
        iterations: cfg.iterations,
        max_unfair_steps: cfg.max_unfair_steps,
        max_fair_steps: cfg.max_fair_steps,
        user_set_max_fair_steps: cfg.user_set_max_fair_steps,
        liveness_temperature_threshold: Some(cfg.liveness_threshold),
        fail_on_max_steps: cfg.fail_on_max_steps,
        strategy_bound: cfg.strategy_bound,
        full_exploration: cfg.full_exploration,
        incremental_seed: false,
        replay: None,
        record_test_name: !cfg.test_case_name.is_empty(),
        verbose: cfg.verbose,
        json_trace: true,
        xml_trace: cfg.xml_trace,
        timeline: cfg.timeline,
        activity_coverage: true,
        latency: cfg.latency.clone(),
        latency_profile: cfg.latency_profile.clone(),
        timed: cfg.timed,
        ..CheckerOptions::default()
    }
}

/// Engine output on the console, with an optional deadline (`--timeout`).
struct ConsoleOut<'h> {
    host: &'h mut dyn Host,
    deadline: Option<f64>,
}

impl Output for ConsoleOut<'_> {
    fn line(&mut self, s: &str) {
        self.host.out_line(s);
    }
    fn error(&mut self, s: &str) {
        self.host.write(Stream::Out, "Error: ", Some(Color::Red));
        self.host.write(Stream::Out, s, Some(Color::Yellow));
        self.host.write(Stream::Out, "\n", None);
    }
    fn cancelled(&mut self) -> bool {
        self.deadline.map(|d| self.host.now() >= d).unwrap_or(false)
    }
}

/// Runs the engine on `tc` with up to `workers` workers (`None` = one per CPU core).
fn run_engine(
    host: &mut dyn Host,
    loaded: &Loaded,
    tc: &crate::sema::TestCase,
    opts: &CheckerOptions,
    deadline: Option<f64>,
    workers: Option<usize>,
) -> Result<CheckResult, Exit> {
    let prog = &loaded.analysis.program;
    let checker = Checker::new(prog);
    let opts = &CheckerOptions { assembly_name: loaded.sources.project_name.clone(), ..opts.clone() };
    // a host without threads runs one worker
    let threads = host.threads();
    let requested = threads.map_or(1, |t| workers.unwrap_or(t.available));
    let workers = WorkerPool::workers_for(opts, requested);
    let mut out = ConsoleOut { host, deadline };
    let result = match threads {
        Some(threads) if workers > 1 => run_workers(&checker, tc, opts, &WorkerPool::new(workers), threads, &mut out),
        _ => checker.check(tc, opts, &mut out),
    };
    result.map_err(|e| report_and_exit(host, &e))
}

/// A worker's output, sent to the thread that writes the console.
enum Message {
    Line(String),
    Error(String),
    Done(usize, Box<Result<Exploration, String>>),
}

struct WorkerOut<'a> {
    tx: mpsc::Sender<Message>,
    timed_out: &'a AtomicBool,
}

impl Output for WorkerOut<'_> {
    fn line(&mut self, s: &str) {
        let _ = self.tx.send(Message::Line(s.to_string()));
    }
    fn error(&mut self, s: &str) {
        let _ = self.tx.send(Message::Error(s.to_string()));
    }
    fn cancelled(&mut self) -> bool {
        self.timed_out.load(Ordering::Relaxed)
    }
}

/// Explores `tc` with the workers of `pool`, each on its own thread with its own
/// `Checker`; this thread writes their output and watches the deadline.
fn run_workers(
    checker: &Checker,
    tc: &crate::sema::TestCase,
    opts: &CheckerOptions,
    pool: &WorkerPool,
    threads: Threads,
    out: &mut ConsoleOut,
) -> Result<CheckResult, String> {
    checker.announce(tc, opts, out)?;
    out.line(&format!("... Checker is using {} workers.", pool.count()));
    let prog = checker.prog;
    let timed_out = AtomicBool::new(false);
    let (tx, rx) = mpsc::channel();
    let jobs: Vec<Job<'_>> = (0..pool.count())
        .map(|index| {
            let tx = tx.clone();
            let timed_out = &timed_out;
            Box::new(move || {
                let checker = Checker::new(prog);
                let mut out = WorkerOut { tx, timed_out };
                let part = checker.explore(tc, opts, pool, index, &mut out);
                let _ = out.tx.send(Message::Done(index, Box::new(part)));
            }) as Job<'_>
        })
        .collect();
    drop(tx);
    let mut parts: Vec<Option<Result<Exploration, String>>> = (0..pool.count()).map(|_| None).collect();
    let mut main = || loop {
        match rx.recv_timeout(Duration::from_millis(20)) {
            Ok(Message::Line(s)) => out.line(&s),
            Ok(Message::Error(s)) => out.error(&s),
            Ok(Message::Done(index, part)) => parts[index] = Some(*part),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
        if !timed_out.load(Ordering::Relaxed) && out.cancelled() {
            timed_out.store(true, Ordering::Relaxed);
        }
    };
    (threads.run)(jobs, &mut main);
    let parts = parts.into_iter().map(|p| p.expect("every worker reports")).collect::<Result<Vec<_>, String>>()?;
    Ok(checker.finish(tc, opts, parts))
}

/// `TestingProcess.Run` for one test case; returns the exit code it sets.
pub fn testing_process(host: &mut dyn Host, loaded: &Loaded, cfg: &CheckConfig) -> Result<i32, Exit> {
    let process_start = host.now();
    let seed = match cfg.seed {
        Some(s) => s, // + 673 * TestingProcessId: see `WorkerPool::worker_options`
        None => host.random_seed(),
    };
    let tc = select_test_case(host, loaded, &cfg.test_case_name)?;
    let opts = engine_options(cfg, seed);
    let engine_start = host.now();
    let deadline = cfg.timeout.map(|t| engine_start + f64::from(t));
    let res = run_engine(host, loaded, &tc, &opts, deadline, cfg.workers)?;
    let engine_elapsed = host.now() - engine_start;
    if res.cancelled {
        host.out_line("... Checker timed out.");
    }
    let file = cfg.project_name.clone();
    if let Some(j) = &res.verbose_json_trace {
        host.out_line("... Emitting verbose logs:");
        let path = format!("{}{}_0_verbose.trace.json", cfg.output_directory, file);
        host.out_line(&format!("..... Writing {}", path));
        write_output(host, &path, j)?;
    }
    if !cfg.full_exploration && res.report.num_bugs > 0 {
        host.out_line("Checker found a bug.");
        host.out_line("... Emitting traces:");
        emit_traces(host, cfg, &res, &format!("{}_0", file), engine_elapsed)?;
    }
    if cfg.verbose {
        host.out_line("... ### Process 0 is terminating");
    }
    if let Some(cov) = &res.coverage {
        // ReportActivityCoverage (on by default upstream)
        host.out_line("... Emitting coverage report:");
        let base = format!("{}{}", cfg.output_directory, file);
        let checker = Checker::new(&loaded.analysis.program);
        let text = checker.coverage_report(cov);
        let txt = format!("{}.coverage.txt", base);
        host.out_line(&format!("..... Writing {}", txt));
        write_output(host, &txt, &text)?;
        let sci = format!("{}.sci", base);
        host.out_line(&format!("..... Writing {}", sci));
        let xml = crate::compat::sci::coverage_xml(&loaded.analysis.program, &checker.coverage_tables(), cov);
        write_output(host, &sci, &xml)?;
    }
    let process_elapsed = host.now() - process_start;
    host.out_line(&res.report.text(&opts, "..."));
    if let Some(x) = &res.exhaustive {
        let schedules = res.report.explored_fair_schedules + res.report.explored_unfair_schedules;
        host.out_line(&x.text(schedules, opts.max_unfair_steps, "....."));
    }
    let summary_path = format!("{}{}_pchecker_summary.txt", cfg.output_directory, file);
    host.out_line(&format!("..... Writing {}", summary_path));
    let summary = format!(
        "bugs:{}\nschedules:{}\nmax_depth:{}\ntime_seconds:{}\nmemory_max_mb:{}",
        res.report.num_bugs,
        res.report.explored_fair_schedules + res.report.explored_unfair_schedules,
        res.report.max_fair_steps.unwrap_or(0),
        format_0_00(process_elapsed),
        format_0_00(host.max_memory_gb())
    );
    write_output(host, &summary_path, &summary)?;
    if !res.report.scenarios.is_empty() {
        let suffix = if cfg.test_case_name.is_empty() { String::new() } else { format!("_{}", cfg.test_case_name) };
        let path = format!("{}{}{}{}", cfg.output_directory, file, suffix, crate::compat::scenario::FILE_SUFFIX);
        host.out_line(&format!("..... Writing {}", path));
        let art = crate::compat::scenario::from_report(&res.report, &cfg.test_case_name);
        write_output(host, &path, &art.to_json())?;
    }
    host.out_line(&format!("... Elapsed {} sec.", process_elapsed));
    Ok(if res.report.num_bugs > 0 { 1 } else { 0 })
}

/// The trace file index: one more than the largest `<name>_<n>_<index>` already present.
fn next_trace_index(host: &dyn Host, dir: &Path, file: &str) -> usize {
    let mut index = 0;
    for p in host.read_dir(dir).unwrap_or_default() {
        if !host.is_file(&p) {
            continue;
        }
        let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        if !name.starts_with(file) {
            continue;
        }
        if let Some(i) = trailing_index(&name) {
            index = index.max(i + 1);
        }
    }
    index
}

/// Group 3 of `^(.*)_([0-9]+)_([0-9]+)` (greedy).
fn trailing_index(name: &str) -> Option<usize> {
    let b = name.as_bytes();
    for k in (0..b.len()).rev() {
        if b[k] != b'_' {
            continue;
        }
        let d2 = b[k + 1..].iter().take_while(|c| c.is_ascii_digit()).count();
        if d2 == 0 || k + 1 + d2 >= b.len() || b[k + 1 + d2] != b'_' {
            continue;
        }
        let start = k + 2 + d2;
        let d3 = b[start..].iter().take_while(|c| c.is_ascii_digit()).count();
        if d3 == 0 {
            continue;
        }
        return name[start..start + d3].parse().ok();
    }
    None
}

/// `TestingEngine.TryEmitTraces`.
fn emit_traces(
    host: &mut dyn Host,
    cfg: &CheckConfig,
    res: &CheckResult,
    file: &str,
    engine_elapsed: f64,
) -> Result<(), Exit> {
    let dir = full_path(&host.cwd(), Path::new(&cfg.output_directory));
    let index = next_trace_index(host, &dir, file);
    let base = format!("{}{}_{}", cfg.output_directory, file, index);
    if !cfg.full_exploration {
        if let Some(t) = res.readable_trace.as_ref().filter(|t| !t.is_empty()) {
            let path = format!("{}.txt", base);
            host.out_line(&format!("..... Writing {}", path));
            write_output(host, &path, t)?;
        }
    }
    if let Some(x) = &res.xml_trace {
        let path = format!("{}.trace.xml", base);
        host.out_line(&format!("..... Writing {}", path));
        write_output(host, &path, x)?;
    }
    if let Some(j) = &res.json_trace {
        let path = format!("{}.trace.json", base);
        host.out_line(&format!("..... Writing {}", path));
        write_output(host, &path, j)?;
    }
    if !cfg.full_exploration {
        if let Some(t) = res.schedule_trace.as_ref().filter(|t| !t.is_empty()) {
            let path = format!("{}.schedule", base);
            host.out_line(&format!("..... Writing {}", path));
            write_output(host, &path, t)?;
        }
    }
    host.out_line(&format!(
        "... Elapsed {} sec and used {} GB.",
        format_0_00(engine_elapsed),
        format_0_00(host.max_memory_gb())
    ));
    Ok(())
}
