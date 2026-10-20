//! `prs::compat` is an adapter for upstream P's command line on top of the rest of the
//! crate: only it and the `prs-compat` binary may refer to it. Everything else (the core
//! library, `prs::cli`, `prs::test`, the wasm build) must work without it.

use std::path::{Path, PathBuf};

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {}", dir.display(), e)) {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Whether `line` refers to the compat module: a path through it (`compat::`, as in
/// `crate::compat::run` or `prs::compat::host`) or a `use` of it.
fn refers_to_compat(line: &str) -> bool {
    let bytes = line.as_bytes();
    line.match_indices("compat").any(|(i, _)| {
        let before = i.checked_sub(1).map(|j| bytes[j]);
        let word_start = !before.is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-');
        let rest = &line[i + "compat".len()..];
        word_start && (rest.starts_with("::") || (rest.starts_with([';', ',', '}']) && line.contains("use ")))
    })
}

#[test]
fn only_compat_depends_on_compat() {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src = crate_dir.join("src");
    let allowed = [src.join("compat"), src.join("bin/prs-compat.rs")];
    let mut files = Vec::new();
    rust_files(&src, &mut files);
    rust_files(&crate_dir.join("../prs-wasm/src"), &mut files);
    assert!(files.len() > 20, "found only {} source files", files.len());
    let mut offenders = Vec::new();
    for file in files.iter().filter(|f| !allowed.iter().any(|a| f.starts_with(a))) {
        let text = std::fs::read_to_string(file).unwrap();
        for (n, line) in text.lines().enumerate() {
            if refers_to_compat(line) {
                offenders.push(format!("{}:{}: {}", file.display(), n + 1, line.trim()));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "only prs::compat and the prs-compat binary may use prs::compat; move what is needed \
         out of compat instead:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn the_check_recognizes_references() {
    for line in [
        "use crate::compat::host::Host;",
        "    crate::compat::run(&args, host)",
        "use crate::compat;",
        "use crate::{checker, compat};",
        "let code = prs::compat::run(&args, &mut host);",
        "//! [`crate::compat::run`] runs it",
        "use super::compat::check;",
    ] {
        assert!(refers_to_compat(line), "{}", line);
    }
    for line in [
        "pub mod compat;",
        "//! `prs-compat` is upstream's command line",
        "/// PObserve-compatible status",
        "cannot merge coverage with incompatible declarations",
        "prs_compat::run()",
    ] {
        assert!(!refers_to_compat(line), "{}", line);
    }
}
