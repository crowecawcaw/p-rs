// Inventory-specific event mapping and fixtures; prs::test does the checking.
#![allow(dead_code)]

use inventory_integration_example::{Event, Inventory};
use prs::test::{json, EventSink, Model, Recording};
use std::sync::LazyLock;

pub type TestInventory = Inventory<Box<dyn FnMut(Event)>>;

/// Compiled once from `[package.metadata.prs]` in Cargo.toml.
pub fn model() -> &'static Model {
    static MODEL: LazyLock<Model> = LazyLock::new(|| Model::load().expect("inventory specification must compile"));
    &MODEL
}

pub fn inventory(sink: EventSink) -> TestInventory {
    Inventory::new(Box::new(move |event| record(&sink, event)))
}

pub fn record(sink: &EventSink, event: Event) {
    let result = match event {
        Event::Stocked { sku, units } => sink.emit("eStocked", sku, json!({"units": units})),
        Event::Reserved { sku, order, units } => sink.emit("eReserved", sku, json!({"order": order, "units": units})),
        Event::Released { sku, order, units } => sink.emit("eReleased", sku, json!({"order": order, "units": units})),
    };
    result.expect("fixture must emit before its recording is checked");
}

/// Records from before seeding until `test` returns, then checks, panicking on a
/// violation. Nothing is checked while a panic unwinds, so finish cleanup inside
/// `test`. Each check merges monitor coverage into target/coverage/monitor.json.
pub fn with_inventory(test: impl FnOnce(&mut TestInventory)) {
    static FIXTURE: LazyLock<Model> = LazyLock::new(|| {
        model().clone().coverage(concat!(env!("CARGO_MANIFEST_DIR"), "/target/coverage/monitor.json"))
    });
    FIXTURE.check(|recording: &Recording| {
        let mut inventory = inventory(recording.sink());
        inventory.stock("coffee", 3).unwrap();
        inventory.stock("tea", 2).unwrap();
        test(&mut inventory);
    });
}
