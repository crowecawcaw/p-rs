//! `p check` (`PCheckerOptions`, `Checker.Run`, `TestingProcess`, `TestingEngine` I/O).

use std::path::{Path, PathBuf};

use pnext_checker::engine::{CheckResult, Checker, CheckerOptions, Output};
use pnext_checker::strategy::ScheduleTrace;

use crate::argparse::{ArgParser, DataType, ParseError};
use crate::artifact::{self, Artifact};
use crate::host::{full_path, is_hidden, Color, Host, HostExt, Stream};
use crate::{format_0_00, pproj, report, report_and_exit, Exit};

const STRATEGIES: &[&str] = &[
    "portfolio",
    "random",
    "feedback",
    "feedbackpct",
    "feedbackpos",
    "pct",
    "pos",
    "fairpct",
    "probabilistic",
    "rl",
    "replay",
    "learn",
    "dfs",
    "stateless",
    "astar",
];

/// Strategies pnext implements.
const SUPPORTED_STRATEGIES: &[&str] =
    &["random", "pct", "fairpct", "probabilistic", "pos", "feedback", "feedbackpct", "feedbackpos", "replay"];

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
        "Path to the compiled file to check for correctness (*.dll). If this option is not passed, the compiler searches for a *.dll file in the current folder",
    )
    .optional();
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
    p.add(s, "sch-pct", None, "Choose the PCT scheduling strategy with given maximum number of priority switch points", DataType::UInt);
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
    p.add(s, "timeline-kgram", None, "Gram length for the kgram/hybrid timeline representation (default: 3)", DataType::UInt);
    p.add(s, "timeline-payload", None, "Enrich timeline event labels with a stable hash of the event payload", DataType::Bool);

    p.group("replay", "Replay and debug options");
    p.add(Some("replay"), "replay", Some("r"), "Schedule file to replay", DataType::Str);

    let a = Some("advanced");
    p.group("advanced", "Advanced options");
    p.add(a, "explore", None, "Keep testing until the bound (e.g. schedule or time) is reached", DataType::Bool);
    p.add(a, "seed", None, "Specify the random value generator seed", DataType::UInt);
    p.add(a, "graph-bug", None, "Output a DGML graph of the schedule that found a bug", DataType::Bool);
    p.add(a, "graph", None, "Output a DGML graph of all test schedules whether a bug was found or not", DataType::Bool);
    p.add(a, "xml-trace", None, "Specify a filename for XML runtime log output to be written to", DataType::Bool);
    p.add(a, "jvm-args", None, "Specify a concatenated list of JVM arguments to pass, each starting with a colon", DataType::Str)
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

/// `CheckerConfiguration` (the parts pnext uses).
#[derive(Debug, Clone)]
pub struct CheckConfig {
    pub pcompiled_path: String,
    pub output_path: String,
    /// Output directory with a trailing separator, as upstream builds it.
    pub output_directory: String,
    pub assembly: String,
    pub mode: Mode,
    pub test_case_name: String,
    pub list_test_cases: bool,
    pub strategy: String,
    pub strategy_bound: usize,
    pub iterations: usize,
    pub seed: Option<u32>,
    pub full_exploration: bool,
    pub max_unfair_steps: usize,
    pub max_fair_steps: usize,
    pub user_set_max_fair_steps: bool,
    pub fail_on_max_steps: bool,
    pub liveness_threshold: usize,
    pub timeout: u32,
    pub verbose: bool,
    pub schedule_file: String,
}

impl CheckConfig {
    fn new(host: &dyn Host) -> CheckConfig {
        CheckConfig {
            pcompiled_path: host.cwd().to_string_lossy().to_string(),
            output_path: "PCheckerOutput".into(),
            output_directory: String::new(),
            assembly: String::new(),
            mode: Mode::BugFinding,
            test_case_name: String::new(),
            list_test_cases: false,
            strategy: "random".into(),
            strategy_bound: 0,
            iterations: 1,
            seed: None,
            full_exploration: false,
            max_unfair_steps: 10000,
            max_fair_steps: 100000,
            user_set_max_fair_steps: false,
            fail_on_max_steps: false,
            liveness_threshold: 0,
            timeout: 0,
            verbose: false,
            schedule_file: String::new(),
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
    let mut cfg = CheckConfig::new(host);
    if let Some(pp) = pproj::find_local_pproj(host) {
        cfg.pcompiled_path = pproj::parse_for_checker(host, &pp)?;
    }
    for a in &result {
        let v = a.value.clone();
        let uint = || v.as_ref().map(|v| v.as_uint()).unwrap_or(0);
        let string = || v.as_ref().map(|v| v.as_str().to_string()).unwrap_or_default();
        match a.long_name.as_str() {
            "outdir" => cfg.output_path = string(),
            "verbose" => cfg.verbose = true,
            "debug" | "memout" | "graph" | "graph-bug" | "xml-trace" | "jvm-args" | "checker-args" | "psym-args"
            | "timeline-repr" | "timeline-kgram" | "timeline-payload" => {}
            "timeout" => {
                cfg.timeout = uint();
                if cfg.iterations == 1 {
                    cfg.iterations = 0;
                }
            }
            "path" => cfg.assembly = string(),
            "mode" => cfg.mode = if string().to_lowercase() == "pex" { Mode::PEx } else { Mode::BugFinding },
            "testcase" => cfg.test_case_name = string(),
            "list-tests" => cfg.list_test_cases = true,
            "seed" => cfg.seed = Some(uint()),
            "sch-random" | "sch-pos" | "sch-feedbackpos" | "sch-feedback" => cfg.strategy = a.long_name[4..].to_string(),
            "sch-probabilistic" | "sch-pct" | "sch-fairpct" | "sch-feedbackpct" => {
                cfg.strategy = a.long_name[4..].to_string();
                cfg.strategy_bound = uint() as usize;
            }
            "sch-rl" => cfg.strategy = "rl".into(),
            "sch-pex" => cfg.strategy = string(),
            "replay" => {
                let f = string();
                if Path::new(&f).extension().map(|e| e != "schedule").unwrap_or(true) {
                    return Err(checker_report_and_exit(
                        host,
                        "Please give a valid schedule file '--replay x', where 'x' has extension '.schedule'.",
                    ));
                }
                cfg.schedule_file = f;
                cfg.strategy = "replay".into();
            }
            "iterations" | "schedules" => cfg.iterations = uint() as usize,
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
            other => return Err(help_and_exit(&p, host, &format!("Unhandled parsed argument: '{}'", other))),
        }
    }
    if cfg.assembly.is_empty() {
        find_local_compiled_file(host, &mut cfg)?;
    }
    // SanitizeConfiguration
    if cfg.liveness_threshold == 0 && cfg.max_fair_steps > 0 {
        cfg.liveness_threshold = cfg.max_fair_steps / 2;
    }
    if !STRATEGIES.contains(&cfg.strategy.as_str()) {
        return Err(checker_report_and_exit(host, "Please provide a scheduling strategy (see --sch* options)"));
    }
    if cfg.max_fair_steps < cfg.max_unfair_steps {
        return Err(checker_report_and_exit(host, "For the option '--max-steps N[,M]', please make sure that M >= N."));
    }
    set_output_directory(host, &mut cfg);
    Ok(cfg)
}

/// `FindLocalPCompiledFile`.
fn find_local_compiled_file(host: &mut dyn Host, cfg: &mut CheckConfig) -> Result<(), Exit> {
    host.out_line(&format!(".. Searching for a P compiled file locally in folder {}", cfg.pcompiled_path));
    let pattern = format!("*.{}", artifact::EXTENSION);
    let root = full_path(&host.cwd(), Path::new(&cfg.pcompiled_path));
    let mut files = Vec::new();
    pproj::list_files(host, &root, Some(3), &mut files);
    for f in files {
        if is_hidden(&f) || f.extension().map(|e| e != artifact::EXTENSION).unwrap_or(true) {
            continue;
        }
        let rel = f.strip_prefix(&root).unwrap_or(&f);
        if !rel.parent().map(|p| p.components().any(|c| c.as_os_str() == "PChecker")).unwrap_or(false) {
            continue;
        }
        let mut display = cfg.pcompiled_path.clone();
        if !display.ends_with('/') {
            display.push('/');
        }
        display.push_str(&rel.to_string_lossy());
        cfg.assembly = display;
        host.out_line(&format!(".. Found a P compiled file: {}", cfg.assembly));
        return Ok(());
    }
    host.out_line(&format!(
        ".. No P compiled file {} found in the current folder: {}",
        pattern,
        host.cwd().display()
    ));
    Err(checker_report_and_exit(host, &format!("Could not find any {} file.", pattern)))
}

fn sanitize_segment(name: &str) -> String {
    name.chars().map(|c| if c.is_alphanumeric() || matches!(c, '.' | '-' | '_') { c } else { '_' }).collect()
}

/// `CheckerConfiguration.SetOutputDirectory`: picks `<outdir>/[<testcase>/]<Mode>/` and
/// rotates up to ten previous runs to `<Mode>0` .. `<Mode>9`.
fn set_output_directory(host: &mut dyn Host, cfg: &mut CheckConfig) {
    let suffix = if cfg.test_case_name.is_empty() {
        cfg.mode.name().to_string()
    } else {
        format!("{}/{}", sanitize_segment(&cfg.test_case_name), cfg.mode.name())
    };
    let base = if cfg.output_path.is_empty() {
        let parent = Path::new(&cfg.assembly).parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
        let parent = if parent.is_empty() { ".".to_string() } else { parent };
        let name = Path::new(&cfg.assembly).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        format!("{}/Output/{}/", parent, name)
    } else {
        format!("{}/", cfg.output_path)
    };
    cfg.output_directory = format!("{}{}/", base, suffix);
    let cwd = host.cwd();
    let abs = |s: &str| full_path(&cwd, Path::new(s));
    let stem = &cfg.output_directory[..cfg.output_directory.len() - 1];
    let history = |h: usize| format!("{}{}", stem, h);
    const MAX_HISTORY: usize = 10;
    let mut older = history(MAX_HISTORY - 1);
    if host.is_dir(&abs(&older)) {
        let _ = host.remove_dir_all(&abs(&older));
    }
    if cfg.strategy != "replay" {
        for h in (0..MAX_HISTORY - 1).rev() {
            let newer = history(h);
            if host.is_dir(&abs(&newer)) {
                let _ = host.rename(&abs(&newer), &abs(&older));
            }
            older = newer;
        }
        if host.is_dir(&abs(&cfg.output_directory)) {
            let _ = host.rename(&abs(&cfg.output_directory), &abs(&older));
        }
    }
    let _ = host.create_dir_all(&abs(&cfg.output_directory));
}

/// A loaded, type-checked compiled file.
pub struct Loaded {
    pub artifact: Artifact,
    pub analysis: pnext_sema::Analysis,
}

/// Resolves a compiled-file path: `.pnext` as is; for an upstream-style `.dll` path, the
/// `.pnext` next to it or in its parent folder.
fn resolve_compiled(host: &dyn Host, path: &str) -> Option<PathBuf> {
    let full = full_path(&host.cwd(), Path::new(path));
    if full.extension().map(|e| e == artifact::EXTENSION).unwrap_or(false) {
        return host.is_file(&full).then_some(full);
    }
    let with = full.with_extension(artifact::EXTENSION);
    if host.is_file(&with) {
        return Some(with);
    }
    let name = with.file_name()?.to_owned();
    let up = with.parent()?.parent()?.join(name);
    host.is_file(&up).then_some(up)
}

/// `TestingEngine.LoadAssembly`.
pub fn load(host: &mut dyn Host, path: &str) -> Result<Loaded, Exit> {
    let not_found = format!("Could not load file or assembly '{}'. The system cannot find the file specified.", path);
    let Some(file) = resolve_compiled(host, path) else {
        return Err(report_and_exit(host, &not_found));
    };
    let text = match host.read_to_string(&file) {
        Ok(t) => t,
        Err(_) => return Err(report_and_exit(host, &not_found)),
    };
    let artifact = match Artifact::from_json(&text) {
        Ok(a) => a,
        Err(e) => return Err(report_and_exit(host, &format!("Could not load file or assembly '{}'. {}", path, e))),
    };
    let sm = artifact.source_map();
    match pnext_sema::compile(&sm, &pnext_sema::Options::default()) {
        Ok(analysis) => Ok(Loaded { artifact, analysis }),
        Err(_) => Err(report_and_exit(
            host,
            &format!("The compiled file '{}' does not type-check; recompile it with `p compile`.", path),
        )),
    }
}

/// `Checker.Run`.
pub fn run_checker(mut cfg: CheckConfig, host: &mut dyn Host) -> Result<i32, Exit> {
    if cfg.strategy == "replay" {
        host.out_line(&format!(
            "Replay option is used, checker is ignoring all other parameters and using the {} to replay the schedule",
            cfg.schedule_file
        ));
        host.out_line(&format!("... Replaying {}", cfg.schedule_file));
        if cfg.mode == Mode::PEx {
            report(host, "[PTool] Checker with PEx mode is currently unsupported.");
            return Ok(0);
        }
        let loaded = load(host, &cfg.assembly)?;
        let tc_name = cfg.test_case_name.clone();
        let tc = select_test_case(host, &loaded, &tc_name)?;
        let text = match host.read_to_string(&full_path(&host.cwd(), Path::new(&cfg.schedule_file))) {
            Ok(t) => t,
            Err(e) => return Err(report_and_exit(host, &e.to_string())),
        };
        let trace = ScheduleTrace::parse(&text).map_err(|e| report_and_exit(host, &e))?;
        if let Some(t) = trace.liveness_temperature_threshold {
            cfg.liveness_threshold = t;
        }
        let mut opts = engine_options(&cfg, 0);
        opts.replay = Some(trace);
        let start = host.now();
        let res = run_engine(host, &loaded, &tc, &opts, None)?;
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
        return Ok(0);
    }

    host.out_line(&format!(".. Checking {}", cfg.assembly));
    let mut exit_code = 0;
    match cfg.mode {
        Mode::PEx => report(host, "[PTool] Checker with PEx mode is currently unsupported."),
        Mode::BugFinding => {
            let loaded = load(host, &cfg.assembly)?;
            let cases = fetch_test_cases(host, &loaded, &cfg);
            if !cfg.list_test_cases {
                for tc in cases {
                    cfg.test_case_name = tc;
                    exit_code = testing_process(host, &loaded, &cfg)?;
                }
            }
        }
    }
    host.out_line(". Done");
    Ok(exit_code)
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
fn select_test_case(host: &mut dyn Host, loaded: &Loaded, name: &str) -> Result<pnext_sema::TestCase, Exit> {
    let all = loaded.analysis.program.test_cases();
    let exact: Vec<_> = all.iter().filter(|t| t.name == name).cloned().collect();
    let filtered: Vec<_> = if !exact.is_empty() { exact } else { all.iter().filter(|t| t.name.contains(name)).cloned().collect() };
    if filtered.is_empty() {
        if !all.is_empty() {
            let mut msg = format!("Cannot find a P test case that contains {}. Possible options are: \n", name);
            for t in &all {
                msg.push_str(&t.name);
                msg.push('\n');
            }
            return Err(report_and_exit(host, &msg));
        }
        return Err(report_and_exit(
            host,
            "Cannot detect a P test case to check.\n -- If you have a test case specified and P checker is not finding it, please report this error to the P team --",
        ));
    }
    if filtered.len() > 1 {
        let mut msg = format!(
            "We found '{}' test cases. Please provide a more precise name of the test case you wish to check using (--testcase | -tc).\nPossible options are: \n",
            all.len()
        );
        for t in &all {
            msg.push_str(&t.name);
            msg.push('\n');
        }
        return Err(report_and_exit(host, &msg));
    }
    let tc = filtered.into_iter().next().unwrap();
    host.out_line(&format!(".. Test case :: {}", tc.name));
    Ok(tc)
}

fn engine_options(cfg: &CheckConfig, seed: u32) -> CheckerOptions {
    CheckerOptions {
        strategy: cfg.strategy.clone(),
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
        activity_coverage: true,
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

fn run_engine(
    host: &mut dyn Host,
    loaded: &Loaded,
    tc: &pnext_sema::TestCase,
    opts: &CheckerOptions,
    deadline: Option<f64>,
) -> Result<CheckResult, Exit> {
    if !SUPPORTED_STRATEGIES.contains(&opts.strategy.as_str()) {
        return Err(report_and_exit(
            host,
            &format!("The '{}' scheduling strategy is not supported by pnext.", opts.strategy),
        ));
    }
    let checker = Checker::new(&loaded.analysis.program);
    let mut out = ConsoleOut { host, deadline };
    match checker.check(tc, opts, &mut out) {
        Ok(r) => Ok(r),
        Err(e) => Err(report_and_exit(host, &e)),
    }
}

/// `TestingProcess.Run` for one test case; returns the exit code it sets.
fn testing_process(host: &mut dyn Host, loaded: &Loaded, cfg: &CheckConfig) -> Result<i32, Exit> {
    let process_start = host.now();
    let seed = match cfg.seed {
        Some(s) => s, // + 673 * TestingProcessId (always 0)
        None => host.random_seed(),
    };
    let tc = select_test_case(host, loaded, &cfg.test_case_name)?;
    let opts = engine_options(cfg, seed);
    let engine_start = host.now();
    let deadline = (cfg.timeout > 0).then(|| engine_start + cfg.timeout as f64);
    let res = run_engine(host, loaded, &tc, &opts, deadline)?;
    let engine_elapsed = host.now() - engine_start;
    if res.cancelled {
        host.out_line("... Checker timed out.");
    }
    let file = Path::new(&cfg.assembly).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    if !cfg.full_exploration && res.report.num_bugs > 0 {
        host.out_line("Checker found a bug.");
        host.out_line("... Emitting traces:");
        emit_traces(host, cfg, &res, &format!("{}_0", file), engine_elapsed);
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
        let _ = host.write_file(&full_path(&host.cwd(), Path::new(&txt)), &text);
        let sci = format!("{}.sci", base);
        host.out_line(&format!("..... Writing {}", sci));
        let xml = crate::sci::coverage_xml(&loaded.analysis.program, &checker.coverage_tables(), cov);
        let _ = host.write_file(&full_path(&host.cwd(), Path::new(&sci)), &xml);
    }
    let process_elapsed = host.now() - process_start;
    host.out_line(&res.report.text(&opts, "..."));
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
    let _ = host.write_file(&full_path(&host.cwd(), Path::new(&summary_path)), &summary);
    if !res.report.scenarios.is_empty() {
        let suffix = if cfg.test_case_name.is_empty() { String::new() } else { format!("_{}", cfg.test_case_name) };
        let path = format!("{}{}{}{}", cfg.output_directory, file, suffix, crate::scenario::FILE_SUFFIX);
        host.out_line(&format!("..... Writing {}", path));
        let art = crate::scenario::from_report(&res.report, &cfg.test_case_name);
        let _ = host.write_file(&full_path(&host.cwd(), Path::new(&path)), &art.to_json());
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
fn emit_traces(host: &mut dyn Host, cfg: &CheckConfig, res: &CheckResult, file: &str, engine_elapsed: f64) {
    let dir = full_path(&host.cwd(), Path::new(&cfg.output_directory));
    let index = next_trace_index(host, &dir, file);
    let base = format!("{}{}_{}", cfg.output_directory, file, index);
    if !cfg.full_exploration {
        if let Some(t) = res.readable_trace.as_ref().filter(|t| !t.is_empty()) {
            let path = format!("{}.txt", base);
            host.out_line(&format!("..... Writing {}", path));
            let _ = host.write_file(&full_path(&host.cwd(), Path::new(&path)), t);
        }
    }
    if let Some(j) = &res.json_trace {
        let path = format!("{}.trace.json", base);
        host.out_line(&format!("..... Writing {}", path));
        let _ = host.write_file(&full_path(&host.cwd(), Path::new(&path)), j);
    }
    if !cfg.full_exploration {
        if let Some(t) = res.schedule_trace.as_ref().filter(|t| !t.is_empty()) {
            let path = format!("{}.schedule", base);
            host.out_line(&format!("..... Writing {}", path));
            let _ = host.write_file(&full_path(&host.cwd(), Path::new(&path)), t);
        }
    }
    host.out_line(&format!(
        "... Elapsed {} sec and used {} GB.",
        format_0_00(engine_elapsed),
        format_0_00(host.max_memory_gb())
    ));
}
