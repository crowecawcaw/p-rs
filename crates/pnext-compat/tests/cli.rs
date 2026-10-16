//! End-to-end tests of `p compile` / `p check` on an in-memory host.

use std::path::Path;

use pnext_compat::host::{Host, MemHost};

fn upstream(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/upstream").join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {}", p.display(), e))
}

fn run(host: &mut MemHost, args: &[&str]) -> i32 {
    host.stdout.clear();
    host.stderr.clear();
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    pnext_compat::run(&args, host)
}

/// `IsSubset` from upstream's `PCheckerLogGeneratorTests`.
fn json_subset(sub: &serde_json::Value, sup: &serde_json::Value) -> bool {
    use serde_json::Value::*;
    if sub == sup {
        return true;
    }
    match (sub, sup) {
        (Object(a), Object(b)) => a.iter().all(|(k, v)| b.get(k).map(|w| json_subset(v, w)).unwrap_or(false)),
        (Array(a), Array(b)) => a.iter().all(|x| b.iter().any(|y| json_subset(x, y))),
        _ => false,
    }
}

#[test]
fn log_generator_matches_upstream_golden_files() {
    let mut h = MemHost::new("/work");
    h.add_file("bug2.p", &upstream("RegressionTests/Feature1SMLevelDecls/DynamicError/bug2/bug2.p"));
    assert_eq!(run(&mut h, &["compile", "-pf", "bug2.p", "-pn", "bug2", "-o", "out"]), 0, "{}", h.stdout);
    assert!(h.stdout.contains("Compilation succeeded."));
    let code = run(&mut h, &["check", "out/PChecker/bug2.pnext", "-o", "res"]);
    assert_eq!(code, 1, "{}", h.stdout);
    assert!(h.stdout.contains("Checker found a bug."));

    let txt = h.read_to_string(Path::new("res/BugFinding/bug2_0_0.txt")).unwrap();
    let expected_txt = upstream("CorrectLogs/bugs2/bug2_0_0.txt");
    assert!(txt.contains(&expected_txt), "trace:\n{}", txt);

    let json: serde_json::Value =
        serde_json::from_str(&h.read_to_string(Path::new("res/BugFinding/bug2_0_0.trace.json")).unwrap()).unwrap();
    let expected = upstream("CorrectLogs/bugs2/bug2_0_0.trace.json");
    let expected: serde_json::Value = serde_json::from_str(expected.trim_start_matches('\u{feff}')).unwrap();
    assert!(json_subset(&expected, &json), "json:\n{}", json);
    assert!(h.is_file(Path::new("res/BugFinding/bug2_0_0.schedule")));
    assert!(h.is_file(Path::new("res/BugFinding/bug2_pchecker_summary.txt")));
}

#[test]
fn replay_reproduces_the_bug() {
    let mut h = MemHost::new("/work");
    h.add_file("bug2.p", &upstream("RegressionTests/Feature1SMLevelDecls/DynamicError/bug2/bug2.p"));
    run(&mut h, &["compile", "-pf", "bug2.p", "-pn", "bug2", "-o", "out"]);
    run(&mut h, &["check", "out/PChecker/bug2.pnext"]);
    run(&mut h, &["check", "out/PChecker/bug2.pnext", "--replay", "PCheckerOutput/BugFinding/bug2_0_0.schedule"]);
    assert!(h.stdout.contains("... Reproduced 1 bug."), "{}", h.stdout);
}

#[test]
fn project_file_compile_and_test_case_listing() {
    let mut h = MemHost::new("/proj");
    h.add_file(
        "Demo.pproj",
        "<Project><ProjectName>Demo</ProjectName><InputFiles><PFile>./PSrc/</PFile></InputFiles><OutputDir>./PGenerated/</OutputDir></Project>",
    );
    h.add_file(
        "PSrc/demo.p",
        "machine M { start state S { entry { assert false, \"boom\"; } } }\n\
         machine N { start state S { } }\n\
         test tcFails [main=M]: { M };\n\
         test tcPasses [main=N]: { N };\n",
    );
    assert_eq!(run(&mut h, &["compile"]), 0, "{}{}", h.stdout, h.stderr);
    assert!(h.stdout.contains("....... includes p file: /proj/PSrc/demo.p"));
    assert!(h.is_file(Path::new("PGenerated/PChecker/Demo.pnext")));

    assert_eq!(run(&mut h, &["check", "--list-tests"]), 0);
    assert!(h.stdout.contains(".. List of test cases (total 2)\ntcFails\ntcPasses\n"), "{}", h.stdout);

    // ambiguous: upstream lists the options on stderr and exits with 1
    assert_eq!(run(&mut h, &["check"]), 1);
    assert!(h.stderr.contains("Possible options are: \ntcFails\ntcPasses\n"), "{}", h.stderr);

    assert_eq!(run(&mut h, &["check", "-tc", "tcPasses", "-s", "10"]), 0, "{}", h.stdout);
    assert!(h.stdout.contains("..... Explored 10 schedules"));
    assert_eq!(run(&mut h, &["check", "-tc", "tcFails"]), 1);
    assert!(h.stdout.contains("Error: Assertion Failed: PSrc/demo.p:1:37 boom"), "{}", h.stdout);
}

#[test]
fn compile_errors_are_reported() {
    let mut h = MemHost::new("/w");
    h.add_file("bad.p", "machine M { start state S { entry { x = 1; } } }");
    assert_eq!(run(&mut h, &["compile", "-pf", "bad.p"]), 1);
    assert!(h.stdout.contains("[Error:]"), "{}", h.stdout);
    assert!(h.stdout.ends_with("~~ [PTool]: Thanks for using P! ~~\n"));
}

#[test]
fn bad_option_prints_help_and_error() {
    let mut h = MemHost::new("/w");
    assert_eq!(run(&mut h, &["check", "--bogus"]), 1);
    assert!(h.stdout.starts_with("usage: p check path [--help]"), "{}", h.stdout);
    assert_eq!(h.stderr, "Error: Unexpected argument: '--bogus'\n");
}

#[test]
fn output_directory_history_rotates() {
    let mut h = MemHost::new("/w");
    h.add_file("bug2.p", &upstream("RegressionTests/Feature1SMLevelDecls/DynamicError/bug2/bug2.p"));
    run(&mut h, &["compile", "-pf", "bug2.p", "-pn", "bug2", "-o", "out"]);
    for _ in 0..3 {
        run(&mut h, &["check", "out/PChecker/bug2.pnext"]);
    }
    assert!(h.is_dir(Path::new("PCheckerOutput/BugFinding")));
    assert!(h.is_dir(Path::new("PCheckerOutput/BugFinding0")));
    assert!(h.is_dir(Path::new("PCheckerOutput/BugFinding1")));
    assert!(!h.is_dir(Path::new("PCheckerOutput/BugFinding2")));
}

#[test]
fn scenario_coverage_is_reported_written_and_merged() {
    let mut h = MemHost::new("/w");
    h.add_file(
        "s.p",
        &upstream("RegressionTests/Feature1SMLevelDecls/Correct/ScenarioCoverageBasic/ScenarioCoverageBasic.p"),
    );
    assert_eq!(run(&mut h, &["compile", "-pf", "s.p", "-pn", "S"]), 0);
    assert_eq!(run(&mut h, &["check", "-s", "5"]), 0, "{}", h.stdout);
    assert!(h.stdout.contains(
        "..... Scenario coverage: 1/2 scenarios covered (1 gap)\n\
         .....   [covered] ReadAfterWrite: triggered in 5 schedules, 1 unique satisfying timeline\n\
         .....   [  GAP  ] NeverCovered: not covered (best partial progress: 1/2 states)\n"
    ), "{}", h.stdout);
    assert!(h.stdout.contains("..... Writing PCheckerOutput/BugFinding/S_scenario_coverage.json"));
    assert_eq!(run(&mut h, &["merge-scenario-coverage", "PCheckerOutput"]), 0);
    assert!(h.stdout.starts_with(
        "Unified scenario coverage across 1 test case:\n  1/2 scenarios covered in >=1 test case; 1 coverage gap.\n"
    ), "{}", h.stdout);
}
