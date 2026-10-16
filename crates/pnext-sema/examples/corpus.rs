//! Type-checks every regression test under the given roots and compares the outcome with
//! the directory category (`Correct`/`DynamicError` must type-check, `StaticError` must not).
//! Prints mismatches and, with `-v`, every diagnostic.
use std::path::{Path, PathBuf};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let verbose = args.iter().any(|a| a == "-v");
    let mut stats = Stats::default();
    for root in args.iter().filter(|a| !a.starts_with('-')) {
        find_tests(Path::new(root), &mut stats, verbose);
    }
    println!(
        "ok={} mismatched={} param_ok={} param_bad={}",
        stats.ok, stats.mismatched, stats.param_ok, stats.param_bad
    );
}

#[derive(Default)]
struct Stats {
    ok: usize,
    mismatched: usize,
    param_ok: usize,
    param_bad: usize,
}

fn p_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
    entries.sort();
    for e in entries {
        if e.is_dir() {
            p_files(&e, out);
        } else if e.extension().map(|x| x == "p").unwrap_or(false) {
            out.push(e);
        }
    }
}

fn find_tests(dir: &Path, stats: &mut Stats, verbose: bool) {
    let mut files = Vec::new();
    p_files(dir, &mut files);
    if files.is_empty() {
        return;
    }
    // a test directory is one whose ancestor is a category directory
    let category = dir
        .ancestors()
        .skip(1)
        .find_map(|a| a.file_name().and_then(|n| n.to_str()).filter(|n| matches!(*n, "Correct" | "DynamicError" | "StaticError")).map(|s| s.to_string()));
    let is_test_dir = category.is_some()
        && dir.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str()).map(|n| matches!(n, "Correct" | "DynamicError" | "StaticError")).unwrap_or(false);
    if !is_test_dir {
        let mut entries: Vec<_> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
        entries.sort();
        for e in entries.into_iter().filter(|e| e.is_dir()) {
            find_tests(&e, stats, verbose);
        }
        return;
    }
    let category = category.unwrap();
    let mut sm = pnext_syntax::SourceMap::new();
    for f in &files {
        let text = String::from_utf8_lossy(&std::fs::read(f).unwrap()).to_string();
        sm.add(f.display().to_string(), text);
    }
    let r = pnext_sema::compile(&sm, &pnext_sema::Options::default());
    let expect_fail = category == "StaticError";
    let failed = r.is_err();
    let msgs: Vec<String> = match &r {
        Ok(_) => Vec::new(),
        Err(pnext_sema::CompileError::Parse { file, message, .. }) => {
            vec![format!("[Parser Error:] [{}] parse error: {}", sm.file(*file).base_name, message)]
        }
        Err(pnext_sema::CompileError::NotSupported { file, line, col, keyword }) => {
            vec![format!("[NotSupportedError:] {} (line {}:{}): \"{}\" keyword is only supported by PVerifier backend.", sm.file(*file).name, line, col, keyword)]
        }
        Err(pnext_sema::CompileError::Analysis(a)) => a.messages(&sm).iter().map(|m| format!("[Error:] {}", m)).collect(),
    };
    if failed == expect_fail {
        stats.ok += 1;
        if verbose {
            println!("OK   {} ({})", dir.display(), category);
            for m in &msgs {
                println!("     {}", m);
            }
        }
    } else {
        stats.mismatched += 1;
        println!("MISMATCH {} ({}): {}", dir.display(), category, if failed { "failed" } else { "passed" });
        for m in &msgs {
            println!("     {}", m);
        }
    }
    if let Ok(a) = &r {
        let expected = dir.join("ExpectedParametricTests.txt");
        if expected.exists() {
            let want: Vec<String> = std::fs::read_to_string(&expected)
                .unwrap()
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect();
            let got: Vec<String> = a.program.test_cases().iter().map(|t| t.name.clone()).collect();
            let mut ws = want.clone();
            ws.sort();
            let mut gs = got.clone();
            gs.sort();
            if ws == gs {
                stats.param_ok += 1;
            } else {
                stats.param_bad += 1;
                println!("PARAM MISMATCH {}", dir.display());
                for w in &want {
                    if !got.contains(w) {
                        println!("     missing  {}", w);
                    }
                }
                for g in &got {
                    if !want.contains(g) {
                        println!("     extra    {}", g);
                    }
                }
            }
        }
    }
}
