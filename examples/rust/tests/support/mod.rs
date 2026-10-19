// Application-specific event mapping and fixture setup. prs-test owns compilation,
// recording, monitor checks, diagnostics, and coverage reporting.
#![allow(dead_code)]

use inventory_integration_example::{Event, Inventory};
use prs_test::{json, CheckReport, EventSink, Model, Suite};
use std::sync::{Mutex, OnceLock};

pub type TestInventory = Inventory<Box<dyn FnMut(Event)>>;

pub fn model() -> &'static Model {
    static MODEL: OnceLock<Model> = OnceLock::new();
    MODEL.get_or_init(|| {
        Model::compile(&[("inventory.p", include_str!("../../specs/inventory.p"))], "InventoryCorrect")
            .expect("inventory specification must compile")
    })
}

pub fn inventory(sink: EventSink) -> TestInventory {
    Inventory::new(Box::new(move |event| record(&sink, event)))
}

// Keep this boundary specific to your application's existing domain events.
// JSON supports named tuples, ordinary tuples, scalar payloads, and unit events.
pub fn record(sink: &EventSink, event: Event) {
    let result = match event {
        Event::Stocked { sku, units } => sink.emit(sku, "eStocked", json!({"units": units})),
        Event::Reserved { sku, order, units } => sink.emit(sku, "eReserved", json!({"order":order,"units":units})),
        Event::Released { sku, order, units } => sink.emit(sku, "eReleased", json!({"order":order,"units":units})),
    };
    result.expect("fixture must emit before its recording is checked");
}

/// Recording starts before seeding and includes any explicit cleanup in the body.
/// Normal Rust panics propagate unchanged. Nothing asserts during unwinding.
pub fn with_inventory(test: impl FnOnce(&mut TestInventory)) {
    let recording = model().recording();
    let mut inventory = inventory(recording.sink());
    inventory.stock("coffee", 3).unwrap();
    inventory.stock("tea", 2).unwrap();
    test(&mut inventory);
    let checked = recording.check();
    // Retain failed-prefix coverage before propagating the original monitor failure.
    let report = match &checked {
        Ok(report) => report,
        Err(error) => error.report(),
    };
    let coverage = publish_coverage(report);
    if let Err(error) = &coverage {
        eprintln!("coverage reporting failed: {error}");
    }
    checked.unwrap();
    coverage.unwrap();
}

// Coverage is opt-in: only the shared fixture adds results. Generated cases stay
// independent and do not retain per-case reports. Cargo runs these tests in parallel.
fn publish_coverage(report: &CheckReport) -> Result<(), Box<dyn std::error::Error>> {
    static SUITE: OnceLock<Mutex<Suite>> = OnceLock::new();
    let mut suite = SUITE.get_or_init(|| Mutex::new(Suite::new())).lock().map_err(|e| e.to_string())?;
    suite.add(report)?;
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/coverage/model.json");
    suite.write_json(&path)?;
    println!("P model coverage (completed fixture checks so far):\n{}\nJSON: {}", suite.render_text(), path.display());
    Ok(())
}
