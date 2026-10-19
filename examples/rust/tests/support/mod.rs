// Inventory-specific event mapping and fixtures; prs-test does the checking.
#![allow(dead_code)]

use inventory_integration_example::{Event, Inventory};
use prs_test::{json, CheckReport, EventSink, Model, Suite};
use std::sync::{LazyLock, Mutex};

pub type TestInventory = Inventory<Box<dyn FnMut(Event)>>;

pub fn model() -> &'static Model {
    static MODEL: LazyLock<Model> = LazyLock::new(|| {
        Model::compile(&[("inventory.p", include_str!("../../specs/inventory.p"))], "InventoryCorrect")
            .expect("inventory specification must compile")
    });
    &MODEL
}

pub fn inventory(sink: EventSink) -> TestInventory {
    Inventory::new(Box::new(move |event| record(&sink, event)))
}

pub fn record(sink: &EventSink, event: Event) {
    let result = match event {
        Event::Stocked { sku, units } => sink.emit(sku, "eStocked", json!({"units": units})),
        Event::Reserved { sku, order, units } => sink.emit(sku, "eReserved", json!({"order":order,"units":units})),
        Event::Released { sku, order, units } => sink.emit(sku, "eReleased", json!({"order":order,"units":units})),
    };
    result.expect("fixture must emit before its recording is checked");
}

/// Records from before seeding until `test` returns, then checks. Nothing is
/// checked while a panic unwinds, so finish cleanup inside `test`.
pub fn with_inventory(test: impl FnOnce(&mut TestInventory)) {
    let recording = model().recording();
    let mut inventory = inventory(recording.sink());
    inventory.stock("coffee", 3).unwrap();
    inventory.stock("tea", 2).unwrap();
    test(&mut inventory);
    let checked = recording.check();
    // A failed check still has coverage for the events before the failure.
    let coverage = publish_coverage(checked.as_ref().unwrap_or_else(|error| error.report()));
    checked.unwrap();
    coverage.expect("coverage reporting failed");
}

// Cargo runs tests in parallel, so the suite is shared behind a mutex.
fn publish_coverage(report: &CheckReport) -> Result<(), Box<dyn std::error::Error>> {
    static SUITE: LazyLock<Mutex<Suite>> = LazyLock::new(Default::default);
    let mut suite = SUITE.lock().map_err(|e| e.to_string())?;
    suite.add(report)?;
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/coverage/model.json");
    suite.write_json(&path)?;
    println!("P model coverage (completed fixture checks so far):\n{}\nJSON: {}", suite.render_text(), path.display());
    Ok(())
}
