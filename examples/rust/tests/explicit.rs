mod support;

use inventory_integration_example::{Error, Event};
use support::Recorder;

#[test]
fn selected_test_checks_its_own_recording() {
    let recorder = Recorder::default();
    let mut inventory = recorder.inventory();
    // Seed after connecting the recorder: the monitor needs initial stock.
    inventory.stock("coffee", 3).unwrap();
    inventory.reserve("coffee", "order-1", 2).unwrap();
    inventory.release("coffee", "order-1").unwrap();
    assert_eq!(inventory.available("coffee"), Some(3));
    recorder.check().unwrap();
}

#[test]
fn rejected_requests_emit_nothing() {
    let recorder = Recorder::default();
    let mut inventory = recorder.inventory();
    inventory.stock("coffee", 3).unwrap();
    let before = recorder.len();
    assert_eq!(inventory.reserve("coffee", "large", 4), Err(Error::InsufficientStock));
    assert_eq!(inventory.reserve("coffee", "zero", 0), Err(Error::InvalidQuantity));
    assert_eq!(inventory.release("coffee", "missing"), Err(Error::UnknownOrder));
    assert_eq!(recorder.len(), before);
    recorder.check().unwrap();
}

#[test]
fn negative_control_detects_overselling() {
    let recorder = Recorder::default();
    let mut inventory = recorder.inventory();
    inventory.stock("coffee", 3).unwrap();
    inventory.reserve("coffee", "order-1", 2).unwrap();
    // Simulate the event an overselling regression would produce.
    recorder.record(Event::Reserved { sku: "coffee".into(), order: "order-2".into(), units: 2 });
    let error = recorder.check().expect_err("the spec must reject the broken trace");
    assert!(error.contains("reservation exceeds available stock"), "{error}");
}
