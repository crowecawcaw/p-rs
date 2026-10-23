use snappea_lint::{lint, Context, Finding, Registry, Rule};
use snappea_syntax::SourceMap;

fn findings(files: &[&str]) -> Vec<Finding> {
    let mut sm = SourceMap::new();
    for (i, text) in files.iter().enumerate() {
        sm.add(format!("model{i}.p"), *text);
    }
    lint(&sm).unwrap()
}
fn has(text: &str, id: &str) -> bool {
    findings(&[text]).iter().any(|f| f.rule == id)
}
const PASSIVE: &str = "event E; spec Passive observes E { start state S { on E goto S; } }";
const SAFE: &str = "event E: (actual: int, expected: int); spec Guard observes E { start state S { on E do (p: (actual: int, expected: int)) { assert p.actual == p.expected; } } } machine Main { start state S {} } test t [main=Main]: assert Guard in {Main};";

#[test]
fn frozen_agent_models_expose_the_observed_bad_patterns() {
    let p01 = findings(&[include_str!("models/agent-p01.p")]);
    assert!(p01.iter().any(|f| f.rule == "tautological-assert" && f.span.start > 0));
    assert!(p01.iter().any(|f| f.rule == "unselected-spec"));
    let p03 = findings(&[include_str!("models/agent-p03.p")]);
    assert!(p03.iter().any(|f| f.rule == "unused-payload-field" && f.message.contains("p.digest")));
    let p06 = findings(&[include_str!("models/agent-p06.p")]);
    assert!(p06.iter().any(|f| f.rule == "permissive-monitor" && f.message.contains("BundleTrace")));
    assert!(p06.iter().any(|f| f.rule == "unselected-spec" && f.message.contains("AtomicCommitCheck")));
}

#[test]
fn independent_observation_monitor_is_clean() {
    assert!(findings(&[SAFE]).is_empty());
}

#[test]
fn permissive_requires_total_handling_and_no_progress_obligation() {
    assert!(has(PASSIVE, "permissive-monitor"));
    assert!(has(
        "event E; spec S observes E { start state A { on E goto B; } state B { ignore E; } }",
        "permissive-monitor"
    ));
    assert!(!has("event E; spec S observes E { start state A { on E goto B; } state B {} }", "permissive-monitor"));
    assert!(!has("event E; spec S observes E { start hot state A { on E goto A; } }", "permissive-monitor"));
    assert!(!has("event E; spec S observes E { start state A { on E do { assert false; } } }", "permissive-monitor"));
}

#[test]
fn named_handlers_are_inspected() {
    assert!(has(
        "event E; spec S observes E { fun Empty() {} start state A { on E do Empty; } }",
        "permissive-monitor"
    ));
    assert!(!has(
        "event E; spec S observes E { fun Guard() { assert false; } start state A { on E do Guard; } }",
        "permissive-monitor"
    ));
    let text = "type Payload = (digest: string, rev: int); event E: Payload; spec S observes E { fun Guard(p: Payload) { assert p.rev == 1; } start state A { on E do Guard; } }";
    assert!(findings(&[text]).iter().any(|f| f.rule == "unused-payload-field" && f.message.contains("digest")));
}

#[test]
fn selection_resolves_modules_across_files_and_runnable_implementations() {
    let spec =
        "event E; spec S observes E { start state A { on E do { assert false; } } } machine Main { start state A {} }";
    let module = "module Base = { Main }; module Guarded = assert S in Base;";
    assert!(!findings(&[spec, module, "test t [main=Main]: Guarded;"]).iter().any(|f| f.rule == "unselected-spec"));
    assert!(!findings(&[spec, module, "implementation i [main=Main]: Guarded;"])
        .iter()
        .any(|f| f.rule == "unselected-spec"));
    assert!(findings(&[spec, module, "test t [main=Main]: Base;"]).iter().any(|f| f.rule == "unselected-spec"));
}

fn block(body: &str) -> String {
    format!("machine Main {{ var x: int; var y: int; fun Mutate(): bool {{ x = 3; return true; }} start state S {{ entry {{ {body} }} }} }}")
}

#[test]
fn tautology_tracks_scalar_values_and_forgets_uncertain_writes() {
    assert!(has(&block("x = y; assert x == y;"), "tautological-assert"));
    assert!(has(&block("x = 1; y = x; assert x == y;"), "tautological-assert"));
    assert!(has(&block("assert true;"), "tautological-assert"));
    assert!(!has(&block("x = y; y = 3; assert x == y;"), "tautological-assert"));
    assert!(!has(&block("x = 1; x = y; assert x == 1;"), "tautological-assert"));
    assert!(!has(&block("x = 1; if ($) { x = 2; } assert x == 1;"), "tautological-assert"));
    assert!(!has(&block("x = 1; if (Mutate()) { assert x == 1; }"), "tautological-assert"));
    assert!(!has(&block("x = 1; while ($) { x = 2; } assert x == 1;"), "tautological-assert"));
    assert!(!has("machine Main { var x: float; start state S { entry { assert x == x; } } }", "tautological-assert"));
    assert!(!has(&block("assert false;"), "tautological-assert"));
}

#[test]
fn tuple_assignments_do_not_make_payload_look_read_and_aliases_are_conservative() {
    let model = |body| {
        format!("event E: (a: int, b: int); spec S observes E {{ start state A {{ on E do (p: (a: int, b: int)) {{ {body} }} }} }}")
    };
    let fs = findings(&[&model("p.b = 1; assert p.a == 2;")]);
    assert!(fs.iter().any(|f| f.rule == "unused-payload-field" && f.message.contains("p.b")));
    assert!(!has(&model("var copy: (a: int, b: int); copy = p; assert copy.a == 2;"), "unused-payload-field"));
}

#[test]
fn suppression_is_scoped_and_unknown_ids_fail() {
    let suppressed = format!("// snappea-lint: allow permissive-monitor, unselected-spec -- trace fixture\n{PASSIVE}");
    // The comment precedes event E, not the spec, so it must not silently work.
    assert!(has(&suppressed, "invalid-suppression"));
    let suppressed = "event E;\n// snappea-lint: allow permissive-monitor, unselected-spec -- trace fixture\nspec Passive observes E { start state S { on E goto S; } }";
    assert!(findings(&[suppressed]).is_empty());
    let text = block("// snappea-lint: allow tautological-assert -- test fixture\nassert true; assert true;");
    // Directive is trailing code on its line: it must fail rather than over-suppress.
    assert!(has(&text, "invalid-suppression"));
    let text = block("\n// snappea-lint: allow tautological-assert -- fixture\nassert true;\nassert true;");
    assert_eq!(findings(&[&text]).iter().filter(|f| f.rule == "tautological-assert").count(), 1);
    assert!(has("// snappea-lint: allow typo\nmachine Main { start state S {} }", "invalid-suppression"));
    assert!(!has(&block("print \"// snappea-lint: allow typo\";"), "invalid-suppression"));
    assert!(!has("/* // snappea-lint: allow typo */ machine Main { start state S {} }", "invalid-suppression"));
}

#[test]
fn external_rules_share_the_same_suppression_and_reporting_pipeline() {
    struct Example;
    impl Rule for Example {
        fn id(&self) -> &'static str {
            "example-rule"
        }
        fn description(&self) -> &'static str {
            "Example extension"
        }
        fn check(&self, ctx: &Context<'_>, out: &mut Vec<Finding>) {
            for m in ctx.machines() {
                out.push(Finding::new(self.id(), m.name.span, "example"));
            }
        }
    }
    let mut registry = Registry::default();
    registry.register(Example);
    let mut sm = SourceMap::new();
    sm.add("test.p", "// snappea-lint: allow example-rule -- fixture\nmachine Main { start state S {} }");
    assert!(registry.check(&Context::parse(&sm).unwrap()).is_empty());
}
