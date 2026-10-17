//! `p observe` against golden outputs of upstream PObserve on upstream's regression logs
//! (`tests/upstream/PObserve`, recorded by `scripts/vendor-pobserve-goldens.py`).
//!
//! The comparison normalizes what `scripts/compare-pobserve.py` does: colors, the output
//! directory, the spec location line of the configuration, timings, Java stack frames, and
//! the order of error blocks (upstream checks keys in parallel).

mod support;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use pnext_compat::host::{Host, MemHost};

/// Runs whose unknown-error messages differ: upstream's Java `NullPointerException` message
/// names a variable of the generated Java code (`"TMP_tmp2_12" is null`).
const KNOWN_MESSAGE_DIFFERENCES: &[&str] = &[
    "test_case_7_log_11_1_happy_unsorted.sorted",
    "test_case_7_log_12_2_happy_unsorted.sorted",
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/upstream/PObserve")
}

fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            files(&p, out);
        } else {
            out.push(p);
        }
    }
}

fn add_tree(h: &mut MemHost, from: &Path, to: &str) {
    let mut all = Vec::new();
    files(from, &mut all);
    for f in all {
        let rel = f
            .strip_prefix(from)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        h.add_file(&format!("{}/{}", to, rel), &fs::read_to_string(&f).unwrap());
    }
}

fn params(case: &str) -> BTreeMap<String, String> {
    let text =
        fs::read_to_string(root().join(format!("params/test_case_{}_params.txt", case))).unwrap();
    text.lines()
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .collect()
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.peek() == Some(&'[') {
            for d in chars.by_ref() {
                if d == 'm' {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn mask_total_time(l: &str) -> String {
    match l.find("Total time taken = ") {
        Some(i) if l.ends_with(" min") => format!("{}Total time taken = <t> min", &l[..i]),
        _ => l.to_string(),
    }
}

/// `UnknownError_12` -> `UnknownError_N`.
fn mask_error_numbers(s: &str) -> String {
    let mut out = s.to_string();
    for prefix in ["UnknownError_", "ParserError_"] {
        let mut res = String::new();
        let mut rest = out.as_str();
        while let Some(i) = rest.find(prefix) {
            res.push_str(&rest[..i + prefix.len()]);
            rest = &rest[i + prefix.len()..];
            let n = rest.chars().take_while(|c| c.is_ascii_digit()).count();
            res.push_str(if n > 0 { "N" } else { "" });
            rest = &rest[n..];
        }
        res.push_str(rest);
        out = res;
    }
    out
}

fn normalize_console(text: &str, outdir: &str) -> String {
    let text = strip_ansi(text).replace(outdir, "<out>");
    let lines: Vec<String> = text
        .split('\n')
        .filter(|l| {
            let t = l.trim_start();
            !(l.starts_with("  ")
                && ["outputDir", "supplierJars", "pproj", "pfiles"]
                    .iter()
                    .any(|k| {
                        t.strip_prefix(k)
                            .is_some_and(|r| r.trim_start().starts_with('='))
                    }))
        })
        .map(mask_total_time)
        .collect();
    let text = lines.join("\n");
    // error blocks between the first and last rule before "Total time", as a set
    let rule = "-------------------------------------\n";
    let Some(end) = text.find("Total time") else {
        return text;
    };
    let head = &text[..end];
    let (Some(first), Some(last)) = (head.find(rule), head.rfind(rule)) else {
        return text;
    };
    if first == last {
        return text;
    }
    let body = &head[first..last + rule.len()];
    let mut blocks: Vec<String> = body
        .split(rule)
        .filter(|b| !b.trim().is_empty())
        .map(mask_error_numbers)
        .collect();
    blocks.sort();
    let joined: String = blocks
        .iter()
        .map(|b| format!("{}{}{}", rule, b, rule))
        .collect();
    format!(
        "{}{}{}",
        &text[..first],
        joined,
        &text[first + body.len()..]
    )
}

fn normalize_file(name: &str, text: &str) -> String {
    if name == "PObserveMetrics.txt" {
        return text
            .split('\n')
            .map(|l| {
                let l = mask_total_time(l);
                // per-event rows: name, verified count, avg time, max time
                let cols: Vec<&str> = l.split_whitespace().collect();
                let is_row = cols.len() == 4
                    && cols[0].starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
                    && cols[1].bytes().all(|b| b.is_ascii_digit())
                    && cols[2].bytes().all(|b| b.is_ascii_digit() || b == b'.')
                    && cols[3].bytes().all(|b| b.is_ascii_digit());
                if is_row {
                    let count_end = l.find(cols[1]).unwrap() + cols[1].len();
                    format!("{} <avg> <max>", &l[..count_end])
                } else {
                    l
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
    }
    if name.starts_with("UnknownError") || name.starts_with("ParserError") {
        return text
            .split('\n')
            .filter(|l| !l.starts_with("\tat ") && !l.starts_with("\t... "))
            .collect::<Vec<_>>()
            .join("\n");
    }
    text.to_string()
}

/// Output files, with error files renumbered in order of their contents.
fn outputs(h: &MemHost, dir: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for p in h.files_under(dir) {
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        map.insert(name, h.read_to_string(&p).unwrap());
    }
    renumber(map)
}

fn renumber(mut map: BTreeMap<String, String>) -> BTreeMap<String, String> {
    for prefix in ["UnknownError_", "ParserError_"] {
        let keys: Vec<String> = map
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();
        let mut group: Vec<String> = keys.iter().map(|k| map.remove(k).unwrap()).collect();
        group.sort();
        for (i, v) in group.into_iter().enumerate() {
            map.insert(format!("{}{}.txt", prefix, i), v);
        }
    }
    map
}

#[test]
fn matches_upstream_pobserve_goldens() {
    let root = root();
    let mut cases = Vec::new();
    for suite in ["happy-logs", "error-logs"] {
        let suite_dir = root.join("expected").join(suite);
        for case_dir in fs::read_dir(&suite_dir).unwrap() {
            for e in fs::read_dir(case_dir.unwrap().path()).unwrap() {
                cases.push(e.unwrap().path());
            }
        }
    }
    cases.sort();
    assert!(
        cases.len() >= 90,
        "expected goldens under {}",
        root.display()
    );
    let mut failures = Vec::new();
    for exp in &cases {
        let rel = exp
            .strip_prefix(root.join("expected"))
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let label = exp.file_name().unwrap().to_string_lossy().to_string();
        let (log_rel, sorted) = match rel.strip_suffix(".sorted") {
            Some(r) => (r.to_string(), true),
            None => (rel.clone(), false),
        };
        let case = log_rel.split("test_case_").nth(1).unwrap()[..1].to_string();
        let p = params(&case);

        let mut h = MemHost::new("/work");
        add_tree(&mut h, &root.join("spec"), "/work/spec");
        let log_src = root.join("logs").join(&log_rel);
        if log_src.is_dir() {
            add_tree(&mut h, &log_src, &format!("/work/logs/{}", log_rel));
        } else {
            h.add_file(
                &format!("/work/logs/{}", log_rel),
                &fs::read_to_string(&log_src).unwrap(),
            );
        }
        let log = format!("/work/logs/{}", log_rel);
        let mut args: Vec<String> = [
            "-pp",
            "/work/spec/p_specs.pproj",
            "-o",
            "/work/out",
            "--spec",
            &p["--specName"],
            "--parser",
            &p["--parser"],
            "-l",
            &log,
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        if sorted {
            args.push("--assumeInputsAreSorted".into());
        }
        let code = pnext_compat::observe::run_with(&args, &mut h, support::pobserve_parsers::all());

        let mut problems = Vec::new();
        let want_code: i32 = fs::read_to_string(exp.join("exit_code"))
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        if code != want_code {
            problems.push(format!("exit code {} (upstream {})", code, want_code));
        }
        if KNOWN_MESSAGE_DIFFERENCES.contains(&label.as_str()) {
            if !problems.is_empty() {
                failures.push(format!("{}: {}", rel, problems.join("; ")));
            }
            continue;
        }
        let console = normalize_console(&h.stdout, "/work/out");
        if console != fs::read_to_string(exp.join("console.txt")).unwrap() {
            problems.push(format!("console differs:\n{}", console));
        }
        let got = outputs(&h, "/work/out");
        let mut want = BTreeMap::new();
        for e in fs::read_dir(exp).unwrap() {
            let p = e.unwrap().path();
            let n = p.file_name().unwrap().to_string_lossy().to_string();
            if n != "exit_code" && n != "console.txt" {
                want.insert(n, fs::read_to_string(&p).unwrap());
            }
        }
        let want = renumber(want);
        if got.keys().ne(want.keys()) {
            problems.push(format!(
                "files {:?} (upstream {:?})",
                got.keys().collect::<Vec<_>>(),
                want.keys().collect::<Vec<_>>()
            ));
        }
        for (n, w) in &want {
            if let Some(g) = got.get(n) {
                if normalize_file(n, g) != *w {
                    problems.push(format!("{} differs:\n{}", n, normalize_file(n, g)));
                }
            }
        }
        if !problems.is_empty() {
            failures.push(format!("{}: {}", rel, problems.join("; ")));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} runs differ from upstream:\n{}",
        failures.len(),
        cases.len(),
        failures.join("\n\n")
    );
}
