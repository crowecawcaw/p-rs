//! The file `p compile` produces and `p check` consumes.
//!
//! Upstream compiles to a .NET assembly; prs interprets P directly, so the "compiled"
//! output is a snapshot of the type-checked project: its name plus the full text of every
//! source file. Like an assembly it is self-contained, so `p check` sees exactly what was
//! compiled even if the sources change afterwards.

use serde::{Deserialize, Serialize};

/// File extension of compiled projects.
pub const EXTENSION: &str = "prs";

/// Format tag stored in every artifact.
pub const FORMAT: &str = "prs-compiled-project/1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceFile {
    /// Absolute path at compile time.
    pub path: String,
    /// Path relative to the directory `p compile` ran in; upstream bakes this into assert
    /// messages and diagnostics.
    pub name: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Artifact {
    pub format: String,
    pub project_name: String,
    pub sources: Vec<SourceFile>,
    /// Foreign (`.cs`/`.java`) files listed by the project; not executable by prs.
    #[serde(default)]
    pub foreign_files: Vec<String>,
}

impl Artifact {
    pub fn new(project_name: &str, sources: Vec<SourceFile>, foreign_files: Vec<String>) -> Artifact {
        Artifact { format: FORMAT.into(), project_name: project_name.into(), sources, foreign_files }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("serializable")
    }

    pub fn from_json(text: &str) -> Result<Artifact, String> {
        let a: Artifact = serde_json::from_str(text).map_err(|e| e.to_string())?;
        if a.format != FORMAT {
            return Err(format!("unsupported compiled file format '{}'", a.format));
        }
        Ok(a)
    }

    /// The sources as a [`SourceMap`](prs_syntax::SourceMap).
    /// Absolute path of the file registered as `id` in [`Artifact::source_map`].
    pub fn full_path(&self, id: prs_syntax::FileId) -> &str {
        &self.sources[id.0 as usize].path
    }

    pub fn source_map(&self) -> prs_syntax::SourceMap {
        let mut sm = prs_syntax::SourceMap::new();
        for s in &self.sources {
            sm.add(s.name.clone(), s.text.clone());
        }
        sm
    }
}
