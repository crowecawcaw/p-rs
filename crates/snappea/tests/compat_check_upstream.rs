//! `snappea-compat` against frozen outputs of upstream `p` (`tests/upstream/goldens`,
//! recorded by `scripts/record-upstream-goldens.py` from the upstream commit in
//! `tests/upstream/goldens/UPSTREAM`).
//!
//! Each case copies a directory of `tests/upstream` into an in-memory file system and runs
//! the recorded steps. Every step must give upstream's exit code, standard output and
//! standard error, and the run must write the same files with the same contents (`.sci`
//! files: only that they are written; files over 128 KB: by SHA-256). Both sides are normalized with
//! `tests/upstream/goldens/normalize.tsv`.
//!
//! Cases listed in `tests/upstream/goldens/known-differences.tsv` must still differ, so the
//! list shrinks as gaps are closed. `UPSTREAM_CASES=<id prefix>` runs only matching cases.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use regex::Regex;
use sha2::{Digest, Sha256};
use snappea::host::{Host, MemHost};

const WORK: &str = "/work";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/upstream")
}

fn goldens() -> PathBuf {
    root().join("goldens")
}

struct Rules {
    drops: Vec<Regex>,
    masks: Vec<(Regex, String)>,
}

impl Rules {
    fn load() -> Rules {
        let mut rules = Rules { drops: Vec::new(), masks: Vec::new() };
        for line in fs::read_to_string(goldens().join("normalize.tsv")).unwrap().lines() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let fields: Vec<&str> = line.split('\t').collect();
            match fields[0] {
                "drop" => rules.drops.push(Regex::new(fields[1]).unwrap()),
                "mask" => rules.masks.push((Regex::new(fields[1]).unwrap(), fields[2].to_string())),
                other => panic!("normalize.tsv: unknown rule {other:?}"),
            }
        }
        rules
    }

    fn apply(&self, text: &str, console: bool) -> String {
        let text = text.strip_prefix('\u{feff}').unwrap_or(text).replace(WORK, "<dir>");
        let mut text = if console {
            text.split('\n').filter(|l| !self.drops.iter().any(|d| d.is_match(l))).collect::<Vec<_>>().join("\n")
        } else {
            text
        };
        for (pattern, replacement) in &self.masks {
            text = pattern.replace_all(&text, replacement.as_str()).into_owned();
        }
        text
    }
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
    entries.sort();
    for e in entries {
        if e.is_dir() {
            walk(&e, out);
        } else {
            out.push(e);
        }
    }
}

fn rel(path: &Path, base: &Path) -> String {
    path.strip_prefix(base).unwrap().to_string_lossy().replace('\\', "/")
}

/// The first differing line, with a little context, or `None` when equal.
fn diff(want: &str, got: &str) -> Option<String> {
    if want == got {
        return None;
    }
    let (w, g): (Vec<&str>, Vec<&str>) = (want.split('\n').collect(), got.split('\n').collect());
    let i = w.iter().zip(&g).take_while(|(a, b)| a == b).count();
    let show = |lines: &[&str]| lines.iter().skip(i).take(3).map(|l| format!("      {l:?}")).collect::<Vec<_>>();
    Some(format!("line {}:\n    upstream:\n{}\n    snappea:\n{}", i + 1, show(&w).join("\n"), show(&g).join("\n")))
}

/// Runs one case; returns its differences from upstream.
fn run_case(dir: &Path, rules: &Rules) -> Vec<String> {
    let case: serde_json::Value = serde_json::from_str(&fs::read_to_string(dir.join("case.json")).unwrap()).unwrap();
    let cwd = match case["cwd"].as_str() {
        Some(c) => format!("{WORK}/{c}"),
        None => WORK.to_string(),
    };
    let mut h = MemHost::new(WORK);
    if let Some(source) = case["source"].as_str() {
        let src = root().join(source);
        let mut files = Vec::new();
        walk(&src, &mut files);
        for f in files {
            h.add_file(&format!("{WORK}/{}", rel(&f, &src)), &String::from_utf8_lossy(&fs::read(&f).unwrap()));
        }
    }
    h.cwd = PathBuf::from(&cwd);
    let before: BTreeSet<PathBuf> = h.files_under(&cwd).into_iter().collect();

    let mut problems = Vec::new();
    for step in case["steps"].as_array().unwrap() {
        let name = step["name"].as_str().unwrap();
        let args: Vec<String> =
            step["snappea"].as_array().unwrap().iter().map(|a| a.as_str().unwrap().into()).collect();
        h.stdout.clear();
        h.stderr.clear();
        let code = snappea::compat::run(&args, &mut h);
        let want_code = step["exit_code"].as_i64().unwrap();
        if i64::from(code) != want_code {
            problems.push(format!("{name}: exit code {code} (upstream {want_code})"));
        }
        for (kind, got) in [("out", &h.stdout), ("err", &h.stderr)] {
            let want = fs::read_to_string(dir.join(format!("{name}.{kind}.txt"))).unwrap();
            if let Some(d) = diff(&want, &rules.apply(got, true)) {
                problems.push(format!("{name}: std{kind} differs at {d}"));
            }
        }
    }

    let written: BTreeMap<String, PathBuf> = h
        .files_under(&cwd)
        .into_iter()
        .filter(|p| !before.contains(p))
        .map(|p| (rel(&p, Path::new(&cwd)), p))
        .collect();
    let want_files: Vec<String> =
        fs::read_to_string(dir.join("outputs.txt")).unwrap().lines().map(str::to_string).collect();
    let got_files: Vec<String> = written.keys().cloned().collect();
    if got_files != want_files {
        problems.push(format!("files written: {got_files:?}\n    upstream: {want_files:?}"));
    }
    for (name, path) in &written {
        let golden = dir.join("files").join(name);
        let got = rules.apply(&h.read_to_string(path).unwrap(), false);
        let hashed = dir.join("files").join(format!("{name}.sha256"));
        if golden.is_file() {
            if let Some(d) = diff(&fs::read_to_string(&golden).unwrap(), &got) {
                problems.push(format!("{name} differs at {d}"));
            }
        } else if hashed.is_file() {
            let want = fs::read_to_string(&hashed).unwrap();
            let sha: String = Sha256::digest(got.as_bytes()).iter().map(|b| format!("{b:02x}")).collect();
            if want.trim() != sha {
                problems.push(format!("{name} differs (large file, compared by SHA-256)"));
            }
        }
        // else: not written by upstream (reported above), or not compared (.sci)
    }
    problems
}

#[test]
fn matches_upstream_check_goldens() {
    let rules = Rules::load();
    let cases_root = goldens().join("cases");
    let mut all = Vec::new();
    walk(&cases_root, &mut all);
    let filter = std::env::var("UPSTREAM_CASES").unwrap_or_default();
    let cases: Vec<(String, PathBuf)> = all
        .iter()
        .filter(|p| p.file_name().is_some_and(|n| n == "case.json"))
        .map(|p| (rel(p.parent().unwrap(), &cases_root), p.parent().unwrap().to_path_buf()))
        .filter(|(id, _)| id.starts_with(&filter))
        .collect();
    if filter.is_empty() {
        assert!(cases.len() > 400, "expected goldens under {}", cases_root.display());
    }
    let known: BTreeMap<String, String> = fs::read_to_string(goldens().join("known-differences.tsv"))
        .unwrap()
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| {
            let (id, why) = l.split_once('\t').unwrap_or((l, ""));
            (id.to_string(), why.to_string())
        })
        .collect();

    // Cases are independent: each thread takes the next one until none are left.
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let next = AtomicUsize::new(0);
    let mut results: Vec<(String, Vec<String>)> = std::thread::scope(|s| {
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                s.spawn(|| {
                    let mut done = Vec::new();
                    while let Some((id, dir)) = cases.get(next.fetch_add(1, Ordering::Relaxed)) {
                        done.push((id.clone(), run_case(dir, &rules)));
                    }
                    done
                })
            })
            .collect();
        handles.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });
    results.sort();

    let mut failures = Vec::new();
    for (id, problems) in &results {
        match (known.contains_key(id), problems.is_empty()) {
            (false, false) => failures.push(format!("{id}:\n  {}", problems.join("\n  "))),
            (true, true) => failures.push(format!("{id}: now matches upstream; remove it from known-differences.tsv")),
            _ => {}
        }
    }
    let ran: BTreeSet<&str> = results.iter().map(|(id, _)| id.as_str()).collect();
    if filter.is_empty() {
        for id in known.keys().filter(|k| !ran.contains(k.as_str())) {
            failures.push(format!("{id}: listed in known-differences.tsv but not recorded"));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} cases differ from upstream:\n\n{}",
        failures.len(),
        results.len(),
        failures.join("\n\n")
    );
}
