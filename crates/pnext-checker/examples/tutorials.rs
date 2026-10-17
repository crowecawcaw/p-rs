//! Mirrors upstream's `TutorialsChecker/check.sh`: for every project directory under the
//! given root, checks each test case whose name has no underscore.
//!
//! Usage: tutorials [--schedules N] [--seed N] ROOT
use std::path::{Path, PathBuf};
use std::time::Instant;

use pnext_checker::engine::{Checker, CheckerOptions, Output};

struct Quiet;
impl Output for Quiet {
    fn line(&mut self, _: &str) {}
    fn error(&mut self, _: &str) {}
}

fn p_files(path: &Path, out: &mut Vec<PathBuf>) {
    if path.is_file() {
        if path.extension().map(|x| x == "p").unwrap_or(false) {
            out.push(path.to_path_buf());
        }
        return;
    }
    let Ok(rd) = std::fs::read_dir(path) else { return };
    let mut entries: Vec<_> = rd.map(|e| e.unwrap().path()).collect();
    entries.sort();
    for e in entries {
        p_files(&e, out);
    }
}

fn tag_values<'a>(text: &'a str, tag: &str) -> Vec<&'a str> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    text.split(open.as_str()).skip(1).map(|p| p.split(close.as_str()).next().unwrap().trim()).collect()
}

fn project_files(pproj: &Path, out: &mut Vec<PathBuf>) {
    let text = std::fs::read_to_string(pproj).unwrap();
    let dir = pproj.parent().unwrap();
    for inc in tag_values(&text, "IncludeProject") {
        project_files(&dir.join(inc), out);
    }
    for entry in tag_values(&text, "PFile") {
        p_files(&dir.join(entry), out);
    }
}

fn main() {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut schedules: Option<usize> = None;
    let mut seed = 0u32;
    let mut root = String::new();
    let mut trace: Option<String> = None;
    let mut i = 0;
    while i < raw.len() {
        match raw[i].as_str() {
            "--schedules" => {
                i += 1;
                schedules = Some(raw[i].parse().unwrap());
            }
            "--seed" => {
                i += 1;
                seed = raw[i].parse().unwrap();
            }
            "--trace" => {
                i += 1;
                trace = Some(raw[i].clone());
            }
            r => root = r.to_string(),
        }
        i += 1;
    }
    let mut dirs: Vec<_> =
        std::fs::read_dir(&root).unwrap().map(|e| e.unwrap().path()).filter(|p| p.is_dir()).collect();
    dirs.sort();
    let mut errors = 0;
    for dir in dirs {
        let Some(pproj) = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| p.extension().map(|x| x == "pproj").unwrap_or(false))
        else {
            continue;
        };
        let mut files = Vec::new();
        project_files(&pproj, &mut files);
        files.dedup();
        let mut sm = pnext_syntax::SourceMap::new();
        for f in &files {
            sm.add(f.display().to_string(), std::fs::read_to_string(f).unwrap());
        }
        let a = match pnext_sema::compile(&sm, &pnext_sema::Options::default()) {
            Ok(a) => a,
            Err(_) => {
                println!("{}: compile error", dir.display());
                errors += 1;
                continue;
            }
        };
        let n = schedules.unwrap_or(if dir.ends_with("6_Raft") { 1000 } else { 25000 });
        let checker = Checker::new(&a.program);
        for tc in a.program.test_cases() {
            if tc.name.contains('_') || trace.as_ref().map(|t| t != &tc.name).unwrap_or(false) {
                continue;
            }
            let opts = CheckerOptions { seed, iterations: n, ..Default::default() };
            let start = Instant::now();
            let r = checker.check(&tc, &opts, &mut Quiet);
            let secs = start.elapsed().as_secs_f64();
            match r {
                Ok(res) if res.report.num_bugs > 0 => {
                    if trace.is_some() {
                        for l in &res.last_log {
                            println!("  | {}", l);
                        }
                    }
                    errors += 1;
                    println!(
                        "{} {}: BUG after {} schedules [{:.1}s] {}",
                        dir.display(),
                        tc.name,
                        res.report.explored_fair_schedules + res.report.explored_unfair_schedules,
                        secs,
                        res.report.bug_reports[0].lines().next().unwrap_or("")
                    );
                }
                Ok(res) => println!(
                    "{} {}: ok, {} timelines [{:.1}s]",
                    dir.display(),
                    tc.name,
                    res.report.timelines.len(),
                    secs
                ),
                Err(e) => {
                    errors += 1;
                    println!("{} {}: engine error {}", dir.display(), tc.name, e);
                }
            }
        }
    }
    println!("Error Count: {}", errors);
}
