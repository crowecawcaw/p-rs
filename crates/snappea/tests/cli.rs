//! End-to-end tests of the `snappea` command line on an in-memory host.

use std::path::Path;

use snappea::host::{Host, Job, MemHost, Threads};

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

fn snappea(host: &mut MemHost, args: &[&str]) -> i32 {
    host.stdout.clear();
    host.stderr.clear();
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    snappea::cli::run(&args, host)
}

fn compat(host: &mut MemHost, args: &[&str]) -> i32 {
    host.stdout.clear();
    host.stderr.clear();
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    snappea::compat::run(&args, host)
}

fn read(h: &MemHost, path: &str) -> String {
    h.read_to_string(Path::new(path)).unwrap_or_else(|e| panic!("{}: {}", path, e))
}

const OUT: &str = "PCheckerOutput/tcCoins";

/// The names of the files in `dir`.
fn names(h: &MemHost, dir: &str) -> Vec<String> {
    h.files_under(dir).iter().map(|p| p.file_name().unwrap().to_string_lossy().to_string()).collect()
}

#[test]
fn check_finds_a_bug_that_replays() {
    let mut h = host();
    assert_eq!(snappea(&mut h, &["check", "tcCoins", "--seed", "2"]), 1, "{}", h.stdout);
    // one line per test case, the bug's report and how to replay it; no upstream progress
    assert!(
        h.stdout.starts_with("Checking generatedOutput: 1 test case\nFAIL  tcCoins (bug found after "),
        "{}",
        h.stdout
    );
    assert!(
        h.stdout.contains(
            "\n      Assertion Failed: coins.p:8:1 six heads\n      Replay: snappea replay PCheckerOutput/tcCoins/bug.schedule\n"
        ),
        "{}",
        h.stdout
    );
    assert!(h.stdout.ends_with("\n1 of 1 test case failed in 0.00s. Output: PCheckerOutput/\n"), "{}", h.stdout);
    assert!(!h.stdout.contains("Checker found a bug."), "{}", h.stdout);
    // the test case's folder, with fixed names and no .sci file
    assert_eq!(names(&h, OUT), ["bug.schedule", "bug.trace.json", "bug.txt", "coverage.txt", "summary.txt"]);
    assert!(read(&h, &format!("{}/bug.txt", OUT)).contains("six heads"));
    let schedule = format!("{}/bug.schedule", OUT);
    assert!(read(&h, &schedule).contains("--test-method:tcCoins\n"));
    assert_eq!(snappea(&mut h, &["replay", &schedule]), 1, "{}{}", h.stdout, h.stderr);
    assert_eq!(
        h.stdout,
        "Replaying tcCoins from PCheckerOutput/tcCoins/bug.schedule\nReproduced the bug (0.00s):\n      Assertion Failed: coins.p:8:1 six heads\nRun with --verbose to see the schedule's log.\n"
    );
    // a run without a bug replaces the previous run's files
    h.add_file("coins.p", &COINS.replace("assert false", "assert true"));
    assert_eq!(snappea(&mut h, &["check", "coins.p", "tcCoins"]), 0, "{}", h.stdout);
    assert_eq!(names(&h, OUT), ["coverage.txt", "summary.txt"]);
    assert!(h.stdout.contains("\nok    tcCoins (all "), "{}", h.stdout);
    assert!(h.stdout.ends_with("\n🫛 The test case passed in 0.00s. Output: PCheckerOutput/\n"), "{}", h.stdout);
}

#[test]
fn the_replay_command_names_the_sources() {
    let mut h = host();
    h.add_file("my model/coins.p", COINS);
    assert_eq!(snappea(&mut h, &["check", "my model/coins.p", "tcCoins", "--seed", "2"]), 1, "{}", h.stdout);
    let replay = "snappea replay PCheckerOutput/tcCoins/bug.schedule 'my model/coins.p'";
    assert!(h.stdout.contains(&format!("      Replay: {}\n", replay)), "{}", h.stdout);
    assert_eq!(snappea(&mut h, &["replay", "PCheckerOutput/tcCoins/bug.schedule", "my model/coins.p"]), 1);
}

#[test]
fn replay_accepts_step_options_for_existing_compat_traces() {
    let mut h = MemHost::new("/w");
    h.add_file("ping.p", "event ePing; machine Loop { start state S { entry { send this, ePing; } on ePing do { send this, ePing; } } } test tcLoop [main=Loop]: { Loop };");
    assert_eq!(
        compat(&mut h, &["check", "-tc", "tcLoop", "--schedules", "3", "--max-steps", "50", "--fail-on-maxsteps"]),
        1,
        "{}{}",
        h.stdout,
        h.stderr
    );
    let path = "PCheckerOutput/tcLoop/BugFinding/generatedOutput_0_0.schedule";
    assert_eq!(
        snappea(&mut h, &["replay", path, "ping.p", "--max-steps", "50", "--fail-on-max-steps"]),
        1,
        "{}{}",
        h.stdout,
        h.stderr
    );
    assert!(h.stdout.contains("Reproduced the bug"));
}

#[test]
fn verbose_shows_progress() {
    let mut h = host();
    assert_eq!(snappea(&mut h, &["check", "tcCoins", "--seed", "2", "-v"]), 1, "{}", h.stdout);
    for s in [".. Test case :: tcCoins", "Checker found a bug.", "..... Writing PCheckerOutput/tcCoins/bug.schedule"] {
        assert!(h.stdout.contains(s), "{}: {}", s, h.stdout);
    }
    assert!(h.stdout.contains("\nFAIL  tcCoins (bug found after "), "{}", h.stdout);
}

#[test]
fn warnings_show_without_verbose() {
    let mut h = host();
    h.add_file(
        "p.pproj",
        "<Project><ProjectName>P</ProjectName><InputFiles><PFile>./coins.p</PFile><PFile>./F.cs</PFile></InputFiles></Project>",
    );
    h.add_file("F.cs", "");
    assert_eq!(snappea(&mut h, &["check", "tcFine"]), 0, "{}", h.stdout);
    assert!(h.stdout.starts_with("Warning: foreign file /w/F.cs is ignored;"), "{}", h.stdout);
}

#[test]
fn included_projects_are_found_beside_the_project_file() {
    let mut h = MemHost::new("/w");
    h.add_file(
        "app/App.pproj",
        "<Project><ProjectName>App</ProjectName><InputFiles><PFile>./coins.p</PFile></InputFiles>\
         <IncludeProject>../lib/Lib.pproj</IncludeProject></Project>",
    );
    h.add_file("app/coins.p", COINS);
    h.add_file(
        "lib/Lib.pproj",
        "<Project><ProjectName>Lib</ProjectName><InputFiles><PFile>./lib.p</PFile></InputFiles></Project>",
    );
    h.add_file("lib/lib.p", "machine Lib { start state S { } }\ntest tcLib [main=Lib]: { Lib };\n");
    assert_eq!(snappea(&mut h, &["tests", "app/App.pproj"]), 0, "{}{}", h.stdout, h.stderr);
    assert_eq!(h.stdout, "tcCoins\ntcFine\ntcLib\n");
    // upstream resolves them against the current folder
    assert_eq!(compat(&mut h, &["check", "-pp", "app/App.pproj", "--list-tests"]), 1);
    assert!(h.stderr.contains("Illegal P project file name ../lib/Lib.pproj"), "{}", h.stderr);
}

#[test]
fn tests_are_arguments() {
    let mut h = host();
    // all test cases by default; a bug in any fails the run
    assert_eq!(snappea(&mut h, &["check", "--seed", "2"]), 1, "{}", h.stdout);
    assert!(h.stdout.contains("\nFAIL  tcCoins (") && h.stdout.contains("\nok    tcFine (all "), "{}", h.stdout);
    assert!(h.stdout.ends_with("\n1 of 2 test cases failed in 0.00s. Output: PCheckerOutput/\n"), "{}", h.stdout);
    assert!(h.is_dir(Path::new("/w/PCheckerOutput/tcFine")));
    assert_eq!(snappea(&mut h, &["check", "coins.p", "tcF*"]), 0, "{}", h.stdout);
    assert!(!h.stdout.contains("tcCoins"), "{}", h.stdout);
    assert_eq!(snappea(&mut h, &["check", "tcNope"]), 2);
    assert_eq!(h.stderr, "error: no test case matches 'tcNope'. Test cases: tcCoins, tcFine\n");
}

#[test]
fn tests_lists_test_cases() {
    let mut h = host();
    assert_eq!(snappea(&mut h, &["tests"]), 0);
    assert_eq!(h.stdout, "tcCoins\ntcFine\n");
    h.add_file("coins.p", "machine M { start state S { entry { x = 1; } } }");
    assert_eq!(snappea(&mut h, &["tests", "coins.p"]), 2);
    assert!(h.stdout.contains("[Error:]"), "{}", h.stdout);
}

#[test]
fn command_line_errors_exit_with_2() {
    let mut h = host();
    for (args, err) in [
        (&["check", "--bogus"][..], "error: unknown option '--bogus'\n"),
        (&["check", "--timeout", "5d"], "error: bad duration '5d' (examples: 90, 90s, 5m, 1h)\n"),
        (&["check", "-j", "0"], "error: '--workers' needs at least 1 worker\n"),
        (&["replay"], "error: no .schedule file given\n"),
        (&["frobnicate"], "error: unknown command 'frobnicate'\n"),
    ] {
        assert_eq!(snappea(&mut h, args), 2, "{:?}", args);
        assert!(h.stderr.starts_with(err), "{:?}: {}", args, h.stderr);
    }
    assert_eq!(snappea(&mut h, &["check", "--help"]), 0);
    assert!(h.stdout.starts_with("Usage: snappea check"), "{}", h.stdout);
    assert_eq!(snappea(&mut h, &[]), 0);
    assert!(h.stdout.starts_with("🫛 snappea checks P models."), "{}", h.stdout);
}

#[test]
fn native_scheduling_controls_are_removed_and_compat_keeps_them() {
    let mut h = host();
    for option in [
        "--schedules",
        "--strategy",
        "--exhaustive",
        "--max-steps",
        "--fair-max-steps",
        "--fail-on-max-steps",
        "--timeline",
        "--timeline-payload",
    ] {
        assert_eq!(snappea(&mut h, &["check", option]), 2, "{option}: {}", h.stderr);
        assert!(h.stderr.contains("unknown option"), "{option}: {}", h.stderr);
    }
    for strategy in ["--sch-random", "--sch-feedback", "--sch-rl"] {
        assert_eq!(
            compat(&mut h, &["check", "-tc", "tcCoins", "--schedules", "1000", "--seed", "3", strategy]),
            if strategy == "--sch-rl" { 0 } else { 1 },
            "{}{}",
            h.stdout,
            h.stderr
        );
    }
    assert_eq!(compat(&mut h, &["check", "-tc", "tcFine", "--mode", "pex"]), 0, "{}{}", h.stdout, h.stderr);
    assert_eq!(snappea(&mut h, &["check", "--help"]), 0);
    assert!(h.stdout.contains("default: 10s"));
    assert!(!h.stdout.contains("--strategy"));
}

/// The schedule count of the `FAIL`/`ok` line of `test`.
fn schedules(stdout: &str, test: &str) -> usize {
    let line = stdout.lines().find(|l| l.contains(&format!(" {} (", test))).unwrap();
    let words: Vec<&str> = line.split([' ', '(']).filter(|w| !w.is_empty()).collect();
    let i = words.iter().position(|w| w.starts_with("schedule")).unwrap();
    words[i - 1].parse().unwrap()
}

#[test]
fn workers_find_a_bug_that_replays() {
    for seed in ["1", "2", "3", "4", "5"] {
        let mut h = host();
        assert_eq!(snappea(&mut h, &["check", "tcCoins", "--seed", seed]), 1, "{}", h.stdout);
        assert!(h.stdout.contains(" schedules, 4 workers, "), "{}", h.stdout);
        let txt = read(&h, &format!("{}/bug.txt", OUT));
        assert!(txt.contains("<ErrorLog> Assertion Failed: coins.p:8:1 six heads"), "{}", txt);
        // the trace ends with the statistics of all workers, as the console reports them
        let n = schedules(&h.stdout, "tcCoins");
        assert!(txt.contains(&format!("<StrategyLog> Explored {} schedules", n)), "{}\n{}", n, txt);
        assert!(n < 5000);
        let summary = read(&h, &format!("{}/summary.txt", OUT));
        assert!(summary.contains(&format!("\nschedules:{}\n", n)), "{}", summary);
        let schedule = format!("{}/bug.schedule", OUT);
        assert_eq!(snappea(&mut h, &["replay", &schedule]), 1, "{}", h.stdout);
        assert!(h.stdout.contains("Reproduced the bug"), "{}", h.stdout);
    }
}

#[test]
fn workers_share_the_schedule_count() {
    use snappea::driver::check::{self, CheckConfig, Style};
    let mut h = host();
    h.add_file("coins.p", &COINS.replace("assert false", "assert true"));
    // the driver's progress is numbered across workers, each number once
    h.stdout.clear();
    let mut cfg = CheckConfig {
        style: Style::Snappea,
        workers: Some(4),
        iterations: 1003,
        user_set_iterations: true,
        seed: Some(9),
        test_case_name: "tcCoins".into(),
        ..CheckConfig::default()
    };
    cfg.input.pfiles = vec!["coins.p".into()];
    check::finish_options(&mut cfg);
    check::resolve_sources(&mut h, &mut cfg).unwrap();
    let loaded = check::load(&mut h, &cfg).unwrap();
    check::set_output_directory(&mut h, &mut cfg).unwrap();
    let case = check::testing_process(&mut h, &loaded, &cfg).unwrap();
    assert_eq!((case.schedules, case.workers, case.bugs), (1003, 4, 0));
    assert!(h.stdout.contains("..... Explored 1003 schedules\n"), "{}", h.stdout);
    let progress: Vec<&str> = h.stdout.lines().filter(|l| l.starts_with("..... Schedule #")).collect();
    let mut sorted = progress.clone();
    sorted.sort_by_key(|l| l[16..].parse::<usize>().unwrap());
    sorted.dedup();
    assert_eq!(sorted.len(), 28, "{:?}", progress);
}

#[test]
fn some_runs_use_one_worker() {
    let mut h = host();
    for args in [&["-j", "1"][..], &["-v"]] {
        let mut all = vec!["check", "tcCoins"];
        all.extend_from_slice(args);
        snappea(&mut h, &all);
        assert!(!h.stdout.contains("workers"), "{:?}: {}", args, h.stdout);
    }
}

#[test]
fn exhaustive_search_writes_pex_output() {
    let mut h = host();
    assert_eq!(snappea(&mut h, &["check", "tcCoins"]), 1, "{}", h.stdout);
    assert!(h.is_file(Path::new("/w/PCheckerOutput/tcCoins/bug.schedule")));
    assert_eq!(snappea(&mut h, &["check", "tcFine"]), 0, "{}", h.stdout);
    assert!(
        h.stdout.contains(
            "
ok    tcFine (all "
        ),
        "{}",
        h.stdout
    );
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
    let schedule = "PCheckerOutput/tcRetry/bug.schedule";
    for (latency, header) in [
        ("PTst/latency.toml", "--snappea-time:timed=1;profile=same-az\n"),
        ("PTst/latency.toml:cross-region", "--snappea-time:timed=1;profile=cross-region\n"),
    ] {
        let code = snappea(&mut h, &["check", "tcRetry", "--seed", "1", "--latency", latency, "--timed"]);
        assert_eq!(code, 1, "{}{}", h.stdout, h.stderr);
        assert!(read(&h, schedule).contains(header), "{}", read(&h, schedule));
    }
    assert_eq!(snappea(&mut h, &["replay", schedule]), 1, "{}{}", h.stdout, h.stderr);
    assert_eq!(snappea(&mut h, &["check", "--latency", "missing.toml"]), 2);
    assert!(h.stderr.contains("Cannot read latency file 'missing.toml'"), "{}", h.stderr);
}

#[test]
fn coverage_merge() {
    let mut h = host();
    // snappea's report files; snappea-compat reads only upstream's names
    let report = r#"{"version": 1, "testCase": "tcA", "scenarios": [{"name": "S", "satisfied": true, "satisfyingSchedules": 1, "distinctSatisfyingTimelines": 1, "maxStatesVisited": 1, "monitorStates": 1}]}"#;
    h.add_file("out/tcA/scenario_coverage.json", report);
    assert_eq!(snappea(&mut h, &["coverage", "merge", "out"]), 0, "{}", h.stderr);
    assert!(h.stdout.contains("tcA"), "{}", h.stdout);
    assert_eq!(compat(&mut h, &["merge-scenario-coverage", "out"]), 0);
    assert!(!h.stdout.contains("tcA"), "{}", h.stdout);
    assert_eq!(snappea(&mut h, &["coverage", "merge", "nowhere"]), 2);
    assert_eq!(h.stderr, "error: directory not found: nowhere\n");
    assert_eq!(snappea(&mut h, &["coverage", "merge"]), 0, "{}", h.stderr);
    assert_eq!(snappea(&mut h, &["coverage"]), 2);
}

#[test]
fn observe_has_its_own_options() {
    let mut h = host();
    assert_eq!(snappea(&mut h, &["observe", "--help"]), 0);
    assert!(h.stdout.starts_with("Usage: snappea observe [SOURCE...] [LOG...] --spec <NAME>"), "{}", h.stdout);
    assert_eq!(snappea(&mut h, &["observe"]), 2);
    assert!(h.stderr.starts_with("error: name the specs to check with --spec"), "{}", h.stderr);
    // PObserve's options are snappea-compat's
    assert_eq!(snappea(&mut h, &["observe", "--spec", "S", "--outputDir", "o"]), 2);
    assert!(h.stderr.starts_with("error: unknown option '--outputDir'"), "{}", h.stderr);
}

#[test]
fn live_observe_rejects_incompatible_inputs_before_loading_project() {
    let mut h = host();
    h.add_file("a.log", "");
    for (args, err) in [
        (&["observe", "--spec", "Inventory", "--stdin", "a.log"][..], "cannot be combined with log files"),
        (&["observe", "--spec", "Inventory", "--stdin", "--p-trace"], "'--p-trace' cannot be combined with live input"),
        (&["observe", "--spec", "Inventory", "--live", "a", "--live", "b"], "several live inputs need --sequence"),
        (&["observe", "--spec", "Inventory", "--stdin", "--live", "-"], "stdin can be read only once"),
        (&["observe", "--spec", "Inventory", "a.log", "--record", "r"], "apply to live input"),
        (&["observe", "--spec", "Inventory"], "no logs given"),
        (&["observe", "--spec", "A", "--all-specs", "a.log"], "cannot be combined"),
    ] {
        assert_eq!(snappea(&mut h, args), 2, "{:?}", args);
        assert!(h.stderr.contains(err), "{:?}: {}", args, h.stderr);
    }
}

#[test]
fn observe_prints_each_spec_and_its_errors() {
    let mut h = host();
    h.add_file("inventory.p", include_str!("../../../examples/python/specs/inventory.p"));
    h.add_file(
        "logs/a.jsonl",
        "{\"event\":\"eStocked\",\"key\":\"c\",\"payload\":{\"units\":1},\"timestamp\":1}\n\
         {\"event\":\"eReserved\",\"key\":\"c\",\"payload\":{\"order\":\"o\",\"units\":5},\"timestamp\":2}\n",
    );
    let code = snappea(&mut h, &["observe", "inventory.p", "logs", "--spec", "InventoryCorrect"]);
    assert_eq!(code, 1, "{}{}", h.stdout, h.stderr);
    let mut lines = h.stdout.lines();
    assert_eq!(lines.next(), Some("FAIL  InventoryCorrect (1 error; 2 events, 1 key)"), "{}", h.stdout);
    assert!(lines.next().unwrap().starts_with("      key c: "), "{}", h.stdout);
    // the events leading up to it, with --verbose
    assert_eq!(snappea(&mut h, &["observe", "inventory.p", "logs", "--spec", "InventoryCorrect", "-v"]), 1);
    assert!(h.stdout.contains("        | {\"event\":\"eReserved\""), "{}", h.stdout);
    // a spec that does not exist
    assert_eq!(snappea(&mut h, &["observe", "inventory.p", "logs", "--spec", "Nope"]), 2);
    assert!(h.stderr.starts_with("error: no spec named Nope. Specs: InventoryCorrect"), "{}", h.stderr);
    // a log that is not a P trace, with one spec
    h.add_file("bad.json", "{not json");
    assert_eq!(snappea(&mut h, &["observe", "inventory.p", "bad.json", "--p-trace", "--spec", "InventoryCorrect"]), 2);
    assert!(h.stderr.starts_with("error: cannot read log /w/bad.json: "), "{}", h.stderr);
    assert!(h.stdout.is_empty(), "{}", h.stdout);
}

#[test]
fn merge_observed_reports_without_models_and_reject_bad_input() {
    use snappea::checker::observe_coverage::ObserveCoverage;
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
        snappea(&mut h, &["coverage", "merge-observe", "first.json", "second.json", "--json-report", "suite.json"]),
        0,
        "{}",
        h.stderr
    );
    assert!(h.stdout.contains("States: 1/1"));
    assert!(h.stdout.contains("failed: 1; incomplete: 1"));
    assert!(!h.stdout.contains("schema_version"));
    let output = read(&h, "/w/suite.json");
    h.add_file("again.json", &output);
    assert_eq!(snappea(&mut h, &["coverage", "merge-observe", "suite.json", "again.json", "--json"]), 0);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&h.stdout).unwrap(),
        serde_json::from_str::<serde_json::Value>(&output).unwrap()
    );
    h.add_file("bad.json", "{}");
    assert_eq!(snappea(&mut h, &["coverage", "merge-observe", "bad.json"]), 2);
    assert!(h.stderr.contains("specs must be an array"));
}

/// Issue #47: a misspelled event name is silently dropped unless `--unknown-events` asks.
#[test]
fn observe_reports_a_misspelled_event_name() {
    let mut h = host();
    let spec = include_str!("../../../examples/python/specs/inventory.p");
    h.add_file("inventory.p", spec);
    h.add_file(
        "typo.jsonl",
        "{\"event\":\"eStocked\",\"key\":\"c\",\"payload\":{\"units\":1},\"timestamp\":1}\n\
         {\"event\":\"eReserverd\",\"key\":\"c\",\"payload\":{\"order\":\"o\",\"units\":5},\"timestamp\":2}\n",
    );
    let args = |policy: &'static str| {
        let mut a = vec!["observe", "inventory.p", "--spec", "InventoryCorrect", "-l", "typo.jsonl"];
        a.extend(["--strict", "--json-report", "r.json"]);
        if !policy.is_empty() {
            a.extend(["--unknown-events", policy]);
        }
        a
    };
    assert_eq!(snappea(&mut h, &args("")), 0);
    assert!(h.stderr.is_empty());
    assert!(!read(&h, "r.json").contains("eReserverd"));

    assert_eq!(snappea(&mut h, &args("warn")), 0);
    assert_eq!(
        h.stderr,
        "warning: Unknown event 'eReserverd': not declared in the P sources (1 occurrence, first at typo.jsonl:2); did you mean eReserved?\n"
    );
    let report: serde_json::Value = serde_json::from_str(&read(&h, "r.json")).unwrap();
    assert_eq!(report["specs"][0]["errors"].as_array().unwrap().len(), 0);
    assert_eq!(report["specs"][0]["warnings"].as_array().unwrap().len(), 1);
    assert_eq!(report["specs"][0]["unknown_events"][0]["suggestion"], "eReserved");

    assert_eq!(snappea(&mut h, &args("error")), 2);
    assert!(h.stdout.contains("did you mean eReserved?"), "{}", h.stdout);
    let report: serde_json::Value = serde_json::from_str(&read(&h, "r.json")).unwrap();
    assert_eq!(report["exit_code"], 3);
    let error = &report["specs"][0]["errors"][0];
    assert_eq!(error["kind"], "unknown_event");
    assert!(error["message"].as_str().unwrap().contains("'eReserverd'"));
    assert_eq!(report["specs"][0]["unknown_events"][0]["location"], "typo.jsonl:2");

    assert_eq!(snappea(&mut h, &args("loud")), 2);
    assert!(h.stderr.contains("ignore, warn or error"), "{}", h.stderr);
}
