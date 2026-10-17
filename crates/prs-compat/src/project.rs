//! The P sources a command works on, and how type errors are reported.
//!
//! Upstream compiles a project to a .NET assembly with `p compile` and checks that. prs
//! interprets P directly, so `p check` reads the sources itself (from a `.pproj` file or
//! `--pfiles`) and type-checks them in memory; there is no compile step or compiled file.

use std::path::{Path, PathBuf};

use crate::host::{full_path, is_hidden, relative_path, Color, Host, HostExt};
use crate::pproj;

/// Project name used when no `.pproj` or `--projname` gives one (upstream's default).
pub const DEFAULT_NAME: &str = "generatedOutput";

#[derive(Debug, Clone, PartialEq)]
pub struct SourceFile {
    /// Absolute path.
    pub path: String,
    /// Path relative to the current directory; upstream bakes this into assert messages
    /// and diagnostics.
    pub name: String,
    pub text: String,
}

/// A project's name and sources.
#[derive(Debug, Clone, PartialEq)]
pub struct Sources {
    pub project_name: String,
    pub files: Vec<SourceFile>,
}

impl Sources {
    /// Absolute path of the file registered as `id` in [`Sources::source_map`].
    pub fn full_path(&self, id: prs_syntax::FileId) -> &str {
        &self.files[id.0 as usize].path
    }

    pub fn source_map(&self) -> prs_syntax::SourceMap {
        let mut sm = prs_syntax::SourceMap::new();
        for s in &self.files {
            sm.add(s.name.clone(), s.text.clone());
        }
        sm
    }
}

/// Where the sources come from, as given on the command line.
#[derive(Debug, Clone, Default)]
pub struct Input {
    pub pproj: Option<String>,
    pub pfiles: Vec<String>,
    pub project_name: Option<String>,
}

/// Resolved input files.
#[derive(Debug, Clone)]
pub struct Resolved {
    pub project_name: String,
    pub p_files: Vec<PathBuf>,
    /// Foreign (`.cs`/`.java`) files listed by the project; not executable by prs.
    pub foreign_files: Vec<PathBuf>,
}

/// Resolves `input` to files: the `.pproj` file, `--pfiles`, or else the `.pproj` in the
/// current folder, or else the `*.p` files in it and its immediate subfolders.
/// Problems are reported through `host`.
pub fn resolve(host: &mut dyn Host, input: &Input) -> Result<Resolved, crate::Exit> {
    let mut r = Resolved { project_name: DEFAULT_NAME.into(), p_files: Vec::new(), foreign_files: Vec::new() };
    let pproj = match (&input.pproj, input.pfiles.is_empty()) {
        (Some(p), _) => Some(p.clone()),
        (None, true) => pproj::find_local_pproj(host),
        (None, false) => None,
    };
    let add = |r: &mut Resolved, f: PathBuf| {
        let list =
            if f.to_string_lossy().to_lowercase().ends_with(".p") { &mut r.p_files } else { &mut r.foreign_files };
        if !list.contains(&f) {
            list.push(f);
        }
    };
    if let Some(pp) = pproj {
        let proj = pproj::parse(host, &pp)?;
        r.project_name = proj.name;
        for f in proj.input_files {
            add(&mut r, f);
        }
    }
    for f in &input.pfiles {
        let Some(full) = pproj::legal_p_file(host, f) else {
            return Err(crate::report_and_exit(host, &format!("P file {} not found.", f)));
        };
        add(&mut r, full);
    }
    if r.p_files.is_empty() {
        host.out_line(".. Searching for P files locally in the current folder");
        let mut files = Vec::new();
        pproj::list_files(host, &host.cwd(), Some(1), &mut files);
        for f in files {
            if is_hidden(&f) || f.extension().map(|e| e != "p").unwrap_or(true) {
                continue;
            }
            host.out_line(&format!(".. Adding P file: {}", f.display()));
            r.p_files.push(f);
        }
    }
    if r.p_files.is_empty() {
        return Err(crate::report_and_exit(host, "Could not find any *.p file."));
    }
    if let Some(n) = &input.project_name {
        r.project_name = n.clone();
    }
    if !pproj::is_legal_project_name(&r.project_name) {
        return Err(crate::report_and_exit(host, &format!("{} is not a legal project name", r.project_name)));
    }
    Ok(r)
}

/// Reads `files`, naming each relative to `root`.
pub fn read(host: &dyn Host, project_name: &str, files: &[PathBuf], root: &Path) -> Result<Sources, String> {
    let mut out = Vec::new();
    for f in files {
        let full = full_path(&host.cwd(), f);
        let text = host.read_to_string(&full).map_err(|e| format!("{}: {}", full.display(), e))?;
        out.push(SourceFile { path: full.to_string_lossy().to_string(), name: relative_path(root, &full), text });
    }
    Ok(Sources { project_name: project_name.into(), files: out })
}

fn write_error(host: &mut dyn Host, msg: &str) {
    host.out_colored_line(msg, Color::Red);
}

/// Prints front-end diagnostics the way upstream's `Compiler.Compile` does; returns true on
/// success.
pub fn report_result(
    host: &mut dyn Host,
    src: &Sources,
    sm: &prs_syntax::SourceMap,
    r: &Result<prs_sema::Analysis, prs_sema::CompileError>,
) -> bool {
    use prs_sema::CompileError;
    match r {
        Err(CompileError::Parse { file, message, lexer }) => {
            if !*lexer {
                // Upstream first parses with SLL prediction and a bail-out strategy;
                // any syntax error makes it announce the LL(*) retry.
                host.out_colored_line("Reverting to LL(*) parsing strategy.", Color::Yellow);
            }
            write_error(host, &format!("[Parser Error:] [{}] parse error: {}", sm.file(*file).base_name, message));
            false
        }
        Err(CompileError::NotSupported { file, line, col, keyword }) => {
            write_error(
                host,
                &format!(
                    "[NotSupportedError:] {} (line {}:{}): \"{}\" keyword is only supported by PVerifier backend.",
                    src.full_path(*file),
                    line,
                    col,
                    keyword
                ),
            );
            false
        }
        Err(CompileError::Analysis(a)) => {
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
            for w in &a.warnings {
                host.out_colored_line(w, Color::Yellow);
            }
            true
        }
    }
}
