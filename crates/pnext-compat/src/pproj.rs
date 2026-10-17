//! `.pproj` project files (`ParsePProjectFile`).

use std::path::{Path, PathBuf};

use crate::host::{full_path, is_hidden, Host, HostExt};
use crate::{report_and_exit, Exit};

/// What a project file contributes to a compilation.
#[derive(Debug, Clone, Default)]
pub struct Project {
    pub name: String,
    /// `OutputDir`, made absolute (the directory is created).
    pub output_dir: PathBuf,
    /// `Target` languages, lower-cased (`pchecker`, `pex`, `pobserve`, `pverifier`).
    pub targets: Vec<String>,
    /// Input files (P and foreign), absolute, in discovery order.
    pub input_files: Vec<PathBuf>,
    pub project_root: PathBuf,
    pub dependencies: Vec<PathBuf>,
    /// `<pobserve-package>`: Java package of the PObserve backend's generated classes.
    pub pobserve_package: Option<String>,
}

struct Doc {
    path: PathBuf,
    text: String,
}

impl Doc {
    fn load(host: &dyn Host, path: &Path) -> Result<Doc, String> {
        let text = host.read_to_string(path).map_err(|e| e.to_string())?;
        roxmltree::Document::parse(&text).map_err(|e| format!("malformed XML: {}", e))?;
        Ok(Doc { path: path.to_path_buf(), text })
    }

    fn with<R>(&self, f: impl FnOnce(roxmltree::Node) -> R) -> R {
        let d = roxmltree::Document::parse(&self.text).expect("validated");
        f(d.root_element())
    }

    /// Text of each direct child element named `name`.
    fn elements(&self, name: &str) -> Vec<String> {
        self.with(|root| {
            root.children().filter(|c| c.has_tag_name(name)).map(|c| element_value(&c)).collect()
        })
    }

    /// Text of each `child` element under each `parent` element.
    fn nested(&self, parent: &str, child: &str) -> Vec<String> {
        self.with(|root| {
            root.children()
                .filter(|c| c.has_tag_name(parent))
                .flat_map(|p| p.children().filter(|c| c.has_tag_name(child)).map(|c| element_value(&c)).collect::<Vec<_>>())
                .collect()
        })
    }
}

/// `XElement.Value`: the concatenated text of all descendants.
fn element_value(n: &roxmltree::Node) -> String {
    n.descendants().filter(|d| d.is_text()).map(|d| d.text().unwrap_or("")).collect()
}

fn is_p_file(p: &str) -> bool {
    p.to_lowercase().ends_with(".p")
}

fn is_foreign_file(p: &str) -> bool {
    let l = p.to_lowercase();
    l.ends_with(".cs") || l.ends_with(".java")
}

/// `CheckFileValidity.IsLegalProjectName`.
pub fn is_legal_project_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// `CheckFileValidity.IsLegalPProjFile`: returns the full path when legal.
pub fn legal_pproj(host: &dyn Host, file: &str) -> (bool, PathBuf) {
    let full = full_path(&host.cwd(), Path::new(file));
    (file.len() > 2 && file.ends_with(".pproj") && host.is_file(&full), full)
}

/// `CheckFileValidity.IsLegalPFile`.
pub fn legal_p_file(host: &dyn Host, file: &str) -> Option<PathBuf> {
    let full = full_path(&host.cwd(), Path::new(file));
    (file.len() > 2 && is_p_file(file) && host.is_file(&full)).then_some(full)
}

fn legal_foreign_file(host: &dyn Host, file: &str) -> Option<PathBuf> {
    let full = full_path(&host.cwd(), Path::new(file));
    (file.len() > 2 && is_foreign_file(file) && host.is_file(&full)).then_some(full)
}

/// Recursive file listing in .NET `Directory.GetFiles(..., RecurseSubdirectories)` order:
/// a directory's files first, then its subdirectories breadth-first. Hidden entries skipped
/// when `skip_hidden`.
pub fn list_files(host: &dyn Host, dir: &Path, max_depth: Option<usize>, out: &mut Vec<PathBuf>) {
    let mut queue = std::collections::VecDeque::new();
    queue.push_back((dir.to_path_buf(), 0usize));
    while let Some((d, depth)) = queue.pop_front() {
        let Ok(entries) = host.read_dir(&d) else { continue };
        for e in &entries {
            if host.is_file(e) {
                out.push(e.clone());
            }
        }
        if max_depth.map(|m| depth < m).unwrap_or(true) {
            for e in entries {
                if host.is_dir(&e) {
                    queue.push_back((e, depth + 1));
                }
            }
        }
    }
}

struct ParseErr(String);

/// `ParseProjectFileForCompiler`.
pub fn parse_for_compiler(host: &mut dyn Host, project_file: &str) -> Result<Project, Exit> {
    match parse_for_compiler_inner(host, project_file) {
        Ok(p) => Ok(p),
        Err(Err2::Parse(m)) => Err(report_and_exit(host, &format!("<Error parsing project file>:\n {}", m))),
        Err(Err2::Exit(e)) => Err(e),
    }
}

enum Err2 {
    Parse(String),
    Exit(Exit),
}

impl From<ParseErr> for Err2 {
    fn from(e: ParseErr) -> Self {
        Err2::Parse(e.0)
    }
}

fn parse_for_compiler_inner(host: &mut dyn Host, project_file: &str) -> Result<Project, Err2> {
    let (ok, full) = legal_pproj(host, project_file);
    if !ok {
        return Err(Err2::Parse(format!(
            "Illegal P project file name {} or file {} not found",
            project_file,
            full.display()
        )));
    }
    host.out_line("----------------------------------------");
    host.out_line(&format!("==== Loading project file: {}", project_file));
    let mut inputs = Vec::new();
    let mut deps = Vec::new();
    all_dependencies(host, &full, &mut inputs, &mut deps)?;
    if inputs.is_empty() {
        return Err(Err2::Exit(report_and_exit(
            host,
            "At least one .p file must be provided as input files, no input files found after parsing the project file",
        )));
    }
    let doc = Doc::load(host, &full).map_err(ParseErr)?;
    let names = doc.elements("ProjectName");
    let Some(name) = names.first().cloned() else {
        return Err(Err2::Parse(format!("Missing project name in {}", full.display())));
    };
    if !is_legal_project_name(&name) {
        return Err(Err2::Parse(format!("{} is not a legal project name", name)));
    }
    let cwd = host.cwd();
    let output_dir = match doc.elements("OutputDir").first().filter(|s| !s.trim().is_empty()) {
        Some(o) => {
            let p = full_path(&cwd, Path::new(o));
            host.create_dir_all(&p).map_err(|e| Err2::Parse(e.to_string()))?;
            p
        }
        None => cwd.clone(),
    };
    let targets = match doc.elements("Target").first() {
        None => vec!["pchecker".to_string()],
        Some(t) => {
            let mut v = Vec::new();
            for part in t.split([',', ' ']).filter(|s| !s.is_empty()) {
                let l = part.to_lowercase();
                match l.as_str() {
                    "pchecker" | "pex" | "pobserve" | "pverifier" => v.push(l),
                    _ => {
                        return Err(Err2::Parse(format!(
                            "Expected PChecker, PEx, PObserve, or PVerifier as target, received {}",
                            t
                        )))
                    }
                }
            }
            v
        }
    };
    host.out_line("----------------------------------------");
    Ok(Project {
        name,
        output_dir,
        targets,
        input_files: inputs,
        project_root: full.parent().map(Path::to_path_buf).unwrap_or_default(),
        dependencies: deps,
        pobserve_package: doc.elements("pobserve-package").first().cloned(),
    })
}

fn push_unique(v: &mut Vec<PathBuf>, p: PathBuf) {
    if !v.contains(&p) {
        v.push(p);
    }
}

/// `GetAllProjectDependencies`.
fn all_dependencies(
    host: &mut dyn Host,
    project: &Path,
    inputs: &mut Vec<PathBuf>,
    deps: &mut Vec<PathBuf>,
) -> Result<(), Err2> {
    let doc = Doc::load(host, project).map_err(ParseErr)?;
    for f in read_all_input_files(host, &doc) {
        push_unique(inputs, f);
    }
    for dep in doc.elements("IncludeProject") {
        // upstream resolves included projects against the current directory
        let (ok, full) = legal_pproj(host, &dep);
        if !ok {
            return Err(Err2::Parse(format!("Illegal P project file name {} or file {} not found", dep, full.display())));
        }
        host.out_line(&format!("==== Loading project file: {}", full.display()));
        let dir = full.parent().map(Path::to_path_buf).unwrap_or_default();
        if deps.contains(&dir) {
            continue;
        }
        deps.push(dir);
        all_dependencies(host, &full, inputs, deps)?;
    }
    Ok(())
}

/// `ReadAllInputFiles`.
fn read_all_input_files(host: &mut dyn Host, doc: &Doc) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let base = doc.path.parent().map(Path::to_path_buf).unwrap_or_default();
    for entry in doc.nested("InputFiles", "PFile") {
        let combined = base.join(&entry);
        let mut candidates = Vec::new();
        if host.is_dir(&combined) {
            let mut files = Vec::new();
            list_files(host, &combined, None, &mut files);
            for f in files {
                let s = f.to_string_lossy().to_string();
                if (is_p_file(&s) || is_foreign_file(&s)) && !is_hidden(&f) {
                    candidates.push(s);
                }
            }
        } else {
            candidates.push(combined.to_string_lossy().to_string());
        }
        for c in candidates {
            if let Some(full) = legal_p_file(host, &c) {
                host.out_line(&format!("....... includes p file: {}", full.display()));
                push_unique(&mut out, full);
            } else if let Some(full) = legal_foreign_file(host, &c) {
                host.out_line(&format!("....... includes foreign file: {}", full.display()));
                push_unique(&mut out, full);
            }
        }
    }
    out
}

/// `ParseProjectFileForChecker`: the compiler output directory (`PCompiledPath`), as written
/// in the project file (or the current directory).
pub fn parse_for_checker(host: &mut dyn Host, project_file: &str) -> Result<String, Exit> {
    let (ok, full) = legal_pproj(host, project_file);
    if !ok {
        let m = format!("Illegal P project file name {} or file {} not found", project_file, full.display());
        return Err(report_and_exit(host, &format!("<Error parsing project file>:\n {}", m)));
    }
    let doc = match Doc::load(host, &full) {
        Ok(d) => d,
        Err(m) => return Err(report_and_exit(host, &format!("<Error parsing project file>:\n {}", m))),
    };
    Ok(match doc.elements("OutputDir").first().filter(|s| !s.trim().is_empty()) {
        Some(o) => o.clone(),
        None => host.cwd().to_string_lossy().to_string(),
    })
}

/// `ProjectFileLocator.FindLocalPProject`: the first non-hidden `*.pproj` in the current
/// directory, or `None`.
pub fn find_local_pproj(host: &mut dyn Host) -> Option<String> {
    host.out_line(".. Searching for a P project file *.pproj locally in the current folder");
    let cwd = host.cwd();
    let found = host
        .read_dir(&cwd)
        .unwrap_or_default()
        .into_iter()
        .find(|p| host.is_file(p) && !is_hidden(p) && p.extension().map(|e| e == "pproj").unwrap_or(false));
    match found {
        None => {
            host.out_line(&format!(".. No P project file *.pproj found in the current folder: {}", cwd.display()));
            None
        }
        Some(p) => {
            let s = p.to_string_lossy().to_string();
            host.out_line(&format!(".. Found P project file: {}", s));
            Some(s)
        }
    }
}
