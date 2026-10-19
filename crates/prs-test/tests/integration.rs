use prs_test::{json, Model, Suite};

const SPEC: &str = r#"
event Stock: int;
event Pair: (order: string, units: int);
event Done;
event Unobserved;
spec Inventory observes Stock, Pair, Done {
  var stock: int;
  start state Ready {
    on Stock do (n: int) { assert n >= 0, "negative stock"; stock = stock + n; }
    on Pair do (p: (order: string, units: int)) {
      stock = stock - p.units;
      assert stock >= 0, "oversold";
    }
    on Done goto Complete;
  }
  state Complete { }
}
"#;
fn model() -> Model {
    Model::compile(&[("inventory.p", SPEC)], "Inventory").unwrap()
}

#[test]
fn scalar_named_and_ordinary_tuple_and_unit_payloads() {
    let model = model();
    for pair in [json!({"order":"a","units":1}), json!(["a", 1])] {
        let recording = model.recording();
        recording.emit("coffee", "Stock", json!(2)).unwrap();
        recording.emit("coffee", "Pair", pair).unwrap();
        recording.emit("coffee", "Done", json!(null)).unwrap();
        let checked = recording.check().unwrap();
        assert!(checked.passed());
        assert_eq!(checked.events_read(), 3);
        assert_eq!(checked.verified_events(), 3);
    }
    let source = "event Pos: (int, string); spec S observes Pos { start state A { on Pos do {} } }";
    let model = Model::compile(&[("positional.p", source)], "S").unwrap();
    let recording = model.recording();
    recording.emit("key", "Pos", json!([1, "x"])).unwrap();
    assert_eq!(recording.check().unwrap().verified_events(), 1);
}

#[test]
fn invalid_mappings_stay_recorded_and_fail_with_parser_details() {
    for (name, payload) in [
        ("Typo", json!(1)),
        ("Stock", json!("wrong")),
        ("Stock", json!(null)),
        ("Done", json!(1)),
        ("Unobserved", json!(null)),
        ("Pair", json!({"units":1})),
    ] {
        let model = model();
        let recording = model.recording();
        recording.emit("coffee", name, payload).unwrap();
        let failure = recording.check().unwrap_err();
        assert!(!failure.report().passed());
        assert!(failure.to_string().contains("Parser"), "{failure}");
        assert_eq!(failure.events().len(), 1);
    }
}

#[test]
fn strict_empty_optout_and_recording_isolation() {
    assert!(model().recording().check().unwrap_err().to_string().contains("Validation"));
    assert!(model().strict(false).recording().check().is_ok());
    let model = model();
    let first = model.recording();
    first.emit("coffee", "Stock", json!(3)).unwrap();
    first.check().unwrap();
    let second = model.recording();
    second.emit("coffee", "Pair", json!(["order", 1])).unwrap();
    assert!(second.check().unwrap_err().to_string().contains("oversold"));
}

#[test]
fn failed_report_exposes_prefix_coverage_and_replay() {
    let model = model();
    let recording = model.recording();
    recording.emit("coffee", "Stock", json!(1)).unwrap();
    recording.emit("coffee", "Pair", json!(["order", 2])).unwrap();
    let error = recording.check().unwrap_err();
    assert!(error.to_string().contains("Replay for key coffee"));
    assert!(error.to_string().contains("oversold"));
    assert_eq!(error.report().exit_status(), 2);
    let mut suite = Suite::new();
    suite.add(error.report()).unwrap();
    assert_eq!(suite.to_json()["observations"][0]["exit_code"], 2);
    assert!(suite.render_text().contains("failed: 1"));
}

#[test]
fn bad_source_and_selector_have_actionable_diagnostics() {
    let error = Model::compile(&[("broken.p", "spec {")], "Inventory").unwrap_err().to_string();
    assert!(error.contains("broken.p"), "{error}");
    assert!(Model::compile(&[("inventory.p", SPEC)], "Missing").unwrap_err().to_string().contains("Missing"));
    let error =
        Model::compile(&[("types.p", "event E; spec S observes E { start state A { entry { absent = 1; } } }")], "S")
            .unwrap_err()
            .to_string();
    assert!(error.contains("types.p"), "{error}");
    assert!(error.contains("absent"), "{error}");
}

#[test]
fn late_events_rejected_and_cloned_sinks_work_across_threads() {
    fn require_send_sync<T: Send + Sync>() {}
    require_send_sync::<prs_test::EventSink>();
    let model = model();
    let recording = model.recording();
    let handles: Vec<_> = (0..4)
        .map(|i| {
            let sink = recording.sink();
            std::thread::spawn(move || sink.emit(format!("product-{i}"), "Stock", json!(1)).unwrap())
        })
        .collect();
    for handle in handles {
        handle.join().unwrap();
    }
    let late = recording.sink();
    assert_eq!(recording.check().unwrap().verified_events(), 4);
    assert!(late.emit("coffee", "Stock", json!(1)).unwrap_err().to_string().contains("closed"));
}

#[test]
fn explicit_times_preserve_order_and_hot_check_is_independent() {
    let model = model();
    let recording = model.recording();
    recording.emit_at("coffee", "Stock", json!(1), 20).unwrap();
    recording.emit_at("coffee", "Stock", json!(1), 10).unwrap();
    assert!(recording.check().unwrap_err().to_string().contains("OutOfOrder"));
    let source = "event E; spec S observes E { start hot state Busy { on E do {} } }";
    let model = Model::compile(&[("hot.p", source)], "S").unwrap().check_hot(true);
    let recording = model.recording();
    recording.emit("key", "E", json!(null)).unwrap();
    assert!(recording.check().is_err());
}

#[test]
fn coverage_rejects_changed_sources_atomically_and_writes_nested_paths() {
    let model = model();
    let record = model.recording();
    record.emit("coffee", "Stock", json!(1)).unwrap();
    let mut suite = Suite::new();
    suite.add(&record.check().unwrap()).unwrap();
    let before = suite.to_json();
    let changed = SPEC.replace("n >= 0", "n >= 1");
    let changed = Model::compile(&[("inventory.p", &changed)], "Inventory").unwrap();
    let record = changed.recording();
    record.emit("coffee", "Stock", json!(1)).unwrap();
    assert!(suite.add(&record.check().unwrap()).unwrap_err().to_string().contains("fingerprint"));
    assert_eq!(suite.to_json(), before);
    let root = std::env::temp_dir().join(format!("prs-test-report-{}", std::process::id()));
    let path = root.join("nested/coverage.json");
    suite.write_json(&path).unwrap();
    assert!(std::fs::read_to_string(&path).unwrap().contains("observed_monitor_suite"));
    assert!(suite.write_json(&root).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn closure_checks_after_cleanup_and_preserves_primary_panic() {
    let model = model();
    let (answer, report) = model
        .with_recording(|sink| {
            sink.emit("coffee", "Stock", json!(1)).unwrap();
            sink.emit("coffee", "Done", json!(null)).unwrap();
            42
        })
        .unwrap();
    assert_eq!(answer, 42);
    assert_eq!(report.verified_events(), 2);
    let panic = std::panic::catch_unwind(|| {
        let _ = model.with_recording(|_| panic!("primary application failure"));
    })
    .unwrap_err();
    assert_eq!(panic.downcast_ref::<&str>(), Some(&"primary application failure"));
}
