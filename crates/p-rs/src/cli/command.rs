//! The `prs` command line.
//!
//! `prs` runs the commands of [`crate::driver`], as `prs-compat` does, with its
//! own options: test cases and sources as arguments, one `--strategy` option, durations
//! with units, parallel workers and time tracking. Exit status: 0 when no bug is found,
//! 1 when one is, 2 on any other error.

use std::path::Path;

use crate::checker::strategy::StrategyKind;
use crate::checker::timeline::TimelineRepr;
use crate::driver::check::{self, CheckConfig, Mode};
use crate::driver::Exit;
use crate::host::{full_path, Buffered, Color, Host, HostExt, Stream};

const OK: i32 = 0;
const BUG: i32 = 1;
const ERROR: i32 = 2;

const HELP: &str = "\
prs checks P models.

Usage: prs <command> [options]

Commands:
  check      Check test cases by exploring schedules
  replay     Replay a .schedule file
  tests      List the test cases
  observe    Check logs against spec monitors
  coverage   Merge scenario or observed-monitor coverage reports
  version    Print the version

Commands that read P sources take .pproj or .p files. Without any, they use the
*.pproj file in the current folder, or else the *.p files in it and its immediate
subfolders.

Run `prs <command> --help` for a command's options.
Exit status: 0 no bug found, 1 bug found, 2 error.
";

const CHECK_HELP: &str = "\
Usage: prs check [SOURCE...] [TEST...] [options]

Checks test cases by exploring schedules. A SOURCE is a .pproj or .p file, a TEST
a test case name or a glob (`*`, `?`). Without tests, all test cases are checked.

Exploration:
  -n, --schedules <N>       Schedules per test case (default: 10000; no limit with
                            --time or --exhaustive)
      --time <DURATION>     Time limit per test case: seconds, or with a unit (90s,
                            5m, 1h)
  -j, --workers <N>         Workers exploring schedules in parallel (default: one per
                            CPU core). The feedback strategies, rl, --exhaustive and
                            --verbose use one.
      --seed <N>            Random seed. With one worker, the schedules are those
                            upstream P explores for the seed.
      --strategy <S>        random (default), pct:<N>, fairpct:<N>, pos, prob:<N>,
                            feedback, feedback-pct:<N>, feedback-pos, rl. <N> is the
                            number of priority switch points for pct, or the
                            probability 0.5^<N> of switching for prob.
      --exhaustive          Explore every scheduling and data choice, depth first
      --max-steps <N>       Steps per schedule (default: 10000)
      --fair-max-steps <N>  Steps per fair schedule (default: 10 x --max-steps)
      --fail-on-max-steps   Report a schedule reaching the step limit as a bug

Feedback strategies:
      --timeline <R>        Timeline representation: pairwise (default), kgram[:<N>],
                            causal, hybrid[:<N>] (<N>: gram length, default 3)
      --timeline-payload    Add event payload hashes to timeline labels

Time:
      --latency <FILE>[:<PROFILE>]
                            Latency profiles (TOML), and the profile to use
      --timed               Time-aware scheduling: never run a step that is certainly
                            later than another

Output:
  -o, --out <DIR>           Output folder (default: PCheckerOutput)
  -v, --verbose             Log every schedule
";

const REPLAY_HELP: &str = "\
Usage: prs replay <FILE.schedule> [SOURCE...] [options]

Replays a schedule written by `prs check`. Exit status 1 means the bug was reproduced.

Options:
      --test <NAME>         Test case to replay, for schedules that name none
  -v, --verbose             Log the schedule
";

const TESTS_HELP: &str = "\
Usage: prs tests [SOURCE...]

Type-checks the sources and lists their test cases, one per line.
";

const COVERAGE_HELP: &str = "\
Usage: prs coverage merge [DIR]
       prs coverage merge-observe [REPORT.json ...] [--stdin] [--json-report OUT] [--json]

Merges the scenario coverage reports (*_scenario_coverage.json) under DIR (default:
the current folder), recursively, into one report.

merge-observe combines observe --coverage JSON reports without loading P sources.
It validates matching spec versions and preserves failure/completion metadata.
Run `prs coverage merge-observe --help` for input and output options.
";

/// Runs `prs` with `args` (without the program name); returns the exit status.
pub fn run(args: &[String], host: &mut dyn Host) -> i32 {
    let Some((cmd, rest)) = args.split_first() else {
        host.write(Stream::Out, HELP, None);
        return OK;
    };
    match cmd.as_str() {
        "check" => check(rest, host),
        "replay" => replay(rest, host),
        "tests" => tests(rest, host),
        "observe" => observe(rest, host),
        "coverage" => coverage(rest, host),
        "version" | "--version" | "-V" => {
            host.out_line(&format!("prs {}", env!("CARGO_PKG_VERSION")));
            OK
        }
        "help" | "--help" | "-h" => match rest.first().map(String::as_str) {
            None => {
                host.write(Stream::Out, HELP, None);
                OK
            }
            Some(c @ ("check" | "replay" | "tests" | "observe" | "coverage")) => {
                run(&[c.to_string(), "--help".into()], host)
            }
            Some(c) => usage_error(host, &format!("unknown command '{}'", c), None),
        },
        c => usage_error(host, &format!("unknown command '{}'", c), None),
    }
}

/// Reports a command line error; exit status 2.
fn usage_error(host: &mut dyn Host, msg: &str, cmd: Option<&str>) -> i32 {
    error(host, msg);
    let help = cmd.map_or("prs --help".to_string(), |c| format!("prs {} --help", c));
    host.write(Stream::Err, &format!("Run `{}` for usage.\n", help), None);
    ERROR
}

fn error(host: &mut dyn Host, msg: &str) {
    host.write(Stream::Err, "error: ", Some(Color::Red));
    host.write(Stream::Err, msg, None);
    host.write(Stream::Err, "\n", None);
}

// ---- options ----------------------------------------------------------------------------

/// An option a command accepts.
struct Opt {
    long: &'static str,
    short: Option<char>,
    takes_value: bool,
}

const fn flag(long: &'static str, short: Option<char>) -> Opt {
    Opt { long, short, takes_value: false }
}

const fn value(long: &'static str, short: Option<char>) -> Opt {
    Opt { long, short, takes_value: true }
}

/// Parsed arguments: positionals, and options (long name, value) in order.
#[derive(Default)]
struct Args {
    positional: Vec<String>,
    options: Vec<(&'static str, String)>,
    help: bool,
}

impl Args {
    fn has(&self, long: &str) -> bool {
        self.options.iter().any(|(l, _)| *l == long)
    }
    /// The last value given for `long`.
    fn get(&self, long: &str) -> Option<&str> {
        self.options.iter().rev().find(|(l, _)| *l == long).map(|(_, v)| v.as_str())
    }
}

fn parse_args(args: &[String], spec: &[Opt]) -> Result<Args, String> {
    let mut out = Args::default();
    let mut it = args.iter();
    let mut only_positional = false;
    while let Some(a) = it.next() {
        if only_positional || a == "-" || !a.starts_with('-') {
            out.positional.push(a.clone());
            continue;
        }
        if a == "--" {
            only_positional = true;
            continue;
        }
        if a == "-h" || a == "--help" {
            out.help = true;
            continue;
        }
        let (opt, inline) = if let Some(long) = a.strip_prefix("--") {
            let (name, inline) = match long.split_once('=') {
                Some((n, v)) => (n, Some(v.to_string())),
                None => (long, None),
            };
            (spec.iter().find(|o| o.long == name), inline)
        } else {
            let mut chars = a[1..].chars();
            let c = chars.next();
            let opt = if chars.next().is_none() { spec.iter().find(|o| o.short == c) } else { None };
            (opt, None)
        };
        let Some(opt) = opt else {
            return Err(format!("unknown option '{}'", a));
        };
        let v = match (opt.takes_value, inline) {
            (true, Some(v)) => v,
            (true, None) => it.next().cloned().ok_or_else(|| format!("'{}' needs a value", a))?,
            (false, None) => String::new(),
            (false, Some(_)) => return Err(format!("'--{}' takes no value", opt.long)),
        };
        out.options.push((opt.long, v));
    }
    Ok(out)
}

fn uint(args: &Args, long: &str) -> Result<Option<usize>, String> {
    args.get(long)
        .map(|v| v.parse::<usize>().map_err(|_| format!("'--{}' needs a non-negative whole number, not '{}'", long, v)))
        .transpose()
}

/// A duration in seconds: `90`, `90s`, `5m` or `1h`.
fn parse_duration(s: &str) -> Option<u32> {
    let (num, unit) = match s.char_indices().find(|(_, c)| !c.is_ascii_digit()) {
        Some((i, _)) => (&s[..i], &s[i..]),
        None => (s, "s"),
    };
    let n: u32 = num.parse().ok()?;
    let scale = match unit {
        "s" => 1,
        "m" => 60,
        "h" => 3600,
        _ => return None,
    };
    n.checked_mul(scale).filter(|&t| t > 0)
}

/// `--strategy`: the strategy and its bound.
fn parse_strategy(s: &str) -> Result<(StrategyKind, usize), String> {
    let (name, bound) = match s.split_once(':') {
        Some((n, b)) => {
            let b = b.parse::<usize>().map_err(|_| format!("bad number in strategy '{}'", s))?;
            (n, Some(b))
        }
        None => (s, None),
    };
    let (kind, needs_bound) = match name {
        "random" => (StrategyKind::Random, false),
        "pct" => (StrategyKind::Pct, true),
        "fairpct" => (StrategyKind::FairPct, true),
        "pos" => (StrategyKind::Pos, false),
        "prob" => (StrategyKind::Probabilistic, true),
        "feedback" => (StrategyKind::Feedback, false),
        "feedback-pct" => (StrategyKind::FeedbackPct, true),
        "feedback-pos" => (StrategyKind::FeedbackPos, false),
        "rl" => (StrategyKind::Rl, false),
        _ => return Err(format!("unknown strategy '{}'", s)),
    };
    match (needs_bound, bound) {
        (true, Some(b)) => Ok((kind, b)),
        (true, None) => Err(format!("strategy '{}' needs a number, e.g. '{}:10'", name, name)),
        (false, None) => Ok((kind, 0)),
        (false, Some(_)) => Err(format!("strategy '{}' takes no number", name)),
    }
}

/// `--timeline`: the representation and gram length.
fn parse_timeline(s: &str, cfg: &mut crate::checker::timeline::TimelineConfig) -> Result<(), String> {
    let (name, k) = match s.split_once(':') {
        Some((n, k)) => (n, Some(k.parse::<usize>().map_err(|_| format!("bad gram length in '{}'", s))?)),
        None => (s, None),
    };
    cfg.repr = match name {
        "pairwise" => TimelineRepr::Pairwise,
        "kgram" => TimelineRepr::KGram,
        "causal" => TimelineRepr::Causal,
        "hybrid" => TimelineRepr::Hybrid,
        _ => return Err(format!("unknown timeline representation '{}'", s)),
    };
    match k {
        Some(_) if !matches!(cfg.repr, TimelineRepr::KGram | TimelineRepr::Hybrid) => {
            Err(format!("timeline '{}' takes no gram length", name))
        }
        Some(k) => {
            cfg.kgram = k;
            Ok(())
        }
        None => Ok(()),
    }
}

/// `--latency FILE[:PROFILE]`. The profile follows the last `:`, unless that `:` belongs to
/// the path (as in `C:\x.toml`).
fn split_latency(s: &str) -> (&str, Option<&str>) {
    match s.rsplit_once(':') {
        Some((file, profile))
            if !file.is_empty() && !profile.is_empty() && !profile.contains(['/', '\\']) && file.len() > 1 =>
        {
            (file, Some(profile))
        }
        _ => (s, None),
    }
}

fn is_source(arg: &str) -> bool {
    let l = arg.to_lowercase();
    l.ends_with(".pproj") || l.ends_with(".p")
}

/// Splits positionals into the sources (`.pproj`, `.p`) and the rest.
fn sources(cfg: &mut CheckConfig, positional: &[String]) -> Result<Vec<String>, String> {
    let mut rest = Vec::new();
    for a in positional {
        if a.to_lowercase().ends_with(".pproj") {
            if let Some(p) = &cfg.input.pproj {
                return Err(format!("two project files given: '{}' and '{}'", p, a));
            }
            cfg.input.pproj = Some(a.clone());
        } else if is_source(a) {
            cfg.input.pfiles.push(a.clone());
        } else {
            rest.push(a.clone());
        }
    }
    Ok(rest)
}

/// Whether `name` matches the glob `pattern` (`*`: any characters, `?`: one).
fn glob_match(pattern: &str, name: &str) -> bool {
    let (p, n): (Vec<char>, Vec<char>) = (pattern.chars().collect(), name.chars().collect());
    let (mut pi, mut ni) = (0, 0);
    let mut star: Option<(usize, usize)> = None;
    while ni < n.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == n[ni]) {
            pi += 1;
            ni += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = Some((pi, ni));
            pi += 1;
        } else if let Some((sp, sn)) = star {
            pi = sp + 1;
            ni = sn + 1;
            star = Some((sp, sn + 1));
        } else {
            return false;
        }
    }
    p[pi..].iter().all(|&c| c == '*')
}

/// The test cases matching `patterns` (all for none), in declaration order.
fn select_tests(names: &[String], patterns: &[String]) -> Result<Vec<String>, String> {
    if patterns.is_empty() {
        return Ok(names.to_vec());
    }
    for p in patterns {
        if !names.iter().any(|n| glob_match(p, n)) {
            return Err(format!("no test case matches '{}'. Test cases: {}", p, names.join(", ")));
        }
    }
    Ok(names.iter().filter(|n| patterns.iter().any(|p| glob_match(p, n))).cloned().collect())
}

// ---- commands -----------------------------------------------------------------------------

const CHECK_OPTS: &[Opt] = &[
    value("schedules", Some('n')),
    value("time", None),
    value("workers", Some('j')),
    value("seed", None),
    value("strategy", None),
    flag("exhaustive", None),
    value("max-steps", None),
    value("fair-max-steps", None),
    flag("fail-on-max-steps", None),
    value("timeline", None),
    flag("timeline-payload", None),
    value("latency", None),
    flag("timed", None),
    value("out", Some('o')),
    flag("verbose", Some('v')),
];

fn check(args: &[String], host: &mut dyn Host) -> i32 {
    let parsed = match parse_args(args, CHECK_OPTS) {
        Ok(a) => a,
        Err(m) => return usage_error(host, &m, Some("check")),
    };
    if parsed.help {
        host.write(Stream::Out, CHECK_HELP, None);
        return OK;
    }
    let mut cfg = CheckConfig::default();
    let patterns = match check_config(&parsed, &mut cfg) {
        Ok(p) => p,
        Err(m) => return usage_error(host, &m, Some("check")),
    };
    match run_check(host, cfg, &patterns, parsed.get("latency")) {
        Ok(code) => code,
        Err(Exit(_)) => ERROR,
    }
}

/// Fills `cfg` from the `check` options; returns the test patterns.
fn check_config(a: &Args, cfg: &mut CheckConfig) -> Result<Vec<String>, String> {
    let patterns = sources(cfg, &a.positional)?;
    let schedules = uint(a, "schedules")?;
    cfg.user_set_iterations = schedules.is_some();
    if let Some(t) = a.get("time") {
        cfg.timeout =
            Some(parse_duration(t).ok_or_else(|| format!("bad duration '{}' (examples: 90, 90s, 5m, 1h)", t))?);
    }
    let unbounded = cfg.timeout.is_some() || a.has("exhaustive");
    cfg.iterations = schedules.unwrap_or(if unbounded { 0 } else { 10000 });
    cfg.workers = match uint(a, "workers")? {
        Some(0) => return Err("'--workers' needs at least 1 worker".into()),
        n => n,
    };
    if let Some(s) = a.get("seed") {
        cfg.seed = Some(s.parse().map_err(|_| format!("bad seed '{}'", s))?);
    }
    if let Some(s) = a.get("strategy") {
        if a.has("exhaustive") {
            return Err("'--strategy' and '--exhaustive' cannot be combined".into());
        }
        (cfg.strategy, cfg.strategy_bound) = parse_strategy(s)?;
    }
    if a.has("exhaustive") {
        cfg.mode = Mode::PEx;
    }
    if let Some(n) = uint(a, "max-steps")? {
        cfg.max_unfair_steps = n;
        cfg.max_fair_steps = n.saturating_mul(10);
    }
    if let Some(n) = uint(a, "fair-max-steps")? {
        cfg.max_fair_steps = n;
        cfg.user_set_max_fair_steps = true;
    }
    if cfg.max_fair_steps < cfg.max_unfair_steps {
        return Err("'--fair-max-steps' must be at least '--max-steps'".into());
    }
    cfg.fail_on_max_steps = a.has("fail-on-max-steps");
    if let Some(t) = a.get("timeline") {
        parse_timeline(t, &mut cfg.timeline)?;
    }
    cfg.timeline.payload = a.has("timeline-payload");
    if let Some(l) = a.get("latency") {
        cfg.latency_profile = split_latency(l).1.map(str::to_string);
    }
    cfg.timed = a.has("timed");
    if let Some(o) = a.get("out") {
        cfg.output_path = o.to_string();
    }
    cfg.verbose = a.has("verbose");
    check::finish_options(cfg);
    Ok(patterns)
}

fn run_check(
    host: &mut dyn Host,
    mut cfg: CheckConfig,
    patterns: &[String],
    latency: Option<&str>,
) -> Result<i32, Exit> {
    if let Some(l) = latency {
        cfg.latency = Some(check::load_latency(host, split_latency(l).0)?);
    }
    check::resolve_sources(host, &mut cfg)?;
    host.out_line(&format!(".. Checking {}", cfg.project_name));
    let loaded = check::load(host, &cfg)?;
    let names: Vec<String> = loaded.analysis.program.test_cases().into_iter().map(|t| t.name).collect();
    let selected = select_tests(&names, patterns).map_err(|m| {
        error(host, &m);
        Exit(ERROR)
    })?;
    if selected.is_empty() {
        error(host, "the sources declare no test case");
        return Err(Exit(ERROR));
    }
    let mut code = OK;
    for name in selected {
        cfg.test_case_name = name;
        check::set_output_directory(host, &mut cfg)?;
        code = code.max(check::testing_process(host, &loaded, &cfg)?);
    }
    host.out_line(". Done");
    Ok(code)
}

const REPLAY_OPTS: &[Opt] = &[value("test", None), flag("verbose", Some('v'))];

fn replay(args: &[String], host: &mut dyn Host) -> i32 {
    let a = match parse_args(args, REPLAY_OPTS) {
        Ok(a) => a,
        Err(m) => return usage_error(host, &m, Some("replay")),
    };
    if a.help {
        host.write(Stream::Out, REPLAY_HELP, None);
        return OK;
    }
    let mut cfg = CheckConfig::default();
    let rest = match sources(&mut cfg, &a.positional) {
        Ok(r) => r,
        Err(m) => return usage_error(host, &m, Some("replay")),
    };
    let file = match rest.as_slice() {
        [f] if f.ends_with(".schedule") => f.clone(),
        [f] => return usage_error(host, &format!("'{}' is not a .schedule file", f), Some("replay")),
        [] => return usage_error(host, "no .schedule file given", Some("replay")),
        [_, extra, ..] => {
            return usage_error(
                host,
                &format!("unexpected argument '{}' (sources are .pproj or .p files)", extra),
                Some("replay"),
            )
        }
    };
    cfg.verbose = a.has("verbose");
    cfg.strategy = StrategyKind::Replay;
    let mut run = |host: &mut dyn Host| -> Result<i32, Exit> {
        let trace = check::read_schedule(host, &file)?;
        check::resolve_sources(host, &mut cfg)?;
        host.out_line(&format!("... Replaying {}", file));
        let loaded = check::load(host, &cfg)?;
        let all: Vec<String> = loaded.analysis.program.test_cases().into_iter().map(|t| t.name).collect();
        let name = match (a.get("test"), &trace.test_name) {
            (Some(t), _) => t.to_string(),
            (None, Some(t)) => t.clone(),
            (None, None) if all.len() == 1 => all[0].clone(),
            (None, None) => {
                error(
                    host,
                    &format!("the schedule names no test case; pick one with --test. Test cases: {}", all.join(", ")),
                );
                return Err(Exit(ERROR));
            }
        };
        if !all.contains(&name) {
            error(host, &format!("no test case '{}'. Test cases: {}", name, all.join(", ")));
            return Err(Exit(ERROR));
        }
        let tc = check::select_test_case(host, &loaded, &name)?;
        let bugs = check::replay(host, &loaded, &tc, &mut cfg, trace)?;
        host.out_line("");
        Ok(if bugs > 0 { BUG } else { OK })
    };
    run(host).unwrap_or(ERROR)
}

fn tests(args: &[String], host: &mut dyn Host) -> i32 {
    let a = match parse_args(args, &[]) {
        Ok(a) => a,
        Err(m) => return usage_error(host, &m, Some("tests")),
    };
    if a.help {
        host.write(Stream::Out, TESTS_HELP, None);
        return OK;
    }
    let mut cfg = CheckConfig::default();
    match sources(&mut cfg, &a.positional) {
        Ok(rest) if rest.is_empty() => {}
        Ok(rest) => {
            return usage_error(
                host,
                &format!("unexpected argument '{}' (sources are .pproj or .p files)", rest[0]),
                Some("tests"),
            )
        }
        Err(m) => return usage_error(host, &m, Some("tests")),
    }
    // progress messages are shown only if loading fails
    let mut quiet = Buffered::new(host);
    let loaded = check::resolve_sources(&mut quiet, &mut cfg).and_then(|_| check::load(&mut quiet, &cfg));
    let Buffered { out, .. } = quiet;
    match loaded {
        Ok(loaded) => {
            for t in loaded.analysis.program.test_cases() {
                host.out_line(&t.name);
            }
            OK
        }
        Err(_) => {
            for (stream, text, color) in out {
                host.write(stream, &text, color);
            }
            ERROR
        }
    }
}

fn observe(args: &[String], host: &mut dyn Host) -> i32 {
    // PObserve's exit codes: 2 a spec violation, 1, 3 and 4 errors
    use crate::driver::observe;
    match observe::run_named(args, host, Vec::new(), "prs observe") {
        observe::EXIT_SUCCESS => OK,
        observe::EXIT_ASSERTION => BUG,
        _ => ERROR,
    }
}

fn coverage(args: &[String], host: &mut dyn Host) -> i32 {
    if args.first().is_some_and(|a| a == "merge-observe") {
        return merge_observe(&args[1..], host);
    }
    let a = match parse_args(args, &[]) {
        Ok(a) => a,
        Err(m) => return usage_error(host, &m, Some("coverage")),
    };
    if a.help {
        host.write(Stream::Out, COVERAGE_HELP, None);
        return OK;
    }
    let dir = match a.positional.as_slice() {
        [m] if m == "merge" => host.cwd().to_string_lossy().to_string(),
        [m, d] if m == "merge" => d.clone(),
        [m, _, extra, ..] if m == "merge" => {
            return usage_error(host, &format!("unexpected argument '{}'", extra), Some("coverage"))
        }
        [] => return usage_error(host, "missing subcommand: merge", Some("coverage")),
        [other, ..] => return usage_error(host, &format!("unknown subcommand '{}'", other), Some("coverage")),
    };
    let full = full_path(&host.cwd(), Path::new(&dir));
    if !host.is_dir(&full) {
        error(host, &format!("directory not found: {}", dir));
        return ERROR;
    }
    let report = crate::driver::scenario::merge_dir(host, &full);
    host.write(Stream::Out, &report, None);
    OK
}

/// Merge portable observed reports without loading or executing P sources.
fn merge_observe(args: &[String], host: &mut dyn Host) -> i32 {
    use crate::checker::observe_report::ObserveCoverageReport;
    let a = match parse_args(args, &[flag("stdin", None), flag("json", None), value("json-report", None)]) {
        Ok(a) => a,
        Err(e) => return usage_error(host, &e, Some("coverage")),
    };
    if a.help {
        host.write(Stream::Out, "Usage: prs coverage merge-observe [REPORT.json ...] [--stdin] [--json-report OUT] [--json]\n--stdin reads a JSON array of full observation or merged reports. No P sources are executed.\n", None);
        return OK;
    }
    if a.positional.is_empty() && !a.has("stdin") {
        return usage_error(host, "provide report files or --stdin", Some("coverage"));
    }
    let run = |host: &mut dyn Host| -> Result<(), String> {
        let mut merged = ObserveCoverageReport::default();
        for file in &a.positional {
            let text =
                host.read_to_string(&full_path(&host.cwd(), Path::new(file))).map_err(|e| format!("{file}: {e}"))?;
            merged.merge(&ObserveCoverageReport::from_json_str(&text).map_err(|e| format!("{file}: {e}"))?)?;
        }
        if a.has("stdin") {
            let mut text = String::new();
            while let Some(line) = host.read_stdin_line().map_err(|e| e.to_string())? {
                text.push_str(&line);
                text.push('\n');
            }
            let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
            if !value.is_array() {
                return Err("--stdin expects a JSON array of full reports".into());
            }
            merged.merge(&ObserveCoverageReport::from_json(&value)?)?;
        }
        let json = serde_json::to_string_pretty(&merged.to_json()).map_err(|e| e.to_string())? + "\n";
        if let Some(path) = a.get("json-report") {
            host.write_file(&full_path(&host.cwd(), Path::new(path)), &json).map_err(|e| e.to_string())?;
        }
        host.write(Stream::Out, &if a.has("json") { json } else { merged.render_text() }, None);
        Ok(())
    };
    match run(host) {
        Ok(()) => OK,
        Err(e) => {
            error(host, &e);
            ERROR
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn globs() {
        assert!(glob_match("tc*", "tcA"));
        assert!(glob_match("*B", "tcB"));
        assert!(glob_match("t?A", "tcA"));
        assert!(glob_match("tcA", "tcA"));
        assert!(!glob_match("tc", "tcA"));
        assert!(glob_match("*a*b*", "xxaYYbzz"));
        assert!(!glob_match("*a*b", "xxaYYbzz"));
    }

    #[test]
    fn durations() {
        assert_eq!(parse_duration("90"), Some(90));
        assert_eq!(parse_duration("90s"), Some(90));
        assert_eq!(parse_duration("5m"), Some(300));
        assert_eq!(parse_duration("1h"), Some(3600));
        assert_eq!(parse_duration("0"), None);
        assert_eq!(parse_duration("1.5m"), None);
        assert_eq!(parse_duration("5d"), None);
    }

    #[test]
    fn strategies() {
        assert_eq!(parse_strategy("pct:3"), Ok((StrategyKind::Pct, 3)));
        assert_eq!(parse_strategy("feedback-pct:10"), Ok((StrategyKind::FeedbackPct, 10)));
        assert_eq!(parse_strategy("pos"), Ok((StrategyKind::Pos, 0)));
        assert!(parse_strategy("pct").is_err());
        assert!(parse_strategy("pos:2").is_err());
        assert!(parse_strategy("dfs").is_err());
    }

    #[test]
    fn latency_profiles() {
        assert_eq!(split_latency("l.toml"), ("l.toml", None));
        assert_eq!(split_latency("l.toml:fast"), ("l.toml", Some("fast")));
        assert_eq!(split_latency("C:\\x\\l.toml"), ("C:\\x\\l.toml", None));
        assert_eq!(split_latency("C:\\x\\l.toml:fast"), ("C:\\x\\l.toml", Some("fast")));
    }
}
