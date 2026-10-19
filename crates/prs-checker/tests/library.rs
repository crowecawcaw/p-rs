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

#[test]
fn unknown_events_in_batch_and_live_observation() {
    use prs_checker::observe::{
        ErrorKind, NoScheduling, ObserveOptions, ObserveSession, ObservedEvent, ParsedEntry, Parser, Spec, Timestamp,
        UnknownEvents,
    };
    use prs_checker::value::Value;
    /// Entries are event names; undeclared ones are reported, not dropped silently.
    struct Names;
    impl Parser for Names {
        fn name(&self) -> &str {
            "Names"
        }
        fn parse(&self, entry: &str, spec: &Spec) -> Result<Vec<ObservedEvent>, String> {
            self.parse_entry(entry, spec).map(|p| p.events)
        }
        fn parse_entry(&self, entry: &str, spec: &Spec) -> Result<ParsedEntry, String> {
            Ok(match spec.event_id(entry) {
                Some(_) => ParsedEntry {
                    events: vec![spec.event("k", Timestamp::default(), entry, Value::Null, None)?],
                    unknown_events: Vec::new(),
                },
                None => ParsedEntry { events: Vec::new(), unknown_events: vec![entry.to_string()] },
            })
        }
    }
    let mut sm = prs_syntax::SourceMap::new();
    sm.add("spec.p", "event eTick; event eOther; spec Ticks observes eTick { start state S { on eTick do {} } }");
    let analysis = prs_sema::compile(&sm, &prs_sema::Options::default()).expect("compiles");
    let spec = Spec::new(&analysis.program, "Ticks").unwrap();
    let input = vec![vec!["eTick".into(), "eOther".into(), "eTikc".into(), "eTikc".into(), "eZebra".into()]];
    let mut clock = || 0.0;
    let opts =
        |unknown_events| ObserveOptions { unknown_events, input_names: vec!["run.log".into()], ..Default::default() };

    let ignored = spec.observe(&input, &Names, &opts(UnknownEvents::Ignore), &mut clock);
    assert!(ignored.errors.is_empty() && ignored.warnings.is_empty() && ignored.unknown_events.is_empty());

    let warned = spec.observe(&input, &Names, &opts(UnknownEvents::Warn), &mut clock);
    assert_eq!(warned.exit_status(), 0);
    assert_eq!(
        warned.warnings,
        [
            "Unknown event 'eTikc': not declared in the P sources (2 occurrences, first at run.log:3); did you mean eTick?",
            "Unknown event 'eZebra': not declared in the P sources (1 occurrence, first at run.log:5)",
        ]
    );

    let failed = spec.observe(&input, &Names, &opts(UnknownEvents::Error), &mut clock);
    assert_eq!(failed.exit_status(), 3);
    assert_eq!(failed.errors.len(), 2);
    assert!(failed.errors.iter().all(|e| e.kind == ErrorKind::UnknownEvent));
    assert_eq!(failed.metrics.verified_events(), 1, "known events are still checked");

    // live: error stops at the first unknown name; warn collects until finish
    let mut strategy = NoScheduling;
    let mut session = ObserveSession::new(&spec, opts(UnknownEvents::Error), &mut strategy, 0.0);
    assert!(session.push("eTick", &Names, &mut clock));
    assert!(session.push("eOther", &Names, &mut clock));
    assert!(!session.push("eTikc", &Names, &mut clock));
    assert!(!session.push("eTick", &Names, &mut clock));
    let report = session.finish_after_failure(0.0);
    assert_eq!(report.errors.len(), 1);
    assert_eq!(report.unknown_events[0].location, "run.log:3");
    assert_eq!(report.unknown_events[0].suggestion.as_deref(), Some("eTick"));

    let mut strategy = NoScheduling;
    let mut session = ObserveSession::new(&spec, opts(UnknownEvents::Warn), &mut strategy, 0.0);
    for entry in ["eTikc", "eTick", "eTikc"] {
        assert!(session.push(entry, &Names, &mut clock));
    }
    let report = session.finish(true, 0.0);
    assert!(report.errors.is_empty());
    assert_eq!(report.unknown_events[0].count, 2);
    assert_eq!(report.warnings.len(), 1);
}
