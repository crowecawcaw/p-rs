mod support;

use inventory_integration_example::{Error, Event};
use support::{inventory, model, record};

#[test]
fn selected_test_checks_its_own_recording() {
    let recorder = model().recording();
    let mut inventory = inventory(recorder.sink());
    // Seed after connecting the recorder, so the monitor sees initial stock.
    inventory.stock("coffee", 3).unwrap();
    inventory.reserve("coffee", "order-1", 2).unwrap();
    inventory.release("coffee", "order-1").unwrap();
    assert_eq!(inventory.available("coffee"), Some(3));
    recorder.check().unwrap();
}

#[test]
fn rejected_requests_emit_nothing() {
    let recorder = model().recording();
    let mut inventory = inventory(recorder.sink());
    inventory.stock("coffee", 3).unwrap();
    let before = recorder.len();
    assert_eq!(inventory.reserve("coffee", "large", 4), Err(Error::InsufficientStock));
    assert_eq!(inventory.reserve("coffee", "zero", 0), Err(Error::InvalidQuantity));
    assert_eq!(inventory.release("coffee", "missing"), Err(Error::UnknownOrder));
    assert_eq!(recorder.len(), before);
    recorder.check().unwrap();
}

// A negative control: the spec must reject an overselling trace.
#[test]
fn negative_control_detects_overselling() {
    let recorder = model().recording();
    let mut inventory = inventory(recorder.sink());
    inventory.stock("coffee", 3).unwrap();
    inventory.reserve("coffee", "order-1", 2).unwrap();
    record(&recorder.sink(), Event::Reserved { sku: "coffee".into(), order: "order-2".into(), units: 2 });
    let violation = recorder.expect_violation("InventoryCorrect", "exceeds available stock");
    assert_eq!(violation.key, "coffee");
    assert_eq!(violation.events.last().unwrap().payload["order"], "order-2");
}

// Code instrumented with the global `prs_emit::emit` needs no sink: the recording
// binds the test's thread. Bind threads you spawn with `recorder.sink().bind()`.
#[test]
fn global_emit_is_recorded() {
    let recorder = model().recording();
    prs_test::prs_emit::emit("eStocked", "coffee", prs_test::json!({"units": 3}));
    prs_test::prs_emit::emit("eReserved", "coffee", prs_test::json!({"order": "o1", "units": 2}));
    assert_eq!(recorder.check().unwrap().verified_events(), 2);
}
