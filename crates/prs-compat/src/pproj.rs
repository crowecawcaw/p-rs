//! `.pproj` project files (`ParsePProjectFile`).

use std::path::{Path, PathBuf};

use crate::host::{full_path, is_hidden, Host, HostExt};
use crate::{report_and_exit, Exit};

/// What a project file contributes: its name and sources. `OutputDir` and `Target` only
/// matter to upstream's compiler and are ignored.
#[derive(Debug, Clone, Default)]
pub struct Project {
    pub name: String,
    /// Input files (P and foreign), absolute, in discovery order.
    pub input_files: Vec<PathBuf>,
    pub project_root: PathBuf,
    /// `<pobserve-package>`: Java package of the PObserve backend's generated classes.
    pub pobserve_package: Option<String>,
}

/// The elements of a project file prs reads.
struct Doc {
    path: PathBuf,
    /// `<ProjectName>` texts.
    project_names: Vec<String>,
    /// `<InputFiles><PFile>` texts.
    input_files: Vec<String>,
    /// `<IncludeProject>` texts.
    include_projects: Vec<String>,
    pobserve_package: Option<String>,
}

impl Doc {
    fn load(host: &dyn Host, path: &Path) -> Result<Doc, String> {
        let text = host.read_to_string(path).map_err(|e| e.to_string())?;
        let xml = roxmltree::Document::parse(&text).map_err(|e| format!("malformed XML: {}", e))?;
        let root = xml.root_element();
        let children = |name: &'static str| root.children().filter(move |c| c.has_tag_name(name));
        let texts = |name| children(name).map(|c| element_value(&c)).collect::<Vec<_>>();
        Ok(Doc {
            path: path.to_path_buf(),
            project_names: texts("ProjectName"),
            input_files: children("InputFiles")
                .flat_map(|p| p.children().filter(|c| c.has_tag_name("PFile")).map(|c| element_value(&c)))
                .collect(),
            include_projects: texts("IncludeProject"),
            pobserve_package: texts("pobserve-package").into_iter().next(),
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
/// a directory's files first, then its subdirectories breadth-first, at most `max_depth`
/// levels below `dir`.
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
pub fn parse(host: &mut dyn Host, project_file: &str) -> Result<Project, Exit> {
    match parse_inner(host, project_file) {
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

fn parse_inner(host: &mut dyn Host, project_file: &str) -> Result<Project, Err2> {
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
    let doc = Doc::load(host, &full).map_err(ParseErr)?;
    let mut inputs = Vec::new();
    all_dependencies(host, &doc, &mut inputs, &mut Vec::new())?;
    if inputs.is_empty() {
        return Err(Err2::Exit(report_and_exit(
            host,
            "At least one .p file must be provided as input files, no input files found after parsing the project file",
        )));
    }
    let Some(name) = doc.project_names.first().cloned() else {
        return Err(Err2::Parse(format!("Missing project name in {}", full.display())));
    };
    if !is_legal_project_name(&name) {
        return Err(Err2::Parse(format!("{} is not a legal project name", name)));
    }
    host.out_line("----------------------------------------");
    Ok(Project {
        name,
        input_files: inputs,
        project_root: full.parent().map(Path::to_path_buf).unwrap_or_default(),
        pobserve_package: doc.pobserve_package,
    })
}

fn push_unique(v: &mut Vec<PathBuf>, p: PathBuf) {
    if !v.contains(&p) {
        v.push(p);
    }
}

/// `GetAllProjectDependencies`: collects the input files of `doc` and the projects it
/// includes; `deps` holds the directories of the projects already included.
fn all_dependencies(
    host: &mut dyn Host,
    doc: &Doc,
    inputs: &mut Vec<PathBuf>,
    deps: &mut Vec<PathBuf>,
) -> Result<(), Err2> {
    for f in read_all_input_files(host, doc) {
        push_unique(inputs, f);
    }
    for dep in &doc.include_projects {
        // upstream resolves included projects against the current directory
        let (ok, full) = legal_pproj(host, dep);
        if !ok {
            return Err(Err2::Parse(format!(
                "Illegal P project file name {} or file {} not found",
                dep,
                full.display()
            )));
        }
        host.out_line(&format!("==== Loading project file: {}", full.display()));
        let dir = full.parent().map(Path::to_path_buf).unwrap_or_default();
        if deps.contains(&dir) {
            continue;
        }
        deps.push(dir);
        let dep_doc = Doc::load(host, &full).map_err(ParseErr)?;
        all_dependencies(host, &dep_doc, inputs, deps)?;
    }
    Ok(())
}

/// `ReadAllInputFiles`.
fn read_all_input_files(host: &mut dyn Host, doc: &Doc) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let base = doc.path.parent().map(Path::to_path_buf).unwrap_or_default();
    for entry in &doc.input_files {
        let combined = base.join(entry);
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
