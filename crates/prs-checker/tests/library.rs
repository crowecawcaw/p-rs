//! The example in `docs/library.md`, kept compiling and running.

use prs_checker::engine::{Checker, CheckerOptions};

const MODEL: &str = "
    event ePing;
    machine Main {
        start state Init {
            entry { send this, ePing; }
            on ePing do { assert false, \"pinged\"; }
        }
    }
    test tcMain [main = Main]: { Main };
";

#[test]
fn library_example() {
    let mut sm = prs_syntax::SourceMap::new();
    sm.add("model.p", MODEL);
    let analysis = prs_sema::compile(&sm, &prs_sema::Options::default()).expect("compiles");
    let tc = analysis.program.test_cases().into_iter().find(|t| t.name == "tcMain").unwrap();
    let checker = Checker::new(&analysis.program);
    let opts = CheckerOptions { iterations: 1000, seed: 42, ..Default::default() };
    let result = checker.check(&tc, &opts, &mut Vec::<String>::new()).expect("runs");
    assert_eq!(result.report.num_bugs, 1);
}

#[test]
fn observe_spec_by_name() {
    let mut sm = prs_syntax::SourceMap::new();
    sm.add("spec.p", "event eTick; spec MySpec observes eTick { start state S { on eTick do {} } }");
    let analysis = prs_sema::compile(&sm, &prs_sema::Options::default()).expect("compiles");
    assert!(prs_checker::observe::Spec::new(&analysis.program, "MySpec").is_ok());
    assert!(prs_checker::observe::Spec::new(&analysis.program, "Missing").is_err());
}

#[test]
fn observed_coverage_unions_real_monitor_runs() {
    use prs_checker::observe::{ObserveOptions, ObservedEvent, Parser, Spec, Timestamp};
    use prs_checker::value::Value;
    struct Events;
    impl Parser for Events {
        fn name(&self) -> &str {
            "Events"
        }
        fn parse(&self, entry: &str, spec: &Spec) -> Result<Vec<ObservedEvent>, String> {
            Ok(vec![spec.event("key", Timestamp::default(), entry, Value::Null, None)?])
        }
    }
    let mut sm = prs_syntax::SourceMap::new();
    sm.add("spec.p", "event eLeft; event eRight; spec Choice observes eLeft,eRight { start state Ready { on eLeft goto Left; on eRight do { goto Right; } } state Left {} state Right {} }");
    let analysis = prs_sema::compile(&sm, &prs_sema::Options::default()).expect("compiles");
    let spec = Spec::new(&analysis.program, "Choice").unwrap();
    let options = ObserveOptions { coverage: true, ..Default::default() };
    let mut clock = || 0.0;
    let mut left = spec.observe(&[vec!["eLeft".into()]], &Events, &options, &mut clock).coverage.unwrap();
    let right = spec.observe(&[vec!["eRight".into()]], &Events, &options, &mut clock).coverage.unwrap();
    assert_eq!(left.visited_states.len(), 2);
    assert!(left.render_text().contains("Unvisited state: Choice.Right"));
    left.merge(&right);
    left.merge(&right);
    assert_eq!(left.to_json()["states"]["covered"], 3);
    assert_eq!(left.to_json()["handlers"]["covered"], 2);
    assert_eq!(left.transitions.len(), 2);
    assert!(!left.render_text().contains("Unvisited state:"));
    assert!(!left.render_text().contains("Uncovered handler:"));
}

#[test]
fn strict_live_prefix_cannot_be_success_without_eof() {
    use prs_checker::observe::{ErrorKind, NoScheduling, ObserveOptions, ObserveSession, Spec};
    let mut sm = prs_syntax::SourceMap::new();
    sm.add("spec.p", "event e; spec S observes e { start state Ready { on e do {} } }");
    let analysis = prs_sema::compile(&sm, &prs_sema::Options::default()).unwrap();
    let spec = Spec::new(&analysis.program, "S").unwrap();
    let opts = ObserveOptions { strict: true, coverage: true, ..Default::default() };
    let mut strategy = NoScheduling;
    let session = ObserveSession::new(&spec, opts.clone(), &mut strategy, 0.0);
    let report = session.finish(false, 0.0);
    assert_eq!(report.errors.len(), 1);
    assert_eq!(report.errors[0].kind, ErrorKind::Validation);
    assert!(report.errors[0].message.contains("before EOF"));
    let suite = prs_checker::observe_report::ObserveCoverageReport::from_observation("S", &report).unwrap();
    let metadata = &suite.to_json()["observations"][0];
    assert_eq!(metadata["exit_code"], 3);
    assert_eq!(metadata["specs"][0]["errors"][0]["kind"], "validation");
    assert!(metadata["specs"][0]["errors"][0]["message"].as_str().unwrap().contains("before EOF"));
    let roundtrip = prs_checker::observe_report::ObserveCoverageReport::from_json(&suite.to_json()).unwrap();
    assert_eq!(roundtrip.to_json(), suite.to_json());
    let mut strategy = NoScheduling;
    let session = ObserveSession::new(&spec, opts, &mut strategy, 0.0);
    assert!(session.finish_after_failure(0.0).errors.is_empty());
    let mut strategy = NoScheduling;
    let session = ObserveSession::new(&spec, ObserveOptions::default(), &mut strategy, 0.0);
    assert!(session.finish(false, 0.0).errors.is_empty());
}
