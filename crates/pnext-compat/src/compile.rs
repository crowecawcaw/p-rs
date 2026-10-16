//! `p compile` (`PCompilerOptions` + `Compiler.Compile`).

use std::path::{Path, PathBuf};

use crate::argparse::{ArgParser, DataType, ParseError, Parsed};
use crate::artifact::{self, Artifact, SourceFile};
use crate::host::{full_path, is_hidden, Color, Host, HostExt, Stream};
use crate::pproj::{self, is_legal_project_name, legal_p_file};
use crate::{report_and_exit, Exit};

const MODES: &[&str] = &["bugfinding", "pchecker", "pex", "pobserve", "verification", "pverifier"];

/// `PCompilerOptions` parser definition.
pub fn parser() -> ArgParser {
    let mut p = ArgParser::new(
        "p compile",
        "The P compiler compiles all the P files in the project together and generates the executable that can be checked for correctness by the P checker.",
    );
    p.group("project", "Compiling using `.pproj` file");
    p.add(
        Some("project"),
        "pproj",
        Some("pp"),
        "P project file to compile (*.pproj). If this option is not passed, the compiler searches for a *.pproj in the current folder",
        DataType::Str,
    );
    p.group("commandline", "Compiling P files directly through commandline");
    p.add(Some("commandline"), "pfiles", Some("pf"), "List of P files to compile", DataType::Str).multi();
    p.add(Some("commandline"), "projname", Some("pn"), "Project name for the compiled output", DataType::Str);
    p.add(Some("commandline"), "outdir", Some("o"), "Dump output to directory (absolute or relative path)", DataType::Str);
    p.add(
        None,
        "mode",
        Some("md"),
        "Compilation mode to use. Can be bugfinding (alias pchecker), pex, pobserve, or verification (alias pverifier). If this option is not passed, bugfinding mode is used as default",
        DataType::None,
    )
    .allowed(MODES);
    p.add(None, "pobserve-package", Some("po"), "PObserve package name", DataType::None).hidden();
    p.add(None, "debug", Some("d"), "Enable debug logs", DataType::Bool).hidden();
    p.add(
        None,
        "strict-errors",
        Some("se"),
        "Abort on the first type error (legacy P 2.x behavior). By default, the compiler reports all type errors in one pass.",
        DataType::Bool,
    );
    p.group("pverifier", "PVerifier options");
    p.add(Some("pverifier"), "timeout", Some("t"), "Set SMT solver timeout in seconds", DataType::Int).hidden();
    p.add(Some("pverifier"), "no-event-handler-checks", Some("nch"), "Do not check that all events are handled", DataType::Bool)
        .hidden();
    p.add(Some("pverifier"), "proof-blocks", Some("pb"), "List of proof blocks to check", DataType::Str).multi();
    p.add(Some("pverifier"), "check-only", Some("co"), "Check only the specified machine", DataType::Str).hidden();
    p.add(Some("pverifier"), "jobs", Some("j"), "Number of parallel processes to use", DataType::Int).hidden();
    p
}

/// Backends (`CompilerOutput`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    PChecker,
    PEx,
    PObserve,
    PVerifier,
}

impl Backend {
    fn name(self) -> &'static str {
        match self {
            Backend::PChecker => "PChecker",
            Backend::PEx => "PEx",
            Backend::PObserve => "PObserve",
            Backend::PVerifier => "PVerifier",
        }
    }

    fn parse(s: &str) -> Backend {
        match s.to_lowercase().as_str() {
            "pex" => Backend::PEx,
            "pobserve" => Backend::PObserve,
            "verification" | "pverifier" => Backend::PVerifier,
            _ => Backend::PChecker,
        }
    }
}

/// `CompilerConfiguration` (the parts pnext uses).
#[derive(Debug, Clone)]
pub struct CompilerConfig {
    pub project_name: String,
    pub output_dir: Option<PathBuf>,
    pub input_p_files: Vec<String>,
    pub input_foreign_files: Vec<String>,
    pub backends: Vec<Backend>,
    pub continue_on_error: bool,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        CompilerConfig {
            project_name: "generatedOutput".into(),
            output_dir: None,
            input_p_files: Vec::new(),
            input_foreign_files: Vec::new(),
            backends: vec![Backend::PChecker],
            continue_on_error: true,
        }
    }
}

/// Runs `p compile` with the given arguments; returns the exit code.
pub fn run(args: &[String], host: &mut dyn Host) -> i32 {
    let cfg = match parse(args, host) {
        Ok(c) => c,
        Err(Exit(code)) => return code,
    };
    compile(&cfg, host)
}

fn help_and_exit(p: &ArgParser, host: &mut dyn Host, msg: &str) -> Exit {
    host.write(Stream::Out, &p.help(), Some(Color::Cyan));
    report_and_exit(host, msg)
}

/// `PCompilerOptions.Parse`.
pub fn parse(args: &[String], host: &mut dyn Host) -> Result<CompilerConfig, Exit> {
    let p = parser();
    let mut result = match p.parse(args) {
        Ok(r) => r,
        Err(ParseError::Help) => {
            host.write(Stream::Out, &p.help(), Some(Color::Cyan));
            return Err(Exit(1));
        }
        Err(ParseError::Invalid(m)) => return Err(help_and_exit(&p, host, &m)),
    };
    if !result.iter().any(|a| a.long_name == "pproj" || a.long_name == "pfiles") {
        if let Some(found) = pproj::find_local_pproj(host) {
            result.push(Parsed { long_name: "pproj".into(), value: Some(crate::argparse::Value::Str(found)) });
        }
    }
    let mut cfg = CompilerConfig::default();
    if let Some(pp) = result.iter().find(|a| a.long_name == "pproj") {
        let file = pp.value.as_ref().map(|v| v.as_str().to_string()).unwrap_or_default();
        let proj = pproj::parse_for_compiler(host, &file)?;
        cfg.project_name = proj.name;
        cfg.output_dir = Some(proj.output_dir);
        cfg.backends = proj.targets.iter().map(|t| Backend::parse(t)).collect();
        for f in proj.input_files {
            let s = f.to_string_lossy().to_string();
            if s.to_lowercase().ends_with(".p") {
                cfg.input_p_files.push(s);
            } else {
                cfg.input_foreign_files.push(s);
            }
        }
    }
    for a in &result {
        let v = a.value.clone();
        match a.long_name.as_str() {
            "outdir" => {
                let d = full_path(&host.cwd(), Path::new(v.as_ref().map(|v| v.as_str()).unwrap_or("")));
                let _ = host.create_dir_all(&d);
                cfg.output_dir = Some(d);
            }
            "projname" => cfg.project_name = v.map(|v| v.as_str().to_string()).unwrap_or_default(),
            "debug" | "timeout" | "check-only" | "jobs" | "pobserve-package" | "proof-blocks" | "pproj" => {}
            "strict-errors" => cfg.continue_on_error = false,
            "mode" => cfg.backends = vec![Backend::parse(v.as_ref().map(|v| v.as_str()).unwrap_or(""))],
            "pfiles" => {
                let mut seen = Vec::new();
                for f in v.map(|v| v.as_str_list()).unwrap_or_default() {
                    if seen.contains(&f) {
                        continue;
                    }
                    seen.push(f.clone());
                    if f.ends_with(".p") {
                        cfg.input_p_files.push(f);
                    } else {
                        cfg.input_foreign_files.push(f);
                    }
                }
            }
            other => return Err(help_and_exit(&p, host, &format!("Unhandled parsed argument: '{}'", other))),
        }
    }
    // SanitizeConfiguration
    if cfg.input_p_files.is_empty() {
        find_local_p_files(host, &mut cfg);
    }
    if cfg.input_p_files.is_empty() {
        return Err(report_and_exit(host, "Could not find any *.p file. Run `p compile --help` for usage help."));
    }
    for f in &cfg.input_p_files {
        if legal_p_file(host, f).is_none() {
            // upstream dereferences the null FileInfo here and reports the resulting exception
            return Err(help_and_exit(&p, host, "Object reference not set to an instance of an object."));
        }
    }
    if !is_legal_project_name(&cfg.project_name) {
        return Err(report_and_exit(
            host,
            &format!("{} is not a legal project name Run `p compile --help` for usage help.", cfg.project_name),
        ));
    }
    if cfg.output_dir.is_none() {
        let d = full_path(&host.cwd(), Path::new("PGenerated"));
        let _ = host.create_dir_all(&d);
        cfg.output_dir = Some(d);
    }
    Ok(cfg)
}

/// `FindLocalPFiles`: `*.p` in the current folder and its immediate subfolders.
fn find_local_p_files(host: &mut dyn Host, cfg: &mut CompilerConfig) {
    host.out_line(".. Searching for P files locally in the current folder");
    let mut files = Vec::new();
    pproj::list_files(host, &host.cwd(), Some(1), &mut files);
    for f in files {
        if is_hidden(&f) || f.extension().map(|e| e != "p").unwrap_or(true) {
            continue;
        }
        let s = f.to_string_lossy().to_string();
        host.out_line(&format!(".. Adding P file: {}", s));
        cfg.input_p_files.push(s);
    }
}

fn write_error(host: &mut dyn Host, msg: &str) {
    host.out_colored_line(msg, Color::Red);
}

/// Loads the input files into a source map keyed by full path.
pub fn load_sources(host: &dyn Host, files: &[String]) -> Result<Vec<SourceFile>, String> {
    let mut out = Vec::new();
    for f in files {
        let full = full_path(&host.cwd(), Path::new(f));
        let text = host.read_to_string(&full).map_err(|e| e.to_string())?;
        out.push(SourceFile { path: full.to_string_lossy().to_string(), text });
    }
    Ok(out)
}

/// Prints front-end diagnostics the way `Compiler.Compile` does; returns true on success.
pub fn report_compile_result(
    host: &mut dyn Host,
    sm: &pnext_syntax::SourceMap,
    r: &Result<pnext_sema::Analysis, pnext_sema::CompileError>,
    after_parse: impl FnOnce(&mut dyn Host),
) -> bool {
    use pnext_sema::CompileError;
    match r {
        Err(CompileError::Parse { file, message }) => {
            write_error(host, &format!("[Parser Error:] [{}] parse error: {}", sm.file(*file).base_name, message));
            false
        }
        Err(CompileError::NotSupported { file, line, col, keyword }) => {
            write_error(
                host,
                &format!(
                    "[NotSupportedError:] {} (line {}:{}): \"{}\" keyword is only supported by PVerifier backend.",
                    sm.file(*file).name,
                    line,
                    col,
                    keyword
                ),
            );
            false
        }
        Err(CompileError::Analysis(a)) => {
            after_parse(host);
            for w in &a.warnings {
                host.out_colored_line(w, Color::Yellow);
            }
            let msgs = a.messages(sm);
            let mut rest = msgs.as_slice();
            if a.thrown.is_some() {
                write_error(host, &format!("[Error:] {}", rest[0]));
                rest = &rest[1..];
            }
            for (i, m) in rest.iter().enumerate() {
                if i > 0 {
                    write_error(host, "");
                }
                write_error(host, &format!("[Error:] {}", m));
            }
            false
        }
        Ok(a) => {
            after_parse(host);
            for w in &a.warnings {
                host.out_colored_line(w, Color::Yellow);
            }
            true
        }
    }
}

/// `Compiler.Compile`.
pub fn compile(cfg: &CompilerConfig, host: &mut dyn Host) -> i32 {
    host.out_line("Parsing ...");
    let sources = match load_sources(host, &cfg.input_p_files) {
        Ok(s) => s,
        Err(e) => {
            write_error(host, &format!("[Parser Error:] {}", e));
            return 1;
        }
    };
    let art = Artifact::new(&cfg.project_name, sources, cfg.input_foreign_files.clone());
    let sm = art.source_map();
    let opts = pnext_sema::Options {
        continue_on_error: cfg.continue_on_error,
        verifier: cfg.backends.contains(&Backend::PVerifier),
    };
    let r = pnext_sema::compile(&sm, &opts);
    let ok = report_compile_result(host, &sm, &r, |h| h.out_line("Type checking ..."));
    if !ok {
        return 1;
    }
    if cfg.backends.contains(&Backend::PVerifier) && cfg.backends.len() > 1 {
        write_error(host, "PVerifier backend must be used on its own!");
        return 1;
    }
    let parent = cfg.output_dir.clone().unwrap_or_else(|| host.cwd());
    let mut seen = Vec::new();
    for &b in &cfg.backends {
        if seen.contains(&b) {
            continue;
        }
        seen.push(b);
        let dir = parent.join(b.name());
        let _ = host.create_dir_all(&dir);
        host.out_line("----------------------------------------");
        host.out_line(&format!("Code generation for {}...", b.name()));
        if b != Backend::PChecker {
            write_error(
                host,
                &format!("[NotImplementedError:] The {} backend is not supported by pnext; use the PChecker backend.", b.name()),
            );
            return 1;
        }
        let file = format!("{}.{}", cfg.project_name, artifact::EXTENSION);
        host.out_line(&format!("Generated {}.", file));
        if let Err(e) = host.write_file(&dir.join(&file), &art.to_json()) {
            write_error(host, &format!("[{} Compiling Generated Code:] {}", b.name(), e));
            return 2;
        }
        host.out_line("Build succeeded.");
    }
    host.out_line("----------------------------------------");
    host.out_line("Compilation succeeded.");
    0
}
