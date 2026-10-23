//! Lint/check parity over real parsed and type-checked P models, without scheduling
//! when a lint fails. Upstream compatibility remains independent of lint policy.
use snappea::host::MemHost;

fn run(host: &mut MemHost, args: &[&str]) -> i32 {
    host.stdout.clear();
    host.stderr.clear();
    snappea::cli::run(&args.iter().map(|s| s.to_string()).collect::<Vec<_>>(), host)
}
const MODEL: &str = "event E;\nspec Passive observes E { start state S { on E goto S; } }\nmachine Main { start state S {} }\ntest t [main=Main]: assert Passive in { Main };";

#[test]
fn lint_uses_the_shared_json_output_for_results_rules_and_failures() {
    let mut h = MemHost::new("/w");
    h.terminal = false;
    h.add_file("model.p", MODEL);
    for command in ["lint", "check"] {
        assert_eq!(run(&mut h, &[command, "model.p"]), 2);
        let result: serde_json::Value = serde_json::from_str(&h.stdout).unwrap();
        assert_eq!(result["exit_code"], 2);
        assert!(result["error"].as_str().unwrap().contains("permissive-monitor"));
    }
    assert_eq!(run(&mut h, &["lint", "--list-rules"]), 0);
    let result: serde_json::Value = serde_json::from_str(&h.stdout).unwrap();
    assert_eq!(result["rules"].as_array().unwrap().len(), 4);
    h.add_file("model.p", "machine Main { start state S {} }");
    assert_eq!(run(&mut h, &["lint", "model.p", "--json-report", "lint.json"]), 0);
    let result: serde_json::Value = serde_json::from_str(&h.stdout).unwrap();
    assert_eq!(result["findings"], serde_json::json!([]));
    assert_eq!(result["exit_code"], 0);
}

#[test]
fn frozen_agent_models_type_check_and_report_the_expected_weaknesses() {
    for (source, expected) in [
        (include_str!("../../snappea-lint/tests/models/agent-p01.p"), "tautological-assert"),
        (include_str!("../../snappea-lint/tests/models/agent-p03.p"), "unused-payload-field"),
        (include_str!("../../snappea-lint/tests/models/agent-p06.p"), "permissive-monitor"),
    ] {
        let mut h = MemHost::new("/w");
        h.add_file("model.p", source);
        assert_eq!(run(&mut h, &["lint", "model.p"]), 2);
        assert!(h.stderr.contains(&format!("error[{expected}]")), "{}{}", h.stdout, h.stderr);
        assert!(h.files_under("PCheckerOutput").is_empty());
    }
}

#[test]
fn lint_and_check_fail_with_same_findings_before_any_output_is_written() {
    let mut h = MemHost::new("/w");
    h.add_file("model.p", MODEL);
    assert_eq!(run(&mut h, &["lint", "model.p"]), 2);
    let lint = h.stderr.clone();
    assert!(lint.contains("model.p:2:6: error[permissive-monitor]"), "{lint}");
    assert_eq!(run(&mut h, &["check", "model.p"]), 2);
    assert_eq!(h.stderr, lint);
    assert!(!h.stdout.contains("Checking "), "{}", h.stdout);
    assert!(h.files_under("PCheckerOutput").is_empty());
}

#[test]
fn scoped_suppression_allows_check_but_does_not_hide_model_bugs() {
    let mut h = MemHost::new("/w");
    let model = MODEL.replace(
        "spec Passive",
        "// snappea-lint: allow permissive-monitor -- intentionally passive fixture\nspec Passive",
    );
    h.add_file("model.p", &model);
    assert_eq!(run(&mut h, &["lint", "model.p"]), 0, "{}{}", h.stdout, h.stderr);
    assert_eq!(h.stdout, "Lint passed.\n");
    assert!(h.files_under("PCheckerOutput").is_empty());
    assert_eq!(run(&mut h, &["check", "model.p"]), 0, "{}{}", h.stdout, h.stderr);
    h.add_file(
        "model.p",
        &model
            .replace("machine Main { start state S {} }", "machine Main { start state S { entry { assert false; } } }"),
    );
    assert_eq!(run(&mut h, &["check", "model.p"]), 1, "{}{}", h.stdout, h.stderr);
}

#[test]
fn source_discovery_pproj_and_parse_errors_work_without_running_schedules() {
    let mut h = MemHost::new("/w");
    h.add_file(
        "Model.pproj",
        "<Project><ProjectName>Model</ProjectName><InputFiles><PFile>model.p</PFile></InputFiles></Project>",
    );
    h.add_file("model.p", MODEL);
    assert_eq!(run(&mut h, &["lint"]), 2);
    assert!(h.stderr.contains("permissive-monitor"));
    assert_eq!(run(&mut h, &["lint", "Model.pproj"]), 2);
    assert!(h.stderr.contains("permissive-monitor"));
    h.add_file("model.p", "machine Main { broken");
    assert_eq!(run(&mut h, &["lint", "model.p"]), 2);
    assert!(!h.stdout.contains("Lint passed"));
    h.add_file("model.p", "machine Main { start state S { entry { assert missing; } } }");
    assert_eq!(run(&mut h, &["lint", "model.p"]), 2);
    assert!(!h.stdout.contains("Lint passed"));
}

#[test]
fn help_rules_and_bad_options_do_not_need_sources() {
    let mut h = MemHost::new("/empty");
    assert_eq!(run(&mut h, &["lint", "--list-rules"]), 0);
    assert_eq!(h.stdout.lines().count(), 4);
    assert!(h.stdout.contains("unused-payload-field"));
    assert_eq!(run(&mut h, &["help", "lint"]), 0);
    assert!(h.stdout.contains("Usage: snappea lint"));
    assert_eq!(run(&mut h, &["lint", "--schedules", "1"]), 2);
    assert_eq!(run(&mut h, &["lint", "testName"]), 2);
    assert_eq!(run(&mut h, &["lint", "--list-rules", "model.p"]), 2);
}

#[test]
fn upstream_compatibility_does_not_apply_lint_policy() {
    let mut h = MemHost::new("/w");
    h.add_file("model.p", MODEL);
    let args = ["check", "--pfiles", "model.p", "--schedules", "1"].map(str::to_string);
    assert_eq!(snappea::compat::run(&args, &mut h), 0, "{}{}", h.stdout, h.stderr);
    assert!(!h.stdout.contains("permissive-monitor") && !h.stderr.contains("permissive-monitor"));
}
