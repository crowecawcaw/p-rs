//! `p observe`: upstream's PObserve command line (`pobserve.PObserve`).
//!
//! Checks recorded logs of a running system against a spec monitor. Options, console
//! output, report files and exit codes follow PObserve, with one change in how the spec is
//! found: PObserve loads specs and parsers from jars produced by `p compile --mode pobserve`,
//! while `p observe` type-checks the P sources directly. The spec comes from the `.pproj` in
//! the current folder, or from `--pproj` / `--pfiles` (the options `p check` takes), and
//! there is no `--jars` option.
//!
//! Parsers are Rust values implementing [`Parser`]. Two are built in ([`JsonLinesParser`]
//! for text logs, [`PTraceParser`] for P's JSON traces); programs embedding the crate pass
//! their own to [`run_with`]. The checking itself is [`prs_checker::observe`].

use std::path::{Path, PathBuf};
use std::rc::Rc;

use prs_checker::javafmt;
use prs_checker::observe::{
    split_entries, ErrorKind, Metrics, ObserveError, ObserveOptions, ObservedEvent, Parser, Report, Spec, Timestamp,
};
use prs_checker::value::{MachineRef, NamedTuple, SlotMap, SlotSet, Value};
use prs_sema::{MachineKind, PType, Program};
use serde_json::Value as Json;

use crate::host::{full_path, Color, Host, HostExt, Stream};
use crate::{pproj, project, write_output, Exit};

/// `PObserveExitStatus`.
pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_CMDLINE_ERROR: i32 = 1;
pub const EXIT_ASSERTION: i32 = 2;
pub const EXIT_INTERNAL_ERROR: i32 = 3;
pub const EXIT_PARSE_LOG_ERROR: i32 = 4;

/// Runs `p observe` with the built-in parsers.
pub fn run(args: &[String], host: &mut dyn Host) -> i32 {
    run_with(args, host, Vec::new())
}

/// Runs `p observe` with additional parsers (selected by `--parser <name>`). When `--parser`
/// is not given and exactly one extra parser is passed, that one is used.
pub fn run_with(args: &[String], host: &mut dyn Host, parsers: Vec<Box<dyn Parser>>) -> i32 {
    run_named(args, host, parsers, "p observe")
}

/// [`run_with`], naming the command `command` in its usage text.
pub fn run_named(args: &[String], host: &mut dyn Host, parsers: Vec<Box<dyn Parser>>, command: &str) -> i32 {
    let mut parsers = parsers;
    let usage = USAGE.replacen("p observe", command, 1);
    let cmdline_error = |host: &mut dyn Host, m: &str| cmdline_error(host, m, &usage);
    let cfg = match parse_args(args, host) {
        Ok(Some(c)) => c,
        Ok(None) => {
            host.write(Stream::Out, &usage, None);
            return EXIT_SUCCESS;
        }
        Err(m) => return cmdline_error(host, &m),
    };
    let loaded = match load_project(&cfg, host) {
        Ok(l) => l,
        Err(m) => return cmdline_error(host, &m),
    };
    let prog = &loaded.program;
    let parser_idx = match select_parser(&cfg, &mut parsers) {
        Ok(i) => i,
        Err(m) => return cmdline_error(host, &m),
    };
    if let Some(c) = &cfg.parser_configuration {
        if let Err(m) = parsers[parser_idx].set_configuration(c) {
            return cmdline_error(host, &m);
        }
    }
    let mut specs = match select_specs(prog, &cfg, &loaded.java_package) {
        Ok(s) => s,
        Err(m) => return cmdline_error(host, &m),
    };
    for spec in &mut specs {
        spec.java_package = loaded.java_package.clone();
    }
    if !cfg.filters.is_empty() {
        return cmdline_error(host, "FILTER not found: event filters (--filters) are not supported by p observe.");
    }
    let output_dir = cfg.output_dir.clone().unwrap_or_else(|| default_output_dir(host));
    let out_abs = java_absolute(&host.cwd(), &output_dir);
    if host.create_dir_all(Path::new(&out_abs)).is_err() {
        return cmdline_error(host, &format!("Failed to create output directory ({}).", output_dir));
    }
    let parser = parsers[parser_idx].as_ref();
    let spec_names: Vec<String> = specs.iter().map(|s| s.name().to_string()).collect();

    // validation succeeded: print the configuration
    host.out_line("Current PObserve configuration:");
    let mut c = String::from("PObserveConfig {\n");
    let mut line = |k: &str, v: &str| c.push_str(&format!("  {:<20} = {}\n", k, v));
    line("outputDir", &output_dir);
    line("inputKind", cfg.input_kind.name());
    line(loaded.source_option, &loaded.source_value);
    line("parserName", cfg.parser.as_deref().unwrap_or("null"));
    line("specificationName", &spec_names.join(", "));
    line("replayWindowSize", &cfg.replay_window.to_string());
    line("logDelimiter", &parser.log_delimiter().replace('\n', "\\n"));
    c.push('}');
    host.out_line(&c);

    // read the logs
    let opts = ObserveOptions {
        replay_window: cfg.replay_window,
        assume_sorted: cfg.assume_sorted,
        keys: cfg.keys.clone(),
        check_hot: cfg.check_hot,
    };
    let mut inputs = Vec::new();
    let mut read_error = None;
    for f in &cfg.log_files {
        let text = match host.read_to_string(f) {
            Ok(t) => t,
            Err(e) => {
                read_error = Some(format!("{} ({})", f.display(), e));
                break;
            }
        };
        match cfg.input_kind {
            InputKind::Text => inputs.push(split_entries(&text, &parser.log_delimiter())),
            InputKind::Json => match ptrace_entries(&text) {
                Ok(v) => inputs.push(v),
                Err(e) => {
                    read_error = Some(e);
                    break;
                }
            },
        }
    }

    // check each spec; with several, each writes its files to a folder of its own
    let several = specs.len() > 1;
    let mut reports = Vec::new();
    for spec in &specs {
        let report = match &read_error {
            // upstream: the reader's exception escapes the stream and ends the run
            Some(m) => Report {
                errors: vec![ObserveError {
                    cause: None,
                    kind: ErrorKind::Unknown,
                    detail: format!("java.lang.RuntimeException: {}", m),
                    message: m.clone(),
                    replay: None,
                }],
                metrics: Metrics { unknown_errors: 1, ..Default::default() },
                log: Vec::new(),
            },
            None => {
                let clock_host: &dyn Host = host;
                let mut clock = || clock_host.now();
                spec.observe(&inputs, parser, &opts, &mut clock)
            }
        };
        let spec_out = if several { join(&out_abs, spec.name()) } else { out_abs.clone() };
        if several {
            host.out_line(&format!("Checking spec {}", spec.name()));
            if host.create_dir_all(Path::new(&spec_out)).is_err() {
                return cmdline_error(host, &format!("Failed to create output directory ({}).", spec_out));
            }
        }
        for l in &report.log {
            host.out_colored_line(l, Color::Red);
        }
        if let Err(Exit(code)) =
            emit_report(host, spec, &report, &spec_out).and_then(|()| emit_metrics(host, &report.metrics, &spec_out))
        {
            return code;
        }
        reports.push(report);
        if read_error.is_some() {
            break;
        }
    }
    let n: u64 = reports.iter().map(|r| r.error_count()).sum();
    let all = Report { errors: reports.iter().flat_map(|r| r.errors.iter().cloned()).collect(), ..Default::default() };
    let code = exit_status(&all);
    if let Some(path) = &cfg.json_report {
        let text = json_report(&specs, &reports, code);
        if let Err(Exit(c)) = write_output(host, path, &text) {
            return c;
        }
    }
    if n > 0 {
        host.out_colored_line(&format!("PObserve run completed with {} error(s)", n), Color::Red);
        host.out_colored_line(&format!("Run details and replay event logs can be found in {}", out_abs), Color::Red);
    } else {
        host.out_line(&format!("PObserve run completed with {} error(s)", n));
        host.out_line(&format!("Run details can be found in {}", out_abs));
    }
    code
}

/// The machine-readable result written by `--json-report` (a prs extension): per spec, the
/// errors with the key and the log entries of the events leading up to each one.
fn json_report(specs: &[Spec], reports: &[Report], exit_code: i32) -> String {
    let specs: Vec<Json> = specs
        .iter()
        .zip(reports)
        .map(|(spec, r)| {
            let errors: Vec<Json> = r
                .errors
                .iter()
                .map(|e| {
                    let kind = match e.kind {
                        ErrorKind::SpecViolation => "spec_violation",
                        ErrorKind::OutOfOrder => "out_of_order",
                        ErrorKind::Parser => "parser",
                        ErrorKind::Unknown => "unknown",
                    };
                    let (key, events) = match &e.replay {
                        Some(rp) => (
                            Json::from(rp.key.clone()),
                            rp.events
                                .iter()
                                .map(|ev| {
                                    serde_json::json!({
                                        "key": ev.key,
                                        "timestamp": ev.timestamp.to_iso(),
                                        "event": spec.prog.event(ev.event).name,
                                        "log_line": ev.log_line,
                                    })
                                })
                                .collect(),
                        ),
                        None => (Json::Null, Vec::new()),
                    };
                    serde_json::json!({
                        "kind": kind,
                        "key": key,
                        "message": e.message,
                        "cause": e.cause,
                        "events": events,
                    })
                })
                .collect();
            serde_json::json!({
                "spec": spec.name(),
                "events_read": r.metrics.events_read,
                "partition_keys": r.metrics.partition_keys,
                "verified_keys": r.metrics.verified_keys,
                "errors": errors,
            })
        })
        .collect();
    let mut text = serde_json::to_string_pretty(&serde_json::json!({ "exit_code": exit_code, "specs": specs }))
        .unwrap_or_default();
    text.push('\n');
    text
}

/// `TrackErrors.getExitStatus`.
pub fn exit_status(report: &Report) -> i32 {
    let e = &report.errors;
    if e.is_empty() {
        EXIT_SUCCESS
    } else if e.iter().any(|x| !matches!(x.kind, ErrorKind::SpecViolation | ErrorKind::Parser)) {
        EXIT_INTERNAL_ERROR
    } else if e.iter().any(|x| x.kind == ErrorKind::SpecViolation) {
        EXIT_ASSERTION
    } else {
        EXIT_PARSE_LOG_ERROR
    }
}

fn cmdline_error(host: &mut dyn Host, msg: &str, usage: &str) -> i32 {
    host.out_colored_line("Failed parsing ::", Color::Red);
    host.out_colored_line(msg, Color::Red);
    host.write(Stream::Out, usage, None);
    EXIT_CMDLINE_ERROR
}

// ---- command line --------------------------------------------------------------------------

/// The JCommander usage text, with `--pproj`/`--pfiles` in place of `--jars`.
pub const USAGE: &str = "Usage: p observe [options]
  Options:
    --all-specs
      Check every spec in the P sources (prs extension)
      Default: false
    --assumeInputsAreSorted
      Assume that the input files are already sorted
      Default: false
    --check-hot
      Report a key whose spec is in a hot state after its last event (prs\x20
      extension)
      Default: false
    --exclude-specs
      Specs --all-specs leaves out (prs extension)
    --filters, -f
      pobserve event filter exact class names
    --help, -h
      Lists all supported options and commands
    --host
      Host to bind to in socket mode (default: localhost)
      Default: localhost
    --inputkind, -ik
      Input log kind {TEXT or JSON}
      Possible Values: [JSON, TEXT, SERIALIZEDEVENTS]
    --json-report
      Write the result as JSON to this file (prs extension)
    --keyListFile
      Path to file containing list of keys(one key per line) to be filtered\x20
      and verified
    --keys
      Keys to be filtered and verified
    --logs, -l
      log file to read or directory with multiple log files
    --outputDir, -o
      Output directory
    --parser, -p
      parser supplier, exact class name
    --parserConfiguration, -pc
      string to be passed to parser
    --pfiles, -pf
      List of P files with the specification (instead of a .pproj)
    --port
      Port to listen on in socket mode
      Default: 9876
    --pproj, -pp
      P project file with the specification (*.pproj). If this option is not\x20
      passed, the *.pproj in the current folder is used
    --replaySize
      Replay window size
    --socket-mode
      Enable socket command API mode
      Default: false
    --spec
      Names of the P specifications to be checked on the logs (one, as\x20
      upstream, or several)

";

/// `SourceInputKind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputKind {
    Text,
    Json,
}

impl InputKind {
    fn name(self) -> &'static str {
        match self {
            InputKind::Text => "TEXT",
            InputKind::Json => "JSON",
        }
    }
}

/// The validated command line (`PObserveConfig`).
#[derive(Debug, Clone)]
pub struct ObserveConfig {
    /// `--spec`: one name, as upstream, or several (a prs extension).
    pub specs: Vec<String>,
    /// `--all-specs`: every spec machine (a prs extension).
    pub all_specs: bool,
    /// `--exclude-specs`: specs `--all-specs` leaves out.
    pub exclude_specs: Vec<String>,
    /// `--json-report`: file to write the result to as JSON.
    pub json_report: Option<String>,
    /// `--check-hot`: report specs left in a hot state.
    pub check_hot: bool,
    pub parser: Option<String>,
    pub parser_configuration: Option<String>,
    pub filters: Vec<String>,
    pub log_files: Vec<PathBuf>,
    pub input_kind: InputKind,
    /// `--outputDir` as given.
    pub output_dir: Option<String>,
    pub replay_window: usize,
    pub assume_sorted: bool,
    pub keys: Vec<String>,
    pub pproj: Option<String>,
    pub pfiles: Vec<String>,
}

#[derive(Clone, Copy, PartialEq)]
enum Arity {
    Flag,
    One,
    Variable,
    /// Like `Variable`, but stops at any token starting with `-`, so `--spec A --jars x`
    /// still reports `--jars` as upstream does.
    Names,
}

const OPTIONS: &[(&[&str], Arity)] = &[
    (&["--help", "-h"], Arity::Flag),
    (&["--spec"], Arity::Names),
    (&["--all-specs"], Arity::Flag),
    (&["--exclude-specs"], Arity::Names),
    (&["--json-report"], Arity::One),
    (&["--check-hot"], Arity::Flag),
    (&["--parser", "-p"], Arity::One),
    (&["--parserConfiguration", "-pc"], Arity::One),
    (&["--filters", "-f"], Arity::Variable),
    (&["--logs", "-l"], Arity::One),
    (&["--socket-mode"], Arity::Flag),
    (&["--port"], Arity::One),
    (&["--host"], Arity::One),
    (&["--inputkind", "-ik"], Arity::One),
    (&["--outputDir", "-o"], Arity::One),
    (&["--replaySize"], Arity::One),
    (&["--assumeInputsAreSorted"], Arity::Flag),
    (&["--keys"], Arity::Variable),
    (&["--keyListFile"], Arity::One),
    (&["--pproj", "-pp"], Arity::One),
    (&["--pfiles", "-pf"], Arity::Variable),
];

fn option(tok: &str) -> Option<(&'static str, Arity)> {
    OPTIONS.iter().find(|(names, _)| names.contains(&tok)).map(|(names, a)| (names[0], *a))
}

fn strip_dashes(name: &str) -> &str {
    name.trim_start_matches('-')
}

fn validate_classname(name: &str, value: &str) -> Result<(), String> {
    if value.chars().any(|c| !(c.is_ascii_alphanumeric() || c == '_')) {
        return Err(format!("The specified class name {} ({}) is invalid.", strip_dashes(name), value));
    }
    Ok(())
}

fn validate_file_path(host: &dyn Host, name: &str, value: &str) -> Result<PathBuf, String> {
    let p = full_path(&host.cwd(), Path::new(value));
    if !host.is_file(&p) && !host.is_dir(&p) {
        return Err(format!("The specified {}({}) does not exist.", strip_dashes(name), value));
    }
    if !host.is_file(&p) {
        return Err(format!("The specified {}({}) is not a file.", strip_dashes(name), value));
    }
    Ok(p)
}

/// Parses and validates the options as JCommander and the parameter validators do.
/// `Ok(None)` means help was requested.
pub fn parse_args(args: &[String], host: &mut dyn Host) -> Result<Option<ObserveConfig>, String> {
    let mut cfg = ObserveConfig {
        specs: Vec::new(),
        all_specs: false,
        exclude_specs: Vec::new(),
        json_report: None,
        check_hot: false,
        parser: None,
        parser_configuration: None,
        filters: Vec::new(),
        log_files: Vec::new(),
        input_kind: InputKind::Text,
        output_dir: None,
        replay_window: 100,
        assume_sorted: false,
        keys: Vec::new(),
        pproj: None,
        pfiles: Vec::new(),
    };
    let mut spec_given = false;
    let mut help = false;
    let mut socket_mode = false;
    let mut serialized = false;
    let mut seen: Vec<&str> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let tok = args[i].as_str();
        let Some((canon, arity)) = option(tok) else {
            return Err(format!(
                "Was passed main parameter '{}' but no main parameter was defined in your arg class",
                tok
            ));
        };
        i += 1;
        let mut values: Vec<String> = Vec::new();
        match arity {
            Arity::Flag => values.push("true".into()),
            Arity::One => {
                if i >= args.len() || option(&args[i]).is_some() {
                    return Err(format!("Expected a value after parameter {}", tok));
                }
                if seen.contains(&canon) {
                    return Err(format!("Can only specify option {} once.", canon));
                }
                values.push(args[i].clone());
                i += 1;
            }
            Arity::Variable | Arity::Names => {
                while i < args.len()
                    && option(&args[i]).is_none()
                    && !(arity == Arity::Names && !values.is_empty() && args[i].starts_with('-'))
                {
                    values.push(args[i].clone());
                    i += 1;
                }
                if values.is_empty() {
                    return Err(format!("Expected a value after parameter {}", tok));
                }
            }
        }
        seen.push(canon);
        for v in values {
            match canon {
                "--help" => help = true,
                "--spec" => {
                    validate_classname(tok, &v)?;
                    if !cfg.specs.contains(&v) {
                        cfg.specs.push(v);
                    }
                    spec_given = true;
                }
                "--all-specs" => cfg.all_specs = true,
                "--exclude-specs" => {
                    validate_classname(tok, &v)?;
                    cfg.exclude_specs.push(v);
                }
                "--json-report" => cfg.json_report = Some(v),
                "--check-hot" => cfg.check_hot = true,
                "--parser" => {
                    validate_classname(tok, &v)?;
                    cfg.parser = Some(v);
                }
                "--parserConfiguration" => {
                    if v.is_empty() {
                        return Err(format!("The specified {} () is invalid. (must not be empty)", strip_dashes(tok)));
                    }
                    cfg.parser_configuration = Some(v);
                }
                "--filters" => {
                    validate_classname(tok, &v)?;
                    cfg.filters.push(v);
                }
                "--logs" => {
                    let p = full_path(&host.cwd(), Path::new(&v));
                    if !host.is_file(&p) && !host.is_dir(&p) {
                        return Err(format!("The specified log location({}) does not exist.", v));
                    }
                    cfg.log_files = if host.is_dir(&p) {
                        host.read_dir(&p).unwrap_or_default().into_iter().filter(|f| host.is_file(f)).collect()
                    } else {
                        vec![p]
                    };
                }
                "--socket-mode" => socket_mode = true,
                "--port" => match v.parse::<i32>() {
                    Ok(n) if (1..=65535).contains(&n) => {}
                    Ok(_) => return Err("Port must be between 1 and 65535".into()),
                    Err(_) => return Err("Port must be a valid integer".into()),
                },
                "--host" => {
                    if v.trim().is_empty() {
                        return Err("Host cannot be empty".into());
                    }
                }
                "--inputkind" => match v.to_uppercase().as_str() {
                    "TEXT" => cfg.input_kind = InputKind::Text,
                    "JSON" => cfg.input_kind = InputKind::Json,
                    "SERIALIZEDEVENTS" => serialized = true,
                    _ => return Err(format!("The specified input kind ({}) is not valid.", v)),
                },
                "--outputDir" => {
                    let p = full_path(&host.cwd(), Path::new(&v));
                    if !host.is_dir(&p) && !host.is_file(&p) && host.create_dir_all(&p).is_err() {
                        return Err(format!("Failed to create output directory ({}).", v));
                    }
                    if !host.is_dir(&p) {
                        return Err(format!("The specified output directory ({}) is not a directory.", v));
                    }
                    cfg.output_dir = Some(v);
                }
                "--replaySize" => {
                    let bad =
                        || format!("The specified {} ({}) is not valid (Must be an integer > 0)", strip_dashes(tok), v);
                    if v.is_empty() || !v.chars().all(|c| c.is_ascii_digit()) {
                        return Err(bad());
                    }
                    match v.parse::<i32>() {
                        Ok(n) if n >= 1 => cfg.replay_window = n as usize,
                        _ => return Err(bad()),
                    }
                }
                "--assumeInputsAreSorted" => cfg.assume_sorted = true,
                "--keys" => push_key(&mut cfg.keys, v),
                "--keyListFile" => {
                    let p = validate_file_path(host, tok, &v)?;
                    match host.read_to_string(&p) {
                        Ok(text) => {
                            for l in text.lines() {
                                push_key(&mut cfg.keys, l.trim().to_string());
                            }
                        }
                        Err(e) => {
                            host.out_colored_line(
                                &format!("Exception occurred while reading keyListFile({}) ::", v),
                                Color::Red,
                            );
                            host.out_colored_line(&e.to_string(), Color::Red);
                        }
                    }
                }
                "--pproj" => cfg.pproj = Some(v),
                "--pfiles" => {
                    if !cfg.pfiles.contains(&v) {
                        cfg.pfiles.push(v);
                    }
                }
                _ => unreachable!(),
            }
        }
    }
    if help {
        return Ok(None);
    }
    if !spec_given && !cfg.all_specs {
        return Err("The following option is required: [--spec]".into());
    }
    if spec_given && cfg.all_specs {
        return Err("--spec and --all-specs cannot be combined.".into());
    }
    if !cfg.exclude_specs.is_empty() && !cfg.all_specs {
        return Err("--exclude-specs needs --all-specs.".into());
    }
    // PObserveConfig.validateAndLoadPObserveConfig
    if socket_mode {
        return Err("Socket mode (--socket-mode) is not supported by p observe.".into());
    }
    if cfg.log_files.is_empty() {
        return Err("No log files specified. Please provide log file or directory with --logs parameter.".into());
    }
    if serialized {
        return Err("Input kind SERIALIZEDEVENTS is not supported by p observe.".into());
    }
    Ok(Some(cfg))
}

fn push_key(keys: &mut Vec<String>, k: String) {
    if !keys.contains(&k) {
        keys.push(k);
    }
}

/// `File.getAbsolutePath`: joins without normalizing.
fn java_absolute(cwd: &Path, p: &str) -> String {
    if Path::new(p).is_absolute() {
        p.to_string()
    } else {
        format!("{}/{}", cwd.to_string_lossy().trim_end_matches('/'), p)
    }
}

/// `"PObserve-" + dd-MM-yyyy_HH:mm:ss` (UTC).
fn default_output_dir(host: &dyn Host) -> String {
    let t = host.unix_time().floor() as i64;
    let (y, mo, d) = civil_from_days(t.div_euclid(86400));
    let s = t.rem_euclid(86400);
    format!("PObserve-{:02}-{:02}-{:04}_{:02}:{:02}:{:02}", d, mo, y, s / 3600, s / 60 % 60, s % 60)
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (yoe + era * 400 + if m <= 2 { 1 } else { 0 }, m, d)
}

fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let mp = if m > 2 { m - 3 } else { m + 9 } as i64;
    let doy = (153 * mp + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// Parses an ISO-8601 instant (`2024-02-02T17:27:34.309Z`, offsets allowed).
pub fn parse_instant(s: &str) -> Option<Timestamp> {
    let s = s.trim();
    let b = s.as_bytes();
    let num = |a: usize, n: usize| -> Option<i64> {
        let t = s.get(a..a + n)?;
        t.chars().all(|c| c.is_ascii_digit()).then(|| t.parse().ok())?
    };
    let y = num(0, 4)?;
    if b.get(4) != Some(&b'-') || b.get(7) != Some(&b'-') || !matches!(b.get(10), Some(b'T') | Some(b't') | Some(b' '))
    {
        return None;
    }
    let (mo, d, h, mi) = (num(5, 2)?, num(8, 2)?, num(11, 2)?, num(14, 2)?);
    if b.get(13) != Some(&b':') {
        return None;
    }
    let (sec, mut i) = if b.get(16) == Some(&b':') { (num(17, 2)?, 19) } else { (0, 16) };
    let mut nanos: u32 = 0;
    if b.get(i) == Some(&b'.') || b.get(i) == Some(&b',') {
        i += 1;
        let start = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        let frac = &s[start..i];
        if frac.is_empty() {
            return None;
        }
        let mut f: String = frac.chars().take(9).collect();
        while f.len() < 9 {
            f.push('0');
        }
        nanos = f.parse().ok()?;
    }
    let offset = match &s[i..] {
        "Z" | "z" | "" => 0,
        o if (o.starts_with('+') || o.starts_with('-')) && o.len() >= 3 => {
            let sign = if o.starts_with('-') { -1 } else { 1 };
            let rest = o[1..].replace(':', "");
            let hh: i64 = rest.get(0..2)?.parse().ok()?;
            let mm: i64 = if rest.len() >= 4 { rest.get(2..4)?.parse().ok()? } else { 0 };
            sign * (hh * 3600 + mm * 60)
        }
        _ => return None,
    };
    if !(1..=12).contains(&mo) || !(1..=31).contains(&d) || h > 23 || mi > 59 || sec > 60 {
        return None;
    }
    let days = days_from_civil(y, mo as u32, d as u32);
    Some(Timestamp::new(days * 86400 + h * 3600 + mi * 60 + sec - offset, nanos))
}

// ---- the specification ---------------------------------------------------------------------

struct Loaded {
    program: Program,
    java_package: String,
    /// What the configuration printout lists in place of `supplierJars`.
    source_option: &'static str,
    source_value: String,
}

/// A host that hides console output (the project loader's progress messages) but keeps
/// error text so it can be reported as a command-line error.
struct Quiet<'a> {
    inner: &'a mut dyn Host,
    errors: String,
}

impl Host for Quiet<'_> {
    fn write(&mut self, stream: Stream, text: &str, _color: Option<Color>) {
        if stream == Stream::Err {
            self.errors.push_str(text);
        }
    }
    fn cwd(&self) -> PathBuf {
        self.inner.cwd()
    }
    fn read_to_string(&self, path: &Path) -> std::io::Result<String> {
        self.inner.read_to_string(path)
    }
    fn write_file(&mut self, path: &Path, contents: &str) -> std::io::Result<()> {
        self.inner.write_file(path, contents)
    }
    fn is_file(&self, path: &Path) -> bool {
        self.inner.is_file(path)
    }
    fn is_dir(&self, path: &Path) -> bool {
        self.inner.is_dir(path)
    }
    fn read_dir(&self, path: &Path) -> std::io::Result<Vec<PathBuf>> {
        self.inner.read_dir(path)
    }
    fn create_dir_all(&mut self, path: &Path) -> std::io::Result<()> {
        self.inner.create_dir_all(path)
    }
    fn remove_dir_all(&mut self, path: &Path) -> std::io::Result<()> {
        self.inner.remove_dir_all(path)
    }
    fn rename(&mut self, from: &Path, to: &Path) -> std::io::Result<()> {
        self.inner.rename(from, to)
    }
    fn now(&self) -> f64 {
        self.inner.now()
    }
    fn max_memory_gb(&self) -> f64 {
        self.inner.max_memory_gb()
    }
    fn random_seed(&mut self) -> u32 {
        self.inner.random_seed()
    }
    fn unix_time(&self) -> f64 {
        self.inner.unix_time()
    }
}

/// Finds the P sources (`.pproj` or `--pfiles`) and type-checks them.
fn load_project(cfg: &ObserveConfig, host: &mut dyn Host) -> Result<Loaded, String> {
    let mut quiet = Quiet { inner: host, errors: String::new() };
    let cwd = quiet.cwd();
    let mut project_name = project::DEFAULT_NAME.to_string();
    let mut package = None;
    let (files, root, option, value): (Vec<PathBuf>, PathBuf, &'static str, String) = if !cfg.pfiles.is_empty() {
        let mut files = Vec::new();
        for f in &cfg.pfiles {
            let p = full_path(&cwd, Path::new(f));
            if !quiet.is_file(&p) {
                return Err(format!("The specified pfiles({}) does not exist.", f));
            }
            files.push(p);
        }
        (files, cwd.clone(), "pfiles", format!("[{}]", cfg.pfiles.join(", ")))
    } else {
        let file = match cfg.pproj.clone().or_else(|| pproj::find_local_pproj(&mut quiet)) {
            Some(f) => f,
            None => {
                return Err(format!(
                    "No P project file (*.pproj) found in the current folder: {}. Pass the specification with --pproj or --pfiles.",
                    cwd.display()
                ))
            }
        };
        let proj = match pproj::parse(&mut quiet, &file) {
            Ok(p) => p,
            Err(_) => {
                let msg = quiet.errors.trim().trim_start_matches("Error: ").to_string();
                return Err(msg);
            }
        };
        project_name = proj.name.clone();
        package = proj.pobserve_package.clone();
        let files =
            proj.input_files.into_iter().filter(|f| f.to_string_lossy().to_lowercase().ends_with(".p")).collect();
        (files, proj.project_root, "pproj", file)
    };
    let art = project::read(&quiet, &project_name, &files, &root)?;
    let sm = art.source_map();
    let r = prs_sema::compile(&sm, &prs_sema::Options::default());
    let host = quiet.inner;
    if !project::report_result(host, &art, &sm, &r) {
        return Err("The P specification does not type-check.".into());
    }
    let program = r.map(|a| a.program).map_err(|_| "The P specification does not type-check.".to_string())?;
    Ok(Loaded {
        program,
        java_package: package.unwrap_or_else(|| format!("{}.pobserve", project_name)),
        source_option: option,
        source_value: value,
    })
}

/// The specs to check: each `--spec`, or with `--all-specs` every spec machine but the
/// excluded ones.
fn select_specs<'p>(prog: &'p Program, cfg: &ObserveConfig, package: &str) -> Result<Vec<Spec<'p>>, String> {
    if !cfg.all_specs {
        return cfg.specs.iter().map(|n| select_spec(prog, n, package)).collect();
    }
    let is_spec = |m: &&prs_sema::MachineDecl| m.kind == MachineKind::Spec;
    for x in &cfg.exclude_specs {
        if !prog.machines.iter().filter(is_spec).any(|m| &m.name == x) {
            return Err(format!("--exclude-specs: there is no spec named {}.", x));
        }
    }
    let specs: Vec<Spec> = prog
        .machines
        .iter()
        .enumerate()
        .filter(|(_, m)| m.kind == MachineKind::Spec && !cfg.exclude_specs.contains(&m.name))
        .map(|(i, _)| Spec::from_decl(prog, prs_sema::MachineId(i as u32)))
        .collect();
    if specs.is_empty() {
        return Err("--all-specs: the P sources declare no specs to check.".into());
    }
    Ok(specs)
}

/// `CreateInstanceFromJar.getInstance(SPECIFICATION_SUPPLIER, name)`: the spec machine whose
/// generated supplier class name (`<package>.PMachines$<Spec>$Supplier`) contains `name`.
fn select_spec<'p>(prog: &'p Program, name: &str, package: &str) -> Result<Spec<'p>, String> {
    let found: Vec<usize> = prog
        .machines
        .iter()
        .enumerate()
        .filter(|(_, m)| m.kind == MachineKind::Spec)
        .filter(|(_, m)| format!("{}.PMachines${}$Supplier", package, m.name).contains(name))
        .map(|(i, _)| i)
        .collect();
    match found.as_slice() {
        [] => Err("SPECIFICATION_SUPPLIER not found in the specified jar.".into()),
        [i] => Ok(Spec::from_decl(prog, prs_sema::MachineId(*i as u32))),
        many => Err(format!(
            "Found multiple SPECIFICATION_SUPPLIERs in the jar({})",
            many.iter().map(|_| "Supplier").collect::<Vec<_>>().join(", ")
        )),
    }
}

/// `CreateInstanceFromJar.getInstance(PARSER, name)` over the registered parsers, then the
/// built-in ones. Without `--parser`, a single registered parser is used, else the built-in
/// parser for the input kind.
fn select_parser(cfg: &ObserveConfig, parsers: &mut Vec<Box<dyn Parser>>) -> Result<usize, String> {
    let registered = parsers.len();
    parsers.push(Box::new(JsonLinesParser::default()));
    parsers.push(Box::new(PTraceParser));
    match &cfg.parser {
        Some(n) => {
            parsers.iter().position(|p| p.name() == n).ok_or_else(|| "PARSER not found in the specified jar.".into())
        }
        None => match registered {
            0 => Ok(match cfg.input_kind {
                InputKind::Text => registered,
                InputKind::Json => registered + 1,
            }),
            1 => Ok(0),
            _ => Err(format!(
                "Found multiple PARSERs in the jar({})",
                parsers[..registered].iter().map(|p| p.name().to_string()).collect::<Vec<_>>().join(", ")
            )),
        },
    }
}

// ---- reports -------------------------------------------------------------------------------

fn join(dir: &str, file: &str) -> String {
    format!("{}/{}", dir.trim_end_matches('/'), file)
}

fn replay_file_name(key: &str) -> String {
    let s: String =
        key.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect();
    let chars: Vec<char> = s.chars().collect();
    let name = if chars.len() > 200 {
        format!(
            "{}__{}",
            chars[..100].iter().collect::<String>(),
            chars[chars.len() - 100..].iter().collect::<String>()
        )
    } else {
        s
    };
    format!("replayEvents_{}.log", name)
}

/// `TrackErrors.emitErrorsReport`.
fn emit_report(host: &mut dyn Host, spec: &Spec, report: &Report, out: &str) -> Result<(), Exit> {
    const RULE: &str = "-------------------------------------";
    if report.errors.is_empty() {
        host.out_line(RULE);
        host.out_line("Success! No bugs found.");
        host.out_line(RULE);
        return Ok(());
    }
    let (mut parser_n, mut unknown_n) = (0, 0);
    for e in &report.errors {
        host.out_line(RULE);
        match e.kind {
            ErrorKind::SpecViolation => {
                host.out_colored_line(&format!("PObserve Spec Violation::\n{}", e.message), Color::Red);
                write_replay(host, spec, e, out)?;
            }
            ErrorKind::OutOfOrder => {
                host.out_colored_line("PObserve Event Out of Order Exception::", Color::Red);
                host.out_colored_line(&e.message, Color::Red);
                write_replay(host, spec, e, out)?;
            }
            ErrorKind::Parser => {
                host.out_colored_line("PObserve Parser Exception::", Color::Red);
                host.out_colored_line(&format!("Exception occurred while parsing log line: {}", e.message), Color::Red);
                write_exception(host, e, &join(out, &format!("ParserError_{}.txt", parser_n)))?;
                parser_n += 1;
            }
            ErrorKind::Unknown => {
                host.out_colored_line(&format!("Failed with an unknown exception:: {}", e.message), Color::Red);
                host.out_colored_line("Please report this issue to the P team", Color::Red);
                write_exception(host, e, &join(out, &format!("UnknownError_{}.txt", unknown_n)))?;
                unknown_n += 1;
            }
        }
        host.out_line(RULE);
    }
    Ok(())
}

fn write_exception(host: &mut dyn Host, e: &ObserveError, path: &str) -> Result<(), Exit> {
    host.out_line(&format!(".. writing error log into file {}", path));
    // upstream prints the Java stack trace; there is none here
    let text = format!("errorMessage={}\n\nerrorLogLine=\n{}", e.detail, e.message);
    write_output(host, path, &text)?;
    host.out_line(&format!("Exception details can be found in:\n\t{}", path));
    Ok(())
}

fn write_replay(host: &mut dyn Host, spec: &Spec, e: &ObserveError, out: &str) -> Result<(), Exit> {
    let Some(r) = &e.replay else { return Ok(()) };
    let path = join(out, &replay_file_name(&r.key));
    host.out_line(&format!(".. writing error log into file {}", path));
    let mut text = format!(
        "errorKey={}\nerrorTimeStamp={}\nerrorMessage={}\nreplayEvents=\n",
        r.key, r.error_timestamp, e.message
    );
    for ev in &r.events {
        text.push_str(&ev.to_java_string(spec));
        text.push('\n');
    }
    write_output(host, &path, &text)
}

/// `PObserveMetrics.outputMetricsSummary`.
fn emit_metrics(host: &mut dyn Host, m: &Metrics, out: &str) -> Result<(), Exit> {
    let mut s = String::new();
    let minutes = m.elapsed_ms as f32 / 60000f32;
    s.push_str(&format!("\nTotal time taken = {} min\n", javafmt::float_to_string(minutes)));
    let d1 = "-".repeat(105);
    let d2 = "-".repeat(102);
    s.push_str(&format!("\n{}\n", d1));
    s.push_str(&format!(
        "{:<23}  {:<23}  {:<23}  {:<23}\n",
        "Total Events Read", "Total Verified Events", "Total Verified Keys", "Total Partition Keys"
    ));
    s.push_str(&format!("{}\n", d1));
    s.push_str(&format!(
        "{:<23}  {:<23}  {:<23}  {:<23}\n\n",
        m.events_read,
        m.verified_events(),
        m.verified_keys,
        m.partition_keys
    ));
    s.push_str(&format!("\n{}\n", d2));
    s.push_str(&format!(
        "{:<23}  {:<23}  {:<28}  {:<23}\n",
        "Total Parser Errors", "Total Spec Errors", "Total Out of Order Errors", "Total Unknown Errors"
    ));
    s.push_str(&format!("{}\n", d2));
    // upstream prints the unknown-error count in the parser-error column
    s.push_str(&format!(
        "{:<23}  {:<23}  {:<28}  {:<23}\n\n",
        m.unknown_errors, m.spec_errors, m.out_of_order_errors, m.unknown_errors
    ));
    host.out_line(&s);
    let mut f = s.clone();
    let d3 = "-".repeat(103);
    f.push_str(&format!("{}\n", d3));
    f.push_str(&format!(
        "{:<30}  {:<20}  {:<20}  {:<20}\n",
        "Event Type", "Total Verified", "Avg Exec Time (ms)", "Max Exec Time (ms)"
    ));
    f.push_str(&format!("{}\n", d3));
    for e in &m.events {
        f.push_str(&format!(
            "{:<30}  {:<20}  {:<20}  {:<20}\n",
            e.name,
            e.verified,
            java_percent_f(e.avg_ms()),
            e.max_ms
        ));
    }
    write_output(host, &join(out, "PObserveMetrics.txt"), &f)
}

/// `String.format("%f", float)`: six decimals, rounded half-up from the float's decimal form.
fn java_percent_f(v: f32) -> String {
    let d: f64 = javafmt::float_to_string(v).parse().unwrap_or(0.0);
    let scaled = (d * 1e6 + 0.5).floor() / 1e6;
    format!("{:.6}", scaled)
}

// ---- JSON payloads -------------------------------------------------------------------------

/// Converts JSON to a P value of type `ty`.
///
/// Integers, floats, booleans and strings map directly; enums accept the element name or
/// value; tuples accept an array or an object keyed by field name (or index); sequences and
/// sets take arrays; maps take an object (keys parsed by the key type) or an array of
/// `[key, value]` pairs; machines take their id number or `Name(id)`; events their name.
pub fn value_from_json(prog: &Program, j: &Json, ty: &PType) -> Result<Value, String> {
    let bad = || format!("cannot read {} as {}", j, type_name(prog, ty));
    Ok(match ty {
        PType::TypeDef(t) => return value_from_json(prog, j, &prog.typedefs[t.idx()].ty),
        _ if j.is_null() && !matches!(ty, PType::Bool | PType::Int | PType::Float | PType::Enum(_)) => Value::Null,
        PType::Bool => Value::Bool(j.as_bool().ok_or_else(bad)?),
        PType::Int => Value::Int(json_int(j).ok_or_else(bad)?),
        PType::Float => Value::Float(j.as_f64().ok_or_else(bad)? as f32 as f64),
        PType::String => Value::str(j.as_str().ok_or_else(bad)?),
        PType::Enum(e) => {
            let en = &prog.enums[e.idx()];
            match j {
                Json::String(s) => Value::Int(en.elems.iter().find(|x| &x.name == s).ok_or_else(bad)?.value),
                _ => {
                    let v = json_int(j).ok_or_else(bad)?;
                    en.elems.iter().find(|x| x.value == v).ok_or_else(bad)?;
                    Value::Int(v)
                }
            }
        }
        PType::Machine | PType::Permission(_) => {
            let id = match j {
                Json::String(s) => {
                    let inner = s.rsplit_once('(').map(|(_, r)| r.trim_end_matches(')')).unwrap_or(s);
                    inner.trim().parse::<u64>().map_err(|_| bad())?
                }
                _ => j.as_u64().ok_or_else(bad)?,
            };
            let name = match j {
                Json::String(s) => s.clone(),
                _ => id.to_string(),
            };
            Value::Machine(Rc::new(MachineRef { id, name: name.into(), permissions: Rc::from(Vec::new()) }))
        }
        PType::Event => {
            let s = j.as_str().ok_or_else(bad)?;
            let s = s.rsplit('.').next().unwrap_or(s);
            let id = prog.events.iter().position(|e| e.name == s).ok_or_else(bad)?;
            Value::Event(prs_sema::EventId(id as u32))
        }
        PType::Tuple(ts) => {
            let items: Vec<&Json> = match j {
                Json::Array(a) => a.iter().collect(),
                Json::Object(o) => (0..ts.len()).map(|i| o.get(&i.to_string()).unwrap_or(&Json::Null)).collect(),
                _ => return Err(bad()),
            };
            if items.len() != ts.len() {
                return Err(bad());
            }
            Value::Tuple(Rc::new(
                items.iter().zip(ts).map(|(v, t)| value_from_json(prog, v, t)).collect::<Result<_, _>>()?,
            ))
        }
        PType::NamedTuple(fs) => {
            let mut values = Vec::new();
            for (i, (n, t)) in fs.iter().enumerate() {
                let v = match j {
                    Json::Object(o) => o.get(n).ok_or_else(|| format!("missing field '{}' in {}", n, j))?,
                    Json::Array(a) if a.len() == fs.len() => &a[i],
                    _ => return Err(bad()),
                };
                values.push(value_from_json(prog, v, t)?);
            }
            let names: Vec<String> = fs.iter().map(|(n, _)| n.clone()).collect();
            Value::NamedTuple(Rc::new(NamedTuple { names: names.into(), values }))
        }
        PType::Seq(t) => {
            let a = j.as_array().ok_or_else(bad)?;
            Value::Seq(Rc::new(a.iter().map(|v| value_from_json(prog, v, t)).collect::<Result<_, _>>()?))
        }
        PType::Set(t) => {
            let a = j.as_array().ok_or_else(bad)?;
            let mut s = SlotSet::new();
            for v in a {
                s.insert(value_from_json(prog, v, t)?);
            }
            Value::Set(Rc::new(s))
        }
        PType::Map(kt, vt) => {
            let mut m = SlotMap::new();
            match j {
                Json::Object(o) => {
                    for (k, v) in o {
                        m.set(map_key(prog, k, kt)?, value_from_json(prog, v, vt)?);
                    }
                }
                Json::Array(a) => {
                    for pair in a {
                        match pair.as_array().map(|p| p.as_slice()) {
                            Some([k, v]) => m.set(value_from_json(prog, k, kt)?, value_from_json(prog, v, vt)?),
                            _ => return Err(bad()),
                        }
                    }
                }
                _ => return Err(bad()),
            }
            Value::Map(Rc::new(m))
        }
        PType::Any | PType::Data | PType::Null | PType::Error | PType::Foreign(_) => infer(j),
    })
}

fn json_int(j: &Json) -> Option<i64> {
    j.as_i64().or_else(|| j.as_f64().filter(|f| f.fract() == 0.0).map(|f| f as i64))
}

/// A JSON object key read as a map key of type `ty`.
fn map_key(prog: &Program, k: &str, ty: &PType) -> Result<Value, String> {
    let mut t = ty;
    while let PType::TypeDef(id) = t {
        t = &prog.typedefs[id.idx()].ty;
    }
    if matches!(t, PType::String) {
        // P traces quote string keys
        if k.len() >= 2 && k.starts_with('"') && k.ends_with('"') {
            return Ok(Value::str(&k[1..k.len() - 1].replace("\\\"", "\"")));
        }
        return Ok(Value::str(k));
    }
    let j: Json = serde_json::from_str(k).unwrap_or_else(|_| Json::String(k.to_string()));
    value_from_json(prog, &j, ty)
}

fn infer(j: &Json) -> Value {
    match j {
        Json::Null => Value::Null,
        Json::Bool(b) => Value::Bool(*b),
        Json::Number(n) => match n.as_i64() {
            Some(i) => Value::Int(i),
            None => Value::Float(n.as_f64().unwrap_or(0.0) as f32 as f64),
        },
        Json::String(s) => Value::str(s),
        Json::Array(a) => Value::Seq(Rc::new(a.iter().map(infer).collect())),
        Json::Object(o) => {
            let names: Vec<String> = o.keys().cloned().collect();
            Value::NamedTuple(Rc::new(NamedTuple { names: names.into(), values: o.values().map(infer).collect() }))
        }
    }
}

fn type_name(prog: &Program, ty: &PType) -> String {
    match ty {
        PType::Enum(e) => prog.enums[e.idx()].name.clone(),
        PType::TypeDef(t) => prog.typedefs[t.idx()].name.clone(),
        other => format!("{:?}", other).to_lowercase(),
    }
}

/// Builds the event `name` with a JSON payload; `Ok(None)` if the program declares no such
/// event.
pub fn event_from_json(
    spec: &Spec,
    key: String,
    timestamp: Timestamp,
    name: &str,
    payload: Option<&Json>,
    log_line: Option<&str>,
) -> Result<Option<ObservedEvent>, String> {
    let Some(event) = spec.event_id(name) else { return Ok(None) };
    let ty = spec.payload_type(event).clone();
    let payload = match payload {
        None | Some(Json::Null) => Value::Null,
        Some(p) => value_from_json(spec.prog, p, &ty)?,
    };
    Ok(Some(ObservedEvent {
        key,
        timestamp,
        event,
        payload,
        log_line: log_line.map(String::from),
        custom_payload: None,
    }))
}

// ---- built-in parsers ----------------------------------------------------------------------

/// One JSON object per log line:
/// `{"key": "acct-1", "timestamp": 1706894854309, "event": "eWithDrawReq", "payload": {...}}`.
///
/// `timestamp` is epoch milliseconds or an ISO-8601 string; `key` and `timestamp` are
/// optional (missing ones read as `""` and `0`). `--parserConfiguration` remaps the fields
/// with a JSON object of dotted paths, e.g. `{"event": "msg.type", "key": "msg.account"}`.
/// Blank lines and events the program does not declare are skipped.
#[derive(Debug, Clone)]
pub struct JsonLinesParser {
    pub key: String,
    pub timestamp: String,
    pub event: String,
    pub payload: String,
}

impl Default for JsonLinesParser {
    fn default() -> Self {
        JsonLinesParser {
            key: "key".into(),
            timestamp: "timestamp".into(),
            event: "event".into(),
            payload: "payload".into(),
        }
    }
}

fn lookup<'j>(j: &'j Json, path: &str) -> Option<&'j Json> {
    if path.is_empty() {
        return Some(j);
    }
    path.split('.').try_fold(j, |cur, part| cur.get(part))
}

fn key_string(j: Option<&Json>) -> String {
    match j {
        None | Some(Json::Null) => String::new(),
        Some(Json::String(s)) => s.clone(),
        Some(other) => other.to_string(),
    }
}

fn timestamp_of(j: Option<&Json>) -> Result<Timestamp, String> {
    match j {
        None | Some(Json::Null) => Ok(Timestamp::default()),
        Some(Json::String(s)) => match s.parse::<i64>() {
            Ok(ms) => Ok(Timestamp::from_millis(ms)),
            Err(_) => parse_instant(s).ok_or_else(|| format!("invalid timestamp '{}'", s)),
        },
        Some(n) => json_int(n).map(Timestamp::from_millis).ok_or_else(|| format!("invalid timestamp {}", n)),
    }
}

impl Parser for JsonLinesParser {
    fn name(&self) -> &str {
        "JsonLines"
    }

    fn set_configuration(&mut self, configuration: &str) -> Result<(), String> {
        let j: Json = serde_json::from_str(configuration)
            .map_err(|e| format!("The specified parserConfiguration ({}) is invalid. ({})", configuration, e))?;
        let Some(o) = j.as_object() else {
            return Err(format!(
                "The specified parserConfiguration ({}) is invalid. (expected a JSON object)",
                configuration
            ));
        };
        for (k, v) in o {
            let v = v.as_str().ok_or_else(|| format!("parserConfiguration: '{}' must be a string", k))?.to_string();
            match k.as_str() {
                "key" => self.key = v,
                "timestamp" => self.timestamp = v,
                "event" => self.event = v,
                "payload" => self.payload = v,
                other => return Err(format!("parserConfiguration: unknown field '{}'", other)),
            }
        }
        Ok(())
    }

    fn parse(&self, entry: &str, spec: &Spec) -> Result<Vec<ObservedEvent>, String> {
        if entry.is_empty() {
            return Ok(Vec::new());
        }
        let j: Json = serde_json::from_str(entry).map_err(|e| e.to_string())?;
        let name = lookup(&j, &self.event).and_then(|v| v.as_str()).ok_or("missing event name")?;
        let ts = timestamp_of(lookup(&j, &self.timestamp))?;
        let key = key_string(lookup(&j, &self.key));
        Ok(event_from_json(spec, key, ts, name, lookup(&j, &self.payload), Some(entry))?.into_iter().collect())
    }
}

/// The `SendEvent` and `Announce` entries of P's JSON traces (`p check` bug traces), for
/// `--inputkind JSON`. All events share one key and keep their trace order.
#[derive(Debug, Clone, Copy, Default)]
pub struct PTraceParser;

impl Parser for PTraceParser {
    fn name(&self) -> &str {
        "PTrace"
    }

    fn parse(&self, entry: &str, spec: &Spec) -> Result<Vec<ObservedEvent>, String> {
        let j: Json = serde_json::from_str(entry).map_err(|e| e.to_string())?;
        let details = j.get("details").ok_or("missing details")?;
        let name = details.get("event").and_then(|v| v.as_str()).ok_or("missing event name")?;
        let name = name.rsplit('.').next().unwrap_or(name);
        let log = details.get("log").and_then(|v| v.as_str());
        Ok(event_from_json(spec, String::new(), Timestamp::default(), name, details.get("payload"), log)?
            .into_iter()
            .collect())
    }
}

/// `PTraceJSONFileReader`: the `SendEvent`/`Announce` entries of a JSON trace, each as JSON
/// text.
fn ptrace_entries(text: &str) -> Result<Vec<String>, String> {
    let j: Json = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let arr = j.as_array().ok_or("expected a JSON array")?;
    let mut out = Vec::new();
    for e in arr {
        let ty = e.get("type").and_then(|t| t.as_str()).unwrap_or("");
        if ty.eq_ignore_ascii_case("sendevent") || ty.eq_ignore_ascii_case("announce") {
            out.push(e.to_string());
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instants() {
        assert_eq!(parse_instant("1970-01-01T00:00:00Z"), Some(Timestamp::new(0, 0)));
        assert_eq!(parse_instant("2024-02-02T17:27:34.309Z").map(|t| t.to_millis()), Some(1706894854309));
        assert_eq!(parse_instant("2024-02-02T18:27:34.309+01:00").map(|t| t.to_millis()), Some(1706894854309));
        assert_eq!(parse_instant("nope"), None);
    }

    #[test]
    fn dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(days_from_civil(2024, 2, 2), 19755);
        assert_eq!(civil_from_days(19755), (2024, 2, 2));
    }

    #[test]
    fn replay_names() {
        assert_eq!(replay_file_name("a/b c"), "replayEvents_a_b_c.log");
        let long = "x".repeat(250);
        assert_eq!(replay_file_name(&long).len(), "replayEvents_".len() + 202 + 4);
    }

    #[test]
    fn percent_f() {
        assert_eq!(java_percent_f(0.0), "0.000000");
        assert_eq!(java_percent_f(1.0 / 3.0), "0.333333");
        assert_eq!(java_percent_f(2.5), "2.500000");
    }
}
