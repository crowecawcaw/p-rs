//! Runs the checker over the upstream regression corpus the way upstream's unit-test harness
//! does: `Correct` tests must check clean, `DynamicError` tests must find a bug, with 1000
//! schedules of at most 1000 steps. The `DefaultImpl` test case is checked, or the single
//! expected test of a parametric test directory.
//!
//! Usage: corpus [-v] [--seed N] [--schedules N] [--filter SUBSTR] ROOT...
use std::path::{Path, PathBuf};
use std::time::Instant;

use pnext_checker::engine::{Checker, CheckerOptions, Output};

struct Quiet;
impl Output for Quiet {
    fn line(&mut self, _: &str) {}
    fn error(&mut self, _: &str) {}
}

#[derive(Default)]
struct Stats {
    ok: usize,
    mismatched: usize,
    skipped: usize,
}

struct Args {
    verbose: bool,
    seed: u32,
    schedules: usize,
    filter: Option<String>,
    trace: bool,
    strategy: String,
    bound: usize,
}

fn main() {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut args = Args {
        verbose: false,
        seed: 0,
        schedules: 1000,
        filter: None,
        trace: false,
        strategy: "random".into(),
        bound: 0,
    };
    let mut roots = Vec::new();
    let mut i = 0;
    while i < raw.len() {
        match raw[i].as_str() {
            "-v" => args.verbose = true,
            "--trace" => args.trace = true,
            "--seed" => {
                i += 1;
                args.seed = raw[i].parse().unwrap();
            }
            "--schedules" => {
                i += 1;
                args.schedules = raw[i].parse().unwrap();
            }
            "--strategy" => {
                i += 1;
                args.strategy = raw[i].clone();
            }
            "--bound" => {
                i += 1;
                args.bound = raw[i].parse().unwrap();
            }
            "--filter" => {
                i += 1;
                args.filter = Some(raw[i].clone());
            }
            r => roots.push(r.to_string()),
        }
        i += 1;
    }
    let mut stats = Stats::default();
    for root in &roots {
        find_tests(Path::new(root), &mut stats, &args);
    }
    println!("ok={} mismatched={} skipped={}", stats.ok, stats.mismatched, stats.skipped);
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

fn is_category(n: &str) -> bool {
    matches!(n, "Correct" | "DynamicError" | "StaticError")
}

fn find_tests(dir: &Path, stats: &mut Stats, args: &Args) {
    let parent_is_category =
        dir.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str()).map(is_category).unwrap_or(false);
    if !parent_is_category {
        let mut entries: Vec<_> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
        entries.sort();
        for e in entries.into_iter().filter(|e| e.is_dir()) {
            find_tests(&e, stats, args);
        }
        return;
    }
    let category = dir.parent().unwrap().file_name().unwrap().to_str().unwrap().to_string();
    if category == "StaticError" {
        return;
    }
    if let Some(f) = &args.filter {
        if !dir.display().to_string().contains(f.as_str()) {
            return;
        }
    }
    let mut files = Vec::new();
    p_files(dir, &mut files);
    if files.is_empty() {
        return;
    }
    let mut sm = pnext_syntax::SourceMap::new();
    for f in &files {
        let text = String::from_utf8_lossy(&std::fs::read(f).unwrap()).to_string();
        sm.add(f.display().to_string(), text);
    }
    let analysis = match pnext_sema::compile(&sm, &pnext_sema::Options::default()) {
        Ok(a) => a,
        Err(_) => {
            stats.mismatched += 1;
            println!("MISMATCH {} ({}): does not compile", dir.display(), category);
            return;
        }
    };
    let cases = analysis.program.test_cases();
    let dir_name = dir.file_name().unwrap().to_str().unwrap().to_lowercase();
    let wanted: Option<String> = if dir_name.contains("param") {
        let expected = std::fs::read_to_string(dir.join("ExpectedParametricTests.txt")).unwrap_or_default();
        let lines: Vec<&str> =
            expected.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')).collect();
        if lines.len() != 1 {
            stats.skipped += 1;
            return;
        }
        Some(lines[0].to_string())
    } else {
        Some("DefaultImpl".to_string())
    };
    let tc = cases.iter().find(|c| Some(&c.name) == wanted.as_ref());
    let Some(tc) = tc else {
        // upstream fails to load the test class: an error exit, which counts as a bug
        let ok = category == "DynamicError";
        record(stats, ok, dir, &category, "no test case", &format!("no test case {:?}", wanted));
        return;
    };
    if args.verbose {
        eprintln!("RUN {}", dir.display());
    }
    let checker = Checker::new(&analysis.program);
    let opts = CheckerOptions {
        seed: args.seed,
        iterations: args.schedules,
        max_unfair_steps: 1000,
        max_fair_steps: 1000,
        user_set_max_fair_steps: true,
        strategy: args.strategy.clone(),
        strategy_bound: args.bound,
        ..Default::default()
    };
    let start = Instant::now();
    let r = checker.check(tc, &opts, &mut Quiet);
    let elapsed = start.elapsed().as_secs_f64();
    match r {
        Err(e) => record(stats, false, dir, &category, "engine error", &e),
        Ok(res) => {
            let found = res.report.num_bugs > 0;
            let ok = found == (category == "DynamicError");
            let what = if found { "bug" } else { "no bug" };
            let detail = res.report.bug_reports.first().cloned().unwrap_or_default();
            if args.trace {
                for l in &res.last_log {
                    println!("  | {}", l);
                }
            }
            if let (true, Some(x)) = (args.verbose, &res.exhaustive) {
                let n = res.report.explored_fair_schedules + res.report.explored_unfair_schedules;
                println!(
                    "SEARCH {} finished={} states={} schedules={} pruned={} bound_hits={} depth={}",
                    dir.display(),
                    x.finished,
                    x.distinct_states,
                    n,
                    x.pruned_schedules,
                    x.depth_bound_hits,
                    x.max_depth
                );
            }
            record(stats, ok, dir, &category, what, &format!("{} [{:.2}s]", detail, elapsed));
        }
    }
    if args.verbose && elapsed > 5.0 {
        println!("SLOW {} {:.1}s", dir.display(), elapsed);
    }
    let _ = args.verbose;
}

fn record(stats: &mut Stats, ok: bool, dir: &Path, category: &str, what: &str, detail: &str) {
    if ok {
        stats.ok += 1;
    } else {
        stats.mismatched += 1;
        println!("MISMATCH {} ({}): {} {}", dir.display(), category, what, detail);
    }
}
