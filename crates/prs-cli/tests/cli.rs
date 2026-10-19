//! End-to-end tests of the `prs` command line on an in-memory host.

use std::path::Path;

use prs_compat::host::{Host, Job, MemHost, Threads};

fn run_threads<'a>(jobs: Vec<Job<'a>>, main: &mut (dyn FnMut() + 'a)) {
    std::thread::scope(|s| {
        for job in jobs {
            s.spawn(job);
        }
        main();
    });
}

const COINS: &str = "machine M {\n\
       start state S {\n\
         entry {\n\
           var n: int;\n\
           while (n < 6) {\n\
             if ($) { n = n + 1; } else { goto Done; }\n\
           }\n\
           assert false, \"six heads\";\n\
         }\n\
       }\n\
       state Done { }\n\
     }\n\
     machine Fine { start state S { } }\n\
     test tcCoins [main=M]: { M };\n\
     test tcFine [main=Fine]: { Fine };\n";

/// A host with four threads and `coins.p`.
fn host() -> MemHost {
    let mut h = MemHost::new("/w");
    h.threads = Some(Threads { available: 4, run: run_threads });
    h.add_file("coins.p", COINS);
    h
}

fn prs(host: &mut MemHost, args: &[&str]) -> i32 {
    host.stdout.clear();
    host.stderr.clear();
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    prs_cli::run(&args, host)
}

fn compat(host: &mut MemHost, args: &[&str]) -> i32 {
    host.stdout.clear();
    host.stderr.clear();
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    prs_compat::run(&args, host)
}

fn read(h: &MemHost, path: &str) -> String {
    h.read_to_string(Path::new(path)).unwrap_or_else(|e| panic!("{}: {}", path, e))
}

/// Every file under `dir`, with contents.
fn files(h: &MemHost, dir: &str) -> Vec<(String, String)> {
    h.files_under(dir).iter().map(|p| (p.display().to_string(), h.read_to_string(p).unwrap())).collect()
}

const OUT: &str = "PCheckerOutput/tcCoins/BugFinding";

#[test]
fn check_finds_a_bug_that_replays() {
    let mut h = host();
    assert_eq!(prs(&mut h, &["check", "tcCoins", "--seed", "2"]), 1, "{}", h.stdout);
    assert!(h.stdout.contains("Checker found a bug."), "{}", h.stdout);
    assert!(read(&h, &format!("{}/generatedOutput_0_0.txt", OUT)).contains("six heads"));
    let schedule = format!("{}/generatedOutput_0_0.schedule", OUT);
    assert!(read(&h, &schedule).contains("--test-method:tcCoins\n"));
    assert_eq!(prs(&mut h, &["replay", &schedule]), 1, "{}{}", h.stdout, h.stderr);
    assert!(h.stdout.contains("... Reproduced 1 bug."), "{}", h.stdout);
    assert!(h.stdout.ends_with("GB.\n"), "{}", h.stdout);
}

#[test]
fn tests_are_arguments() {
    let mut h = host();
    // all test cases by default; a bug in any fails the run
    assert_eq!(prs(&mut h, &["check", "-n", "200", "--seed", "2"]), 1, "{}", h.stdout);
    assert!(h.stdout.contains(".. Test case :: tcCoins") && h.stdout.contains(".. Test case :: tcFine"));
    assert!(h.is_dir(Path::new("/w/PCheckerOutput/tcFine/BugFinding")));
    assert_eq!(prs(&mut h, &["check", "coins.p", "tcF*", "-n", "5"]), 0, "{}", h.stdout);
    assert!(!h.stdout.contains("tcCoins"), "{}", h.stdout);
    assert_eq!(prs(&mut h, &["check", "tcNope"]), 2);
    assert_eq!(h.stderr, "error: no test case matches 'tcNope'. Test cases: tcCoins, tcFine\n");
}

#[test]
fn tests_lists_test_cases() {
    let mut h = host();
    assert_eq!(prs(&mut h, &["tests"]), 0);
    assert_eq!(h.stdout, "tcCoins\ntcFine\n");
    h.add_file("coins.p", "machine M { start state S { entry { x = 1; } } }");
    assert_eq!(prs(&mut h, &["tests", "coins.p"]), 2);
    assert!(h.stdout.contains("[Error:]"), "{}", h.stdout);
}

#[test]
fn command_line_errors_exit_with_2() {
    let mut h = host();
    for (args, err) in [
        (&["check", "--bogus"][..], "error: unknown option '--bogus'\n"),
        (&["check", "-n"], "error: '-n' needs a value\n"),
        (&["check", "-n", "x"], "error: '--schedules' needs a non-negative whole number, not 'x'\n"),
        (&["check", "--strategy", "pct"], "error: strategy 'pct' needs a number, e.g. 'pct:10'\n"),
        (
            &["check", "--strategy", "pos", "--exhaustive"],
            "error: '--strategy' and '--exhaustive' cannot be combined\n",
        ),
        (&["check", "--time", "5d"], "error: bad duration '5d' (examples: 90, 90s, 5m, 1h)\n"),
        (&["check", "-j", "0"], "error: '--workers' needs at least 1 worker\n"),
        (
            &["check", "--max-steps", "10", "--fair-max-steps", "5"],
            "error: '--fair-max-steps' must be at least '--max-steps'\n",
        ),
        (&["replay"], "error: no .schedule file given\n"),
        (&["frobnicate"], "error: unknown command 'frobnicate'\n"),
    ] {
        assert_eq!(prs(&mut h, args), 2, "{:?}", args);
        assert!(h.stderr.starts_with(err), "{:?}: {}", args, h.stderr);
    }
    assert_eq!(prs(&mut h, &["check", "--help"]), 0);
    assert!(h.stdout.starts_with("Usage: prs check"), "{}", h.stdout);
    assert_eq!(prs(&mut h, &[]), 0);
    assert!(h.stdout.starts_with("prs checks P models."), "{}", h.stdout);
}

#[test]
fn one_worker_explores_the_schedules_of_prs_compat() {
    for (prs_args, compat_args) in [
        (&["-n", "500", "--seed", "3"][..], &["-s", "500", "--seed", "3"][..]),
        (&["-n", "20", "--seed", "1", "--strategy", "pct:2"], &["-s", "20", "--seed", "1", "--sch-pct", "2"]),
        (&["--exhaustive"], &["--mode", "pex"]),
    ] {
        let mut a = host();
        let mut args = vec!["check", "tcCoins", "-j", "1"];
        args.extend_from_slice(prs_args);
        let code = prs(&mut a, &args);
        let mut b = host();
        let mut args = vec!["check", "-tc", "tcCoins"];
        args.extend_from_slice(compat_args);
        assert_eq!(code, compat(&mut b, &args), "{:?}", prs_args);
        assert!(!a.stdout.contains("workers."), "{}", a.stdout);
        assert_eq!(files(&a, "/w"), files(&b, "/w"), "{:?}", prs_args);
    }
}

#[test]
fn workers_find_a_bug_that_replays() {
    for seed in ["1", "2", "3", "4", "5"] {
        let mut h = host();
        assert_eq!(prs(&mut h, &["check", "tcCoins", "-n", "5000", "--seed", seed]), 1, "{}", h.stdout);
        assert!(h.stdout.contains("... Checker is using 4 workers.\n"), "{}", h.stdout);
        let txt = read(&h, &format!("{}/generatedOutput_0_0.txt", OUT));
        assert!(txt.contains("<ErrorLog> Assertion Failed: coins.p:8:1 six heads"), "{}", txt);
        // the trace ends with the statistics of all workers, as the console reports them
        let explored = h.stdout.lines().find(|l| l.starts_with("..... Explored") && l.ends_with("schedules")).unwrap();
        assert!(txt.contains(&explored.replace(".....", "<StrategyLog>")), "{}\n{}", explored, txt);
        let n: usize = explored.split(' ').nth(2).unwrap().parse().unwrap();
        assert!(n < 5000);
        let summary = read(&h, &format!("{}/generatedOutput_pchecker_summary.txt", OUT));
        assert!(summary.contains(&format!("\nschedules:{}\n", n)), "{}", summary);
        let schedule = format!("{}/generatedOutput_0_0.schedule", OUT);
        assert_eq!(prs(&mut h, &["replay", &schedule]), 1, "{}", h.stdout);
        assert!(h.stdout.contains("... Reproduced 1 bug."), "{}", h.stdout);
    }
}

#[test]
fn workers_share_the_schedule_count() {
    let mut h = host();
    // a check without bug: no six heads in a row
    h.add_file("coins.p", &COINS.replace("n < 6", "n < 60"));
    assert_eq!(prs(&mut h, &["check", "tcCoins", "-n", "1003", "--seed", "9"]), 0, "{}", h.stdout);
    assert!(h.stdout.contains("..... Explored 1003 schedules\n"), "{}", h.stdout);
    // progress is numbered across workers, each number once
    let progress: Vec<&str> = h.stdout.lines().filter(|l| l.starts_with("..... Schedule #")).collect();
    let mut sorted = progress.clone();
    sorted.sort_by_key(|l| l[16..].parse::<usize>().unwrap());
    sorted.dedup();
    assert_eq!(sorted.len(), 28, "{:?}", progress);
    assert!(read(&h, &format!("{}/generatedOutput_pchecker_summary.txt", OUT)).contains("\nschedules:1003\n"));
    // the coverage of all workers: every event handled
    assert!(read(&h, &format!("{}/generatedOutput.coverage.txt", OUT)).starts_with("Total event coverage: 100.0%"));
    // fewer schedules than workers
    assert_eq!(prs(&mut h, &["check", "tcCoins", "-n", "2", "--seed", "9"]), 0, "{}", h.stdout);
    assert!(h.stdout.contains("... Checker is using 2 workers.\n"), "{}", h.stdout);
    assert!(h.stdout.contains("..... Explored 2 schedules\n"), "{}", h.stdout);
    // a time limit without a schedule count explores until the limit; here, both
    assert_eq!(prs(&mut h, &["check", "tcCoins", "-n", "7", "--time", "1h"]), 0, "{}", h.stdout);
    assert!(h.stdout.contains("..... Explored 7 schedules\n"), "{}", h.stdout);
}

#[test]
fn some_runs_use_one_worker() {
    let mut h = host();
    for args in [
        &["-n", "1"][..],
        &["-n", "100", "--strategy", "feedback"],
        &["-n", "100", "--strategy", "rl"],
        &["--exhaustive"],
        &["-n", "5", "-v"],
    ] {
        let mut all = vec!["check", "tcCoins"];
        all.extend_from_slice(args);
        prs(&mut h, &all);
        assert!(!h.stdout.contains("workers."), "{:?}: {}", args, h.stdout);
    }
}

#[test]
fn exhaustive_search_writes_pex_output() {
    let mut h = host();
    assert_eq!(prs(&mut h, &["check", "tcCoins", "--exhaustive"]), 1, "{}", h.stdout);
    assert!(h.is_file(Path::new("/w/PCheckerOutput/tcCoins/PEx/generatedOutput_0_0.schedule")));
}

/// The time-tracking example of `design/time`.
fn retry_budget() -> MemHost {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../design/time/RetryBudget");
    let mut h = MemHost::new("/w");
    for rel in [
        "RetryBudget.pproj",
        "PSrc/Timer.p",
        "PSrc/Time.p",
        "PSrc/RetryClient.p",
        "PSpec/Budget.p",
        "PTst/Test.p",
        "PTst/latency.toml",
    ] {
        h.add_file(rel, &std::fs::read_to_string(dir.join(rel)).unwrap());
    }
    h
}

#[test]
fn time_tracking() {
    let mut h = retry_budget();
    let schedule = "PCheckerOutput/tcRetry/BugFinding/RetryBudget_0_0.schedule";
    for (latency, header) in [
        ("PTst/latency.toml", "--prs-time:timed=1;profile=same-az\n"),
        ("PTst/latency.toml:cross-region", "--prs-time:timed=1;profile=cross-region\n"),
    ] {
        let code = prs(&mut h, &["check", "tcRetry", "-n", "500", "--seed", "1", "--latency", latency, "--timed"]);
        assert_eq!(code, 1, "{}{}", h.stdout, h.stderr);
        assert!(read(&h, schedule).contains(header), "{}", read(&h, schedule));
    }
    assert_eq!(prs(&mut h, &["replay", schedule]), 1, "{}{}", h.stdout, h.stderr);
    assert_eq!(prs(&mut h, &["check", "--latency", "missing.toml"]), 2);
    assert!(h.stderr.contains("Cannot read latency file 'missing.toml'"), "{}", h.stderr);
}

#[test]
fn coverage_merge() {
    let mut h = host();
    assert_eq!(prs(&mut h, &["coverage", "merge", "nowhere"]), 2);
    assert_eq!(h.stderr, "error: directory not found: nowhere\n");
    assert_eq!(prs(&mut h, &["coverage", "merge"]), 0, "{}", h.stderr);
    assert_eq!(prs(&mut h, &["coverage"]), 2);
}

#[test]
fn observe_is_named_prs_observe() {
    let mut h = host();
    assert_eq!(prs(&mut h, &["observe", "--help"]), 0);
    assert!(h.stdout.starts_with("Usage: prs observe [options]"), "{}", h.stdout);
    assert_eq!(prs(&mut h, &["observe"]), 2);
}

#[test]
fn live_observe_rejects_incompatible_inputs_before_loading_project() {
    let mut h = host();
    assert_eq!(prs(&mut h, &["observe", "--spec", "Inventory", "--stdin"]), 2);
    assert!(h.stdout.contains("both --stdin and --fail-fast"));
    assert_eq!(prs(&mut h, &["observe", "--spec", "Inventory", "--stdin", "--fail-fast", "--logs", "coins.p"]), 2);
    assert!(h.stdout.contains("cannot be combined with --logs"));
    assert_eq!(prs(&mut h, &["observe", "--spec", "Inventory", "--stdin", "--fail-fast", "--inputkind", "JSON"]), 2);
    assert!(h.stdout.contains("requires TEXT input"));
}

#[test]
fn merge_observed_reports_without_models_and_reject_bad_input() {
    use prs_checker::observe_coverage::ObserveCoverage;
    let mut h = MemHost::new("/w");
    let mut coverage = ObserveCoverage {
        source_fingerprint: Some(format!("sha256:{}", "a".repeat(64))),
        ..ObserveCoverage::default()
    };
    coverage.declared_states.insert("Inventory.Ready".into());
    let report = |coverage: &ObserveCoverage, code: i32| {
        serde_json::json!({"exit_code": code,
        "input_complete": code == 0, "specs":[{"spec":"Inventory", "errors":[], "coverage":coverage.to_json()}]})
        .to_string()
    };
    h.add_file("first.json", &report(&coverage, 0));
    coverage.visited_states.insert("Inventory.Ready".into());
    h.add_file("second.json", &report(&coverage, 1));
    assert_eq!(
        prs(&mut h, &["coverage", "merge-observe", "first.json", "second.json", "--json-report", "suite.json"]),
        0,
        "{}",
        h.stderr
    );
    assert!(h.stdout.contains("States: 1/1"));
    assert!(h.stdout.contains("failed: 1; incomplete: 1"));
    assert!(!h.stdout.contains("schema_version"));
    let output = read(&h, "/w/suite.json");
    h.add_file("again.json", &output);
    assert_eq!(prs(&mut h, &["coverage", "merge-observe", "suite.json", "again.json", "--json"]), 0);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&h.stdout).unwrap(),
        serde_json::from_str::<serde_json::Value>(&output).unwrap()
    );
    h.add_file("bad.json", "{}");
    assert_eq!(prs(&mut h, &["coverage", "merge-observe", "bad.json"]), 2);
    assert!(h.stderr.contains("specs must be an array"));
}
