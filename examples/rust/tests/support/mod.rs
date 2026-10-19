// Helpers are shared by independent integration-test targets.
#![allow(dead_code)]

use inventory_integration_example::{Event, Inventory};
use prs_checker::observe::{ObserveOptions, ObservedEvent, Parser, Spec, Timestamp};
use prs_checker::value::Value;
use std::{cell::RefCell, rc::Rc};

pub type TestInventory = Inventory<Box<dyn FnMut(Event)>>;

#[derive(Clone, Default)]
pub struct Recorder(Rc<RefCell<Vec<Event>>>);

impl Recorder {
    pub fn inventory(&self) -> TestInventory {
        let events = self.0.clone();
        Inventory::new(Box::new(move |event| events.borrow_mut().push(event)))
    }

    pub fn record(&self, event: Event) {
        self.0.borrow_mut().push(event);
    }

    pub fn len(&self) -> usize {
        self.0.borrow().len()
    }

    pub fn check(&self) -> Result<(), String> {
        self.check_with_coverage(false)
    }

    fn check_with_coverage(&self, coverage: bool) -> Result<(), String> {
        let mut sources = prs_syntax::SourceMap::new();
        sources.add("inventory.p", include_str!("../../specs/inventory.p"));
        let analysis = prs_sema::compile(&sources, &Default::default())
            .map_err(|errors| format!("spec compilation failed: {errors:?}"))?;
        let spec = Spec::new(&analysis.program, "InventoryCorrect")?;
        let events = self.0.borrow();
        if events.is_empty() {
            return Err("no events recorded: check the application's event sink wiring".into());
        }
        // The observe API consumes log entries through a Parser. Here the entries
        // are indexes into typed, in-memory domain events; there is no JSON or CLI.
        let entries = (0..events.len()).map(|i| i.to_string()).collect();
        let report = spec.observe(
            &[entries],
            &DomainEvents(&events),
            &ObserveOptions { assume_sorted: true, coverage, strict: true, ..Default::default() },
            &mut || 0.0,
        );
        if coverage {
            publish_coverage(&report)?;
        }
        // Include parser errors: Report::error_count() intentionally excludes them.
        if !report.errors.is_empty() {
            return Err(format!("P inventory check failed:\n{:#?}\nDomain events: {events:#?}", report.errors));
        }
        if report.metrics.verified_events() != events.len() as u64 {
            return Err("not all recorded events reached the monitor".into());
        }
        Ok(())
    }
}

struct DomainEvents<'a>(&'a [Event]);

impl Parser for DomainEvents<'_> {
    fn name(&self) -> &str {
        "inventory-domain-events"
    }

    fn parse(&self, entry: &str, spec: &Spec) -> Result<Vec<ObservedEvent>, String> {
        let index: usize = entry.parse().map_err(|e| format!("invalid event index: {e}"))?;
        let event = self.0.get(index).ok_or("missing domain event")?;
        let (sku, name, values) = match event {
            Event::Stocked { sku, units } => (sku, "eStocked", vec![Value::Int(*units)]),
            Event::Reserved { sku, order, units } => (sku, "eReserved", vec![Value::str(order), Value::Int(*units)]),
            Event::Released { sku, order, units } => (sku, "eReleased", vec![Value::str(order), Value::Int(*units)]),
        };
        // Logical sequence numbers preserve application order without clocks/sleeps.
        spec.tuple_event(sku, Timestamp::from_millis(index as i64), name, values, Some(entry)).map(|e| vec![e])
    }
}

/// Recording starts before seeding, includes the whole body and any explicit
/// cleanup in it, then propagates monitor failure as an ordinary cargo-test panic.
pub fn with_inventory(test: impl FnOnce(&mut TestInventory)) {
    let recorder = Recorder::default();
    let mut inventory = recorder.inventory();
    inventory.stock("coffee", 3).unwrap();
    inventory.stock("tea", 2).unwrap();
    test(&mut inventory);
    recorder.check_with_coverage(true).unwrap_or_else(|message| panic!("{message}"));
}

// Integration tests run concurrently. Hold the lock across merge, write and print,
// so the last published report is the union of all completed fixture checks.
fn publish_coverage(report: &prs_checker::observe::Report) -> Result<(), String> {
    use prs_checker::observe_report::ObserveCoverageReport;
    use std::sync::{Mutex, OnceLock};
    static SUITE: OnceLock<Mutex<ObserveCoverageReport>> = OnceLock::new();
    let mut aggregate =
        SUITE.get_or_init(|| Mutex::new(ObserveCoverageReport::default())).lock().map_err(|e| e.to_string())?;
    aggregate.merge(&ObserveCoverageReport::from_observation("InventoryCorrect", report)?)?;
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/coverage/model.json");
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::write(&path, serde_json::to_string_pretty(&aggregate.to_json()).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    println!(
        "P model coverage (union of completed fixture checks so far):\n{}\nJSON: {}",
        aggregate.render_text(),
        path.display()
    );
    Ok(())
}
