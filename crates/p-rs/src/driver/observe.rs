//! Checking recorded logs against spec monitors on a [`Host`]: the work of `prs observe`
//! and `p observe`, which share upstream PObserve's command line (`pobserve.PObserve`).
//!
//! Options, console output, report files and exit codes follow PObserve, with one change in
//! how the spec is found: PObserve loads specs and parsers from jars produced by
//! `p compile --mode pobserve`, while this type-checks the P sources directly. The spec
//! comes from the `.pproj` in the current folder, or from `--pproj` / `--pfiles` (the
//! options `p check` takes), and there is no `--jars` option.
//!
//! Parsers are Rust values implementing [`Parser`]. Two are built in ([`JsonLinesParser`]
//! for text logs, [`PTraceParser`] for P's JSON traces); programs embedding the crate pass
//! their own to [`run`]. The checking itself is [`crate::checker::observe`].

use std::path::{Path, PathBuf};

use crate::checker::javafmt;
use crate::checker::observe::{
    split_entries, ErrorKind, Metrics, NoScheduling, ObserveError, ObserveOptions, ObserveSession, Parser, Report,
    Spec, UnknownEvents,
};
use crate::checker::observe_inputs::{add_log_paths, observe_live, LogFiles, SequenceMerger};
use crate::checker::observe_json::{ptrace_entries, JsonLinesParser, PTraceParser};
use crate::date::civil_from_days;
use crate::sema::{MachineKind, Program};
use serde_json::Value as Json;

use crate::driver::{pproj, project, write_output, Exit};
use crate::host::{full_path, relative_path, Buffered, Color, Host, HostExt, LiveInput, Stream};

/// `PObserveExitStatus`.
pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_CMDLINE_ERROR: i32 = 1;
pub const EXIT_ASSERTION: i32 = 2;
pub const EXIT_INTERNAL_ERROR: i32 = 3;
pub const EXIT_PARSE_LOG_ERROR: i32 = 4;

/// Runs `prs observe` with `args` (without the command name) and returns the PObserve exit
/// status (`EXIT_*`). `command` names the command in the usage text (`"prs observe"`).
///
/// `parsers` are parsers besides the built-in ones, selected by `--parser <name>`; when
/// `--parser` is not given and exactly one is passed, that one is used.
pub fn run(args: &[String], host: &mut dyn Host, parsers: Vec<Box<dyn Parser>>, command: &str) -> i32 {
    let mut parsers = parsers;
    let usage = USAGE.replacen("prs observe", command, 1);
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
    let live = !cfg.live.is_empty();
    if live && parser.log_delimiter().replace("\\n", "\n") != "\n" {
        return cmdline_error(host, "--stdin requires a newline-delimited parser.");
    }
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
    let cwd = host.cwd();
    let display = |f: &Path| if f.starts_with(&cwd) { relative_path(&cwd, f) } else { f.display().to_string() };
    let input_names: Vec<String> = if live {
        cfg.live
            .iter()
            .map(|v| if v == "-" { "stdin".to_string() } else { display(&full_path(&cwd, Path::new(v))) })
            .collect()
    } else {
        cfg.log_files.iter().map(|f| display(f)).collect()
    };
    let opts = ObserveOptions {
        replay_window: cfg.replay_window,
        assume_sorted: cfg.assume_sorted,
        keys: cfg.keys.clone(),
        check_hot: cfg.check_hot,
        coverage: cfg.coverage,
        strict: cfg.strict,
        unknown_events: cfg.unknown_events,
        input_names,
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
    let mut input_complete = false;
    let mut recorded = String::new();
    let mut reports = if live {
        let mut strategies: Vec<_> = specs.iter().map(|_| NoScheduling).collect();
        let mut sessions: Vec<_> = specs
            .iter()
            .zip(strategies.iter_mut())
            .map(|(spec, strategy)| ObserveSession::new(spec, opts.clone(), strategy, host.now()))
            .collect();
        let inputs: Vec<LiveInput> = cfg
            .live
            .iter()
            .map(|v| if v == "-" { LiveInput::Stdin } else { LiveInput::Path(full_path(&cwd, Path::new(v))) })
            .collect();
        let mut merger = cfg.sequence.as_deref().map(SequenceMerger::new);
        let mut source = host.open_live(&inputs);
        let mut record = |line: &str| {
            if cfg.record.is_some() {
                recorded.push_str(line);
                recorded.push('\n');
            }
        };
        let outcome = observe_live(
            &mut sessions,
            parser,
            source.as_mut(),
            &opts.input_names,
            merger.as_mut(),
            &mut record,
            &mut || host.now(),
        );
        drop(source);
        input_complete = outcome.complete;
        if !input_complete {
            host.out_line("Live observation stopped at the first error; reports cover the checked prefix.");
        }
        sessions
            .into_iter()
            .map(|s| if input_complete { s.finish(true, host.now()) } else { s.finish_after_failure(host.now()) })
            .collect()
    } else {
        Vec::new()
    };
    if let Some(path) = &cfg.record {
        if let Err(Exit(c)) = write_output(host, path, &recorded) {
            return c;
        }
    }
    for (index, spec) in specs.iter().enumerate() {
        let report = if live {
            reports[index].clone()
        } else {
            match &read_error {
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
                    coverage: None,
                    ..Default::default()
                },
                None => {
                    let clock_host: &dyn Host = host;
                    let mut clock = || clock_host.now();
                    spec.observe(&inputs, parser, &opts, &mut clock)
                }
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
        if let Some(coverage) = &report.coverage {
            host.out_line(&coverage.render_text());
        }
        if live {
            reports[index] = report;
        } else {
            reports.push(report);
        }
        if read_error.is_some() {
            break;
        }
    }
    let mut warned: Vec<&str> = Vec::new();
    for warning in reports.iter().flat_map(|r| r.warnings.iter()) {
        if !warned.contains(&warning.as_str()) {
            warned.push(warning);
            host.write(Stream::Err, &format!("warning: {}\n", warning), Some(Color::Yellow));
        }
    }
    let n: u64 = reports.iter().map(|r| r.error_count()).sum();
    let all = Report { errors: reports.iter().flat_map(|r| r.errors.iter().cloned()).collect(), ..Default::default() };
    let code = all.exit_status();
    if let Some(path) = &cfg.json_report {
        let mut text = json_report(&specs, &reports, code);
        if live {
            let mut json: Json = serde_json::from_str(&text).expect("valid report JSON");
            json["input_complete"] = Json::Bool(input_complete);
            json["stop_reason"] = Json::from(if input_complete { "eof" } else { "error" });
            text = serde_json::to_string_pretty(&json).unwrap() + "\n";
        }
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
                        ErrorKind::Validation => "validation",
                        ErrorKind::UnknownEvent => "unknown_event",
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
            let mut result = serde_json::json!({
                "spec": spec.name(),
                "events_read": r.metrics.events_read,
                "partition_keys": r.metrics.partition_keys,
                "verified_keys": r.metrics.verified_keys,
                "errors": errors,
            });
            if let Some(c) = &r.coverage {
                result["coverage"] = c.to_json();
            }
            if !r.unknown_events.is_empty() {
                result["unknown_events"] = r
                    .unknown_events
                    .iter()
                    .map(|u| {
                        serde_json::json!({
                            "event": u.name,
                            "count": u.count,
                            "location": u.location,
                            "suggestion": u.suggestion,
                        })
                    })
                    .collect();
            }
            if !r.warnings.is_empty() {
                result["warnings"] = Json::from(r.warnings.clone());
            }
            result
        })
        .collect();
    let mut text = serde_json::to_string_pretty(&serde_json::json!({ "exit_code": exit_code, "specs": specs }))
        .unwrap_or_default();
    text.push('\n');
    text
}

fn cmdline_error(host: &mut dyn Host, msg: &str, usage: &str) -> i32 {
    host.out_colored_line("Failed parsing ::", Color::Red);
    host.out_colored_line(msg, Color::Red);
    host.write(Stream::Out, usage, None);
    EXIT_CMDLINE_ERROR
}

// ---- command line --------------------------------------------------------------------------

/// PObserve's JCommander usage text, with `--pproj`/`--pfiles` in place of `--jars`; [`run`]
/// puts its command name in place of `prs observe`.
const USAGE: &str = "Usage: prs observe [options]
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
    --stdin
      Read a live newline-delimited TEXT stream from stdin (requires --fail-fast).
      Events must be ordered by timestamp per key. Cannot combine with --logs.
      Same as --live - (prs extension)
    --live
      Live newline-delimited TEXT streams (files, named pipes; - is stdin), read
      until each ends. Several need --sequence. Requires --fail-fast (prs extension)
    --sequence
      Check live lines in the order of this JSON field: a global sequence number
      starting at 1 with no gaps or duplicates (prs extension)
    --record
      Write the checked live lines, in check order, to this file (prs extension)
    --fail-fast
      Stop reading and exit on the first error (requires --stdin or --live).
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
    --strict
      Require at least one successfully checked relevant event per selected spec.
      Applied after key filtering, at EOF for live input; does not imply --check-hot.
    --unknown-events
      Event names no P source declares: ignore, warn or error (prs extension)
      Default: ignore
    --coverage
      Report observed monitor states and handlers on stdout and in --json-report
      (prs extension). Does not measure paths or scheduling coverage.
    --json-report
      Write the result as JSON to this file (prs extension)
    --keyListFile
      Path to file containing list of keys(one key per line) to be filtered\x20
      and verified
    --keys
      Keys to be filtered and verified
    --logs, -l
      log file to read or directory with multiple log files. Several files,
      directories and glob patterns (quoted, ** for subfolders) are a prs extension
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
    /// `--coverage`: report observed monitor state and handler coverage.
    pub coverage: bool,
    pub strict: bool,
    /// `--unknown-events`: event names no P source declares.
    pub unknown_events: UnknownEvents,
    /// `--live` (and `--stdin`, as `-`): live inputs, as given.
    pub live: Vec<String>,
    /// `--sequence`: the field ordering live lines.
    pub sequence: Option<String>,
    /// `--record`: file for the checked live lines.
    pub record: Option<String>,
    pub fail_fast: bool,
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
    (&["--coverage"], Arity::Flag),
    (&["--strict"], Arity::Flag),
    (&["--unknown-events"], Arity::One),
    (&["--stdin"], Arity::Flag),
    (&["--live"], Arity::Variable),
    (&["--sequence"], Arity::One),
    (&["--record"], Arity::One),
    (&["--fail-fast"], Arity::Flag),
    (&["--parser", "-p"], Arity::One),
    (&["--parserConfiguration", "-pc"], Arity::One),
    (&["--filters", "-f"], Arity::Variable),
    (&["--logs", "-l"], Arity::Variable),
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
        coverage: false,
        strict: false,
        unknown_events: UnknownEvents::Ignore,
        live: Vec::new(),
        sequence: None,
        record: None,
        fail_fast: false,
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
                "--stdin" | "--live" => {
                    let v = if canon == "--stdin" { "-".to_string() } else { v };
                    if v == "-" && cfg.live.iter().any(|l| l == "-") {
                        return Err("stdin can be read only once (--stdin is --live -).".into());
                    }
                    cfg.live.push(v);
                }
                "--sequence" => cfg.sequence = Some(v),
                "--record" => cfg.record = Some(v),
                "--fail-fast" => cfg.fail_fast = true,
                "--exclude-specs" => {
                    validate_classname(tok, &v)?;
                    cfg.exclude_specs.push(v);
                }
                "--json-report" => cfg.json_report = Some(v),
                "--check-hot" => cfg.check_hot = true,
                "--coverage" => cfg.coverage = true,
                "--strict" => cfg.strict = true,
                "--unknown-events" => cfg.unknown_events = v.parse()?,
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
                "--logs" => add_log_paths(&HostFiles(host), &host.cwd(), &v, &mut cfg.log_files)?,
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
    let live = !cfg.live.is_empty();
    if live != cfg.fail_fast {
        return Err("Live observation requires both --stdin and --fail-fast (or --live and --fail-fast).".into());
    }
    if live && (!cfg.log_files.is_empty() || cfg.input_kind != InputKind::Text) {
        return Err("Live input (--stdin, --live) requires TEXT input and cannot be combined with --logs.".into());
    }
    if cfg.live.len() > 1 && cfg.sequence.is_none() {
        return Err("Several live inputs need --sequence to order their lines.".into());
    }
    if !live && (cfg.sequence.is_some() || cfg.record.is_some()) {
        return Err("--sequence and --record apply to live input (--live or --stdin).".into());
    }
    if cfg.log_files.is_empty() && !live {
        return Err("No log files specified. Please provide log file or directory with --logs parameter.".into());
    }
    if serialized {
        return Err("Input kind SERIALIZEDEVENTS is not supported by p observe.".into());
    }
    Ok(Some(cfg))
}

/// The host's file system, for expanding `--logs`.
struct HostFiles<'a>(&'a dyn Host);

impl LogFiles for HostFiles<'_> {
    fn is_file(&self, path: &Path) -> bool {
        self.0.is_file(path)
    }
    fn is_dir(&self, path: &Path) -> bool {
        self.0.is_dir(path)
    }
    fn read_dir(&self, path: &Path) -> std::io::Result<Vec<PathBuf>> {
        self.0.read_dir(path)
    }
    fn read_to_string(&self, path: &Path) -> std::io::Result<String> {
        self.0.read_to_string(path)
    }
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

// ---- the specification ---------------------------------------------------------------------

struct Loaded {
    program: Program,
    java_package: String,
    /// What the configuration printout lists in place of `supplierJars`.
    source_option: &'static str,
    source_value: String,
}

/// Finds the P sources (`.pproj` or `--pfiles`) and type-checks them.
fn load_project(cfg: &ObserveConfig, host: &mut dyn Host) -> Result<Loaded, String> {
    let mut quiet = Buffered::new(host);
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
                let msg = quiet.errors().trim().trim_start_matches("Error: ").to_string();
                return Err(msg);
            }
        };
        project_name = proj.name.clone();
        package = proj.pobserve_package.clone();
        let files =
            proj.input_files.into_iter().filter(|f| f.to_string_lossy().to_lowercase().ends_with(".p")).collect();
        (files, proj.project_root, "pproj", file)
    };
    let host = quiet.inner;
    let program = match project::load(host, &project_name, &files, &root) {
        Ok((_, analysis)) => analysis.program,
        Err(project::LoadError::Read(e)) => return Err(e),
        Err(project::LoadError::Types) => return Err("The P specification does not type-check.".into()),
    };
    crate::checker::time::declares_time(&program)?;
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
    let is_spec = |m: &&crate::sema::MachineDecl| m.kind == MachineKind::Spec;
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
        .map(|(i, _)| Spec::from_decl(prog, crate::sema::MachineId(i as u32)))
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
        [i] => Ok(Spec::from_decl(prog, crate::sema::MachineId(*i as u32))),
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
            ErrorKind::Validation | ErrorKind::UnknownEvent => {
                host.out_colored_line(&e.message, Color::Red);
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
    if m.validation_errors > 0 {
        s.push_str(&format!("Total Validation Errors: {}\n", m.validation_errors));
    }
    if m.unknown_event_errors > 0 {
        s.push_str(&format!("Total Unknown Event Errors: {}\n", m.unknown_event_errors));
    }
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

#[cfg(test)]
mod tests {
    use super::*;

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
