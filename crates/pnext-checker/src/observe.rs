//! Runtime monitoring: checking recorded events of a running system against a spec
//! monitor, as upstream's PObserve does.
//!
//! A [`Parser`] turns log entries into timestamped, keyed [`ObservedEvent`]s. Events the
//! spec does not observe are dropped; the rest are partitioned by key, each partition is
//! sorted by timestamp and fed to a fresh instance of the spec. A failed assertion, an
//! event arriving out of order (when inputs are assumed sorted) or any other error ends the
//! check of that key and is reported with the last events seen for it (the replay window).
//!
//! Specs run with the semantics of upstream's Java backend
//! ([`Config::java_semantics`](crate::runtime::Config::java_semantics)), so messages read
//! as PObserve's do. Like the rest of the crate this module does no I/O: inputs are passed
//! as strings and time comes from a caller-supplied clock.

use std::collections::HashMap;
use std::rc::Rc;

use pnext_sema::{EventId, MachineId, ModuleInfo, PType, Program};

use crate::compile::{compile_program, Code};
use crate::javafmt;
use crate::runtime::{Config, Event, Failure, Runtime};
use crate::strategy::{OpInfo, Strategy};
use crate::value::{NamedTuple, Value};

/// A point in time (`java.time.Instant`): seconds since the epoch plus nanoseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Timestamp {
    pub secs: i64,
    pub nanos: u32,
}

impl Timestamp {
    pub fn new(secs: i64, nanos: u32) -> Timestamp {
        Timestamp { secs: secs + (nanos / 1_000_000_000) as i64, nanos: nanos % 1_000_000_000 }
    }

    /// `Instant.ofEpochMilli`.
    pub fn from_millis(ms: i64) -> Timestamp {
        Timestamp { secs: ms.div_euclid(1000), nanos: (ms.rem_euclid(1000) * 1_000_000) as u32 }
    }

    /// `Instant.toEpochMilli`.
    pub fn to_millis(&self) -> i64 {
        self.secs * 1000 + (self.nanos / 1_000_000) as i64
    }

    /// `Instant.toString()`.
    pub fn to_iso(&self) -> String {
        javafmt::instant_to_string(self.secs, self.nanos)
    }
}

/// One event recovered from the logs (`PObserveEvent`).
#[derive(Debug, Clone)]
pub struct ObservedEvent {
    /// Partition key: events with the same key are checked by one spec instance.
    pub key: String,
    pub timestamp: Timestamp,
    pub event: EventId,
    pub payload: Value,
    /// The log entry the event came from.
    pub log_line: Option<String>,
    /// Extra data shown in reports.
    pub custom_payload: Option<String>,
}

impl ObservedEvent {
    /// `PObserveEvent.toString()`.
    pub fn to_java_string(&self, prog: &Program) -> String {
        format!(
            "PObserveEvent{{partitionKey='{}', timestamp={}, event={}, logLine='{}', customPayload={}}}",
            self.key,
            javafmt::instant_nanos9(self.timestamp.secs, self.timestamp.nanos),
            javafmt::event_to_string(prog, self.event, &self.payload),
            self.log_line.as_deref().unwrap_or("null"),
            self.custom_payload.as_deref().unwrap_or("null"),
        )
    }
}

/// Turns log entries into events (upstream's `Parser` interface).
pub trait Parser {
    /// The name `--parser` selects the parser by.
    fn name(&self) -> &str;

    /// What separates log entries (`getLogDelimiter`): `\n` for one entry per line.
    fn log_delimiter(&self) -> String {
        "\n".into()
    }

    /// Receives `--parserConfiguration` (`setConfiguration`).
    fn set_configuration(&mut self, _configuration: &str) -> Result<(), String> {
        Ok(())
    }

    /// Parses one (trimmed) log entry. Entries that hold no event return no events; an
    /// `Err` counts as a parser error for this entry only.
    fn parse(&self, entry: &str, spec: &Spec) -> Result<Vec<ObservedEvent>, String>;
}

/// A spec monitor ready to check events, plus helpers parsers use to build them.
pub struct Spec<'p> {
    pub prog: &'p Program,
    pub decl: MachineId,
    /// Java package of the generated classes (`<pobserve-package>`), used in reports.
    pub java_package: String,
    code: Code,
    module: ModuleInfo,
    config: Config,
}

impl<'p> Spec<'p> {
    /// The spec machine called `name`.
    pub fn new(prog: &'p Program, name: &str) -> Result<Spec<'p>, String> {
        let decl = prog
            .machines
            .iter()
            .position(|m| m.name == name && m.kind == pnext_sema::MachineKind::Spec)
            .map(|i| MachineId(i as u32))
            .ok_or_else(|| format!("no spec machine named '{}'", name))?;
        Ok(Spec::from_decl(prog, decl))
    }

    pub fn from_decl(prog: &'p Program, decl: MachineId) -> Spec<'p> {
        let config = Config { logging: false, java_semantics: true, max_work_per_step: 0, ..Config::default() };
        Spec {
            prog,
            decl,
            java_package: "pobserve".into(),
            code: compile_program(prog),
            module: ModuleInfo::default(),
            config,
        }
    }

    pub fn name(&self) -> &str {
        &self.prog.machine(self.decl).name
    }

    /// The events the spec observes (`getEventTypes()`), in declaration order.
    pub fn observes(&self) -> &[EventId] {
        &self.prog.machine(self.decl).observes
    }

    /// The event declared as `name`.
    pub fn event_id(&self, name: &str) -> Option<EventId> {
        self.prog.events.iter().position(|e| e.name == name).map(|i| EventId(i as u32))
    }

    /// The payload type of event `e`.
    pub fn payload_type(&self, e: EventId) -> &PType {
        &self.prog.event(e).payload
    }

    /// The value of enum element `elem` of enum `ty`.
    pub fn enum_value(&self, ty: &str, elem: &str) -> Option<Value> {
        let en = self.prog.enums.iter().find(|e| e.name == ty)?;
        en.elems.iter().find(|x| x.name == elem).map(|x| Value::Int(x.value))
    }

    /// Builds an event from its name and payload.
    pub fn event(
        &self,
        key: impl Into<String>,
        timestamp: Timestamp,
        name: &str,
        payload: Value,
        log_line: Option<&str>,
    ) -> Result<ObservedEvent, String> {
        let event = self.event_id(name).ok_or_else(|| format!("unknown event '{}'", name))?;
        Ok(ObservedEvent {
            key: key.into(),
            timestamp,
            event,
            payload,
            log_line: log_line.map(|s| s.to_string()),
            custom_payload: None,
        })
    }

    /// Builds an event whose payload is a tuple, from its field values in declaration order
    /// (field names come from the event's payload type).
    pub fn tuple_event(
        &self,
        key: impl Into<String>,
        timestamp: Timestamp,
        name: &str,
        values: Vec<Value>,
        log_line: Option<&str>,
    ) -> Result<ObservedEvent, String> {
        let id = self.event_id(name).ok_or_else(|| format!("unknown event '{}'", name))?;
        let mut ty = self.payload_type(id);
        while let PType::TypeDef(t) = ty {
            ty = &self.prog.typedefs[t.idx()].ty;
        }
        let payload = match ty {
            PType::NamedTuple(fs) if fs.len() == values.len() => {
                let names: Vec<String> = fs.iter().map(|(n, _)| n.clone()).collect();
                Value::NamedTuple(Rc::new(NamedTuple { names: names.into(), values }))
            }
            PType::Tuple(ts) if ts.len() == values.len() => Value::Tuple(Rc::new(values)),
            _ => return Err(format!("the payload of '{}' is not a tuple of {} fields", name, values.len())),
        };
        self.event(key, timestamp, name, payload, log_line)
    }

    /// `true` if events of this kind reach the spec (upstream's `filterBasedOnSpecObservation`).
    pub fn observes_event(&self, e: EventId) -> bool {
        self.observes().contains(&e)
    }
}

/// Builds a named tuple value.
pub fn named_tuple(fields: Vec<(&str, Value)>) -> Value {
    let names: Vec<String> = fields.iter().map(|(n, _)| n.to_string()).collect();
    Value::NamedTuple(Rc::new(NamedTuple { names: names.into(), values: fields.into_iter().map(|(_, v)| v).collect() }))
}

// ---- log entries ---------------------------------------------------------------------------

/// Splits a log file into entries as upstream's `LogBreaker` does and trims each one.
///
/// * `\n` (or the two characters `\` `n`): one entry per line.
/// * `\n<marker>\n`: entries span lines and are separated by lines equal to `<marker>`; a
///   last entry not followed by a marker line is dropped, as upstream does.
/// * anything else: the delimiter separates entries. Upstream treats it as a regular
///   expression; here it is matched literally.
pub fn split_entries(text: &str, delimiter: &str) -> Vec<String> {
    let delim = delimiter.replace("\\n", "\n");
    let lines = || {
        // BufferedReader.readLine: \n, \r and \r\n end lines
        let mut v = Vec::new();
        let mut cur = String::new();
        let mut chars = text.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '\n' => v.push(std::mem::take(&mut cur)),
                '\r' => {
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    v.push(std::mem::take(&mut cur));
                }
                c => cur.push(c),
            }
        }
        if !cur.is_empty() {
            v.push(cur);
        }
        v
    };
    let entries: Vec<String> = if delim == "\n" {
        lines()
    } else if delim.len() >= 2
        && delim.starts_with('\n')
        && delim.ends_with('\n')
        && !delim[1..delim.len() - 1].contains('\n')
    {
        let marker = &delim[1..delim.len() - 1];
        let mut v = Vec::new();
        let mut buf = String::new();
        for line in lines() {
            if line == marker {
                v.push(std::mem::take(&mut buf));
            } else {
                buf.push_str(&line);
                buf.push('\n');
            }
        }
        v
    } else if delim.is_empty() {
        vec![text.to_string()]
    } else {
        let mut parts: Vec<String> = text.split(delim.as_str()).map(|s| s.to_string()).collect();
        if parts.first().is_some_and(|p| p.is_empty()) {
            parts.remove(0);
        }
        if parts.last().is_some_and(|p| p.is_empty()) {
            parts.pop();
        }
        parts
    };
    entries.into_iter().map(|e| java_trim(&e).to_string()).collect()
}

/// `String.trim()`: strips characters up to U+0020 at both ends.
pub fn java_trim(s: &str) -> &str {
    s.trim_matches(|c: char| c <= ' ')
}

// ---- the check -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ObserveOptions {
    /// Number of most recent events kept per key for error reports (`--replaySize`).
    pub replay_window: usize,
    /// Inputs are already sorted by timestamp: merge files instead of sorting, and report
    /// events that go back in time (`--assumeInputsAreSorted`).
    pub assume_sorted: bool,
    /// Only check these keys (`--keys`, `--keyListFile`); empty = all keys.
    pub keys: Vec<String>,
}

impl Default for ObserveOptions {
    fn default() -> Self {
        ObserveOptions { replay_window: 100, assume_sorted: false, keys: Vec::new() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// A spec assertion failed (`PAssertionFailureException`).
    SpecViolation,
    /// An event older than its predecessor (`PObserveEventOutOfOrderException`).
    OutOfOrder,
    /// The parser failed on a log entry (`PObserveLogParsingException`).
    Parser,
    /// Anything else, e.g. an event the spec's current state does not handle.
    Unknown,
}

/// The events of one key leading up to an error (`PObserveReplayEvents`).
#[derive(Debug, Clone)]
pub struct Replay {
    pub key: String,
    /// Milliseconds of the last event added.
    pub error_timestamp: i64,
    pub events: Vec<ObservedEvent>,
}

#[derive(Debug, Clone)]
pub struct ObserveError {
    pub kind: ErrorKind,
    /// `getMessage()` of the upstream exception (`null` when it has none).
    pub message: String,
    /// `toString()` of the upstream exception: the first line of its stack trace.
    pub detail: String,
    pub replay: Option<Replay>,
}

#[derive(Debug, Clone, Default)]
pub struct EventMetrics {
    pub name: String,
    pub verified: u64,
    pub total_ms: u64,
    pub max_ms: u64,
}

impl EventMetrics {
    pub fn avg_ms(&self) -> f32 {
        if self.verified > 0 {
            self.total_ms as f32 / self.verified as f32
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Metrics {
    pub partition_keys: u64,
    pub verified_keys: u64,
    pub parser_errors: u64,
    pub spec_errors: u64,
    pub out_of_order_errors: u64,
    pub unknown_errors: u64,
    pub events_read: u64,
    /// Per observed event type, in the order upstream prints them.
    pub events: Vec<EventMetrics>,
    /// Wall time of the check in milliseconds.
    pub elapsed_ms: i64,
}

impl Metrics {
    pub fn verified_events(&self) -> u64 {
        self.events.iter().map(|e| e.verified).sum()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Report {
    /// In the order upstream records them: parser errors, then errors per key.
    pub errors: Vec<ObserveError>,
    pub metrics: Metrics,
    /// Error-level messages logged while running (parser exceptions, unhandled events).
    pub log: Vec<String>,
}

impl Report {
    /// Errors counted in the final summary (all but parser errors).
    pub fn error_count(&self) -> u64 {
        self.metrics.spec_errors + self.metrics.out_of_order_errors + self.metrics.unknown_errors
    }
}

/// Monitors never reach a scheduling point; this strategy is never consulted.
struct NoScheduling;

impl Strategy for NoScheduling {
    fn next_operation(&mut self, current: u64, _ops: &[OpInfo]) -> Option<u64> {
        Some(current)
    }
    fn next_bool(&mut self, _max: i32) -> bool {
        false
    }
    fn next_int(&mut self, _max: i32) -> i32 {
        0
    }
    fn prepare_next_iteration(&mut self) -> bool {
        false
    }
    fn scheduled_steps(&self) -> usize {
        0
    }
    fn has_reached_max_steps(&self) -> bool {
        false
    }
    fn is_fair(&self) -> bool {
        false
    }
    fn description(&self) -> String {
        String::new()
    }
    fn reset(&mut self) {}
}

/// Outcome of running the spec over one key's events.
enum KeyOutcome {
    Ok,
    Error(ObserveError),
    /// The spec could not even be started: the whole run stops.
    Fatal(ObserveError),
}

impl<'p> Spec<'p> {
    fn failure_error(&self, failure: Option<Failure>, replay: Option<Replay>, log: &mut Vec<String>) -> ObserveError {
        match failure {
            Some(Failure::Assertion(m)) => ObserveError {
                kind: ErrorKind::SpecViolation,
                detail: format!("pobserve.runtime.exceptions.PAssertionFailureException: {}", m),
                message: m,
                replay,
            },
            Some(Failure::Unhandled { state, event }) => {
                log.push(format!("prt.State[{}] missing event handler for {}", state, event));
                ObserveError {
                    kind: ErrorKind::Unknown,
                    message: "null".into(),
                    detail: format!(
                        "prt.State prt.State[{}] has no handler for class {}.PEvents${}",
                        state, self.java_package, event
                    ),
                    replay,
                }
            }
            Some(Failure::Exception(ty, m)) => {
                ObserveError { kind: ErrorKind::Unknown, detail: format!("{}: {}", ty, m), message: m, replay }
            }
            Some(Failure::Other(m)) => ObserveError {
                kind: ErrorKind::Unknown,
                detail: format!("java.lang.RuntimeException: {}", m),
                message: m,
                replay,
            },
            None => ObserveError {
                kind: ErrorKind::Unknown,
                message: "null".into(),
                detail: "java.lang.RuntimeException".into(),
                replay,
            },
        }
    }

    /// `CheckPartitionedEventStream.check`.
    fn check_key(
        &self,
        key: &str,
        events: Vec<ObservedEvent>,
        opts: &ObserveOptions,
        metrics: &mut HashMap<EventId, EventMetrics>,
        clock: &mut dyn FnMut() -> f64,
        log: &mut Vec<String>,
    ) -> KeyOutcome {
        let mut strategy = NoScheduling;
        let mut rt = Runtime::new(self.prog, &self.code, &self.module, &self.config, &mut strategy, None);
        let mon = match rt.start_monitor(self.decl) {
            Ok(m) => m,
            Err(_) => return KeyOutcome::Fatal(self.failure_error(rt.failure.take(), None, log)),
        };
        let window = opts.replay_window.max(1);
        let mut replay: std::collections::VecDeque<ObservedEvent> = std::collections::VecDeque::new();
        let make_replay = |q: &std::collections::VecDeque<ObservedEvent>| Replay {
            key: key.to_string(),
            error_timestamp: q.back().map(|e| e.timestamp.to_millis()).unwrap_or(0),
            events: q.iter().cloned().collect(),
        };
        let mut prev = Timestamp::from_millis(-1);
        for ev in events {
            if replay.len() == window {
                replay.pop_front();
            }
            replay.push_back(ev.clone());
            if ev.timestamp < prev {
                let message = format!(
                    "Encountered an out of order event.\nEncountered an out of order event::\nTimestamp of last processed event: {}\nTimestamp of current event: {}\nError event details: {}",
                    prev.to_iso(),
                    ev.timestamp.to_iso(),
                    ev.to_java_string(self.prog)
                );
                return KeyOutcome::Error(ObserveError {
                    kind: ErrorKind::OutOfOrder,
                    detail: format!("pobserve.commons.exceptions.PObserveEventOutOfOrderException: {}", message),
                    message,
                    replay: Some(make_replay(&replay)),
                });
            }
            prev = ev.timestamp;
            let start = (clock() * 1000.0).floor() as i64;
            let e = Event { id: ev.event, payload: ev.payload.clone(), stamp: None, vt: None };
            rt.step_work = 0;
            if rt.monitor_event(mon, e, None).is_err() {
                return KeyOutcome::Error(self.failure_error(rt.failure.take(), Some(make_replay(&replay)), log));
            }
            let ms = ((clock() * 1000.0).floor() as i64 - start).max(0) as u64;
            if let Some(m) = metrics.get_mut(&ev.event) {
                m.verified += 1;
                m.total_ms += ms;
                m.max_ms = m.max_ms.max(ms);
            }
        }
        KeyOutcome::Ok
    }

    /// Parses `inputs` (the trimmed entries of each log file) with `parser` and checks the
    /// events against the spec (`PObserveExecutor.run`). `clock` returns seconds.
    pub fn observe(
        &self,
        inputs: &[Vec<String>],
        parser: &dyn Parser,
        opts: &ObserveOptions,
        clock: &mut dyn FnMut() -> f64,
    ) -> Report {
        let mut report = Report::default();
        let start = clock();
        let keys: std::collections::HashSet<&str> = opts.keys.iter().map(|s| s.as_str()).collect();

        // parse, keeping the events the spec observes
        let mut per_file: Vec<Vec<ObservedEvent>> = Vec::new();
        for entries in inputs {
            let mut evs = Vec::new();
            for entry in entries {
                match parser.parse(entry, self) {
                    Ok(v) => {
                        for e in v {
                            if self.observes_event(e.event) && (keys.is_empty() || keys.contains(e.key.as_str())) {
                                report.metrics.events_read += 1;
                                evs.push(e);
                            }
                        }
                    }
                    Err(_) => {
                        report.metrics.parser_errors += 1;
                        report.log.push("Parser Exception::".into());
                        report.log.push(format!("Exception occurred while parsing log line: {}", entry));
                        report.errors.push(ObserveError {
                            kind: ErrorKind::Parser,
                            message: entry.clone(),
                            detail: format!("pobserve.commons.exceptions.PObserveLogParsingException: {}", entry),
                            replay: None,
                        });
                    }
                }
            }
            per_file.push(evs);
        }
        let all: Vec<ObservedEvent> =
            if opts.assume_sorted { merge_sorted(per_file) } else { per_file.into_iter().flatten().collect() };

        // partition by key
        let mut order: Vec<String> = Vec::new();
        let mut parts: HashMap<String, Vec<ObservedEvent>> = HashMap::new();
        for e in all {
            if !parts.contains_key(&e.key) {
                order.push(e.key.clone());
            }
            parts.entry(e.key.clone()).or_default().push(e);
        }
        report.metrics.partition_keys = order.len() as u64;
        let key_values: Vec<Value> = order.iter().map(|k| Value::str(k)).collect();
        let key_order: Vec<String> = javafmt::hashmap_order(key_values.iter().map(|k| (k, ())))
            .into_iter()
            .map(|(k, _)| match k {
                Value::Str(s) => s.to_string(),
                _ => unreachable!(),
            })
            .collect();

        let mut metrics: HashMap<EventId, EventMetrics> = self
            .observes()
            .iter()
            .map(|&e| (e, EventMetrics { name: self.prog.event(e).name.clone(), ..Default::default() }))
            .collect();
        for key in key_order {
            let mut events = parts.remove(&key).unwrap_or_default();
            if !opts.assume_sorted {
                events.sort_by_key(|e| e.timestamp);
            }
            match self.check_key(&key, events, opts, &mut metrics, clock, &mut report.log) {
                KeyOutcome::Ok => report.metrics.verified_keys += 1,
                KeyOutcome::Error(err) => {
                    match err.kind {
                        ErrorKind::SpecViolation => report.metrics.spec_errors += 1,
                        ErrorKind::OutOfOrder => report.metrics.out_of_order_errors += 1,
                        _ => report.metrics.unknown_errors += 1,
                    }
                    report.errors.push(err);
                }
                KeyOutcome::Fatal(mut err) => {
                    report.metrics.unknown_errors += 1;
                    err.replay = None;
                    report.errors.push(err);
                    break;
                }
            }
        }

        // event metrics in the order of upstream's HashMap<String, EventMetrics>
        let names: Vec<Value> = self.observes().iter().map(|&e| Value::str(&self.prog.event(e).name)).collect();
        let mut seen = std::collections::HashSet::new();
        for (name, e) in javafmt::hashmap_order(names.iter().zip(self.observes().iter())) {
            if seen.insert(*e) {
                let _ = name;
                if let Some(m) = metrics.remove(e) {
                    report.metrics.events.push(m);
                }
            }
        }
        report.metrics.elapsed_ms = ((clock() - start) * 1000.0).round() as i64;
        report
    }
}

/// Merges per-file event streams already sorted by time (`MergeIntoSortedStream`).
fn merge_sorted(files: Vec<Vec<ObservedEvent>>) -> Vec<ObservedEvent> {
    let mut iters: Vec<std::iter::Peekable<std::vec::IntoIter<ObservedEvent>>> =
        files.into_iter().map(|f| f.into_iter().peekable()).collect();
    let mut out = Vec::new();
    loop {
        let mut best: Option<(usize, Timestamp)> = None;
        for (i, it) in iters.iter_mut().enumerate() {
            if let Some(e) = it.peek() {
                if best.map(|(_, t)| e.timestamp < t).unwrap_or(true) {
                    best = Some((i, e.timestamp));
                }
            }
        }
        match best {
            Some((i, _)) => out.push(iters[i].next().unwrap()),
            None => return out,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_split_like_log_breaker() {
        assert_eq!(split_entries("a\n b \r\nc", "\n"), vec!["a", "b", "c"]);
        assert_eq!(split_entries("a\nb\n--\nc\n--\nd\n", "\n--\n"), vec!["a\nb", "c"]);
        assert_eq!(split_entries("a;;b;", ";;"), vec!["a", "b;"]);
    }

    #[test]
    fn timestamps() {
        let t = Timestamp::from_millis(-1);
        assert_eq!((t.secs, t.nanos), (-1, 999_000_000));
        assert_eq!(t.to_millis(), -1);
        assert_eq!(Timestamp::from_millis(1706894854333).to_millis(), 1706894854333);
    }
}
