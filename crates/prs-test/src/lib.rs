//! In-process P monitor checks for application tests. Compile a [`Model`], connect a
//! recording's cloneable [`EventSink`] before setup, and explicitly check after cleanup.
//! JSON payloads use the same conversion as `prs observe`; no CLI process is launched.
//! Recorders do not check on drop. [`Suite`] optionally aggregates coverage immediately.

#![doc = include_str!("../README.md")]

use std::fmt;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use prs_checker::observe::{ObserveOptions, ObservedEvent, Parser, Report, Spec, Timestamp};
use prs_checker::observe_report::ObserveCoverageReport;
use prs_sema::Program;
use prs_syntax::SourceMap;
pub use serde_json::{json, Value};

/// A compiled specification. Semantic compilation happens once; the interpreter's
/// bytecode is prepared afresh when checking a recording. Every check starts fresh monitors.
#[derive(Debug)]
pub struct Model {
    sources: SourceMap,
    program: Program,
    spec: String,
    options: ObserveOptions,
}

/// Compilation, emission lifecycle, I/O, or coverage-aggregation error.
#[derive(Debug, Clone)]
pub struct SetupError(String);
impl fmt::Display for SetupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for SetupError {}

impl Model {
    /// Compile explicitly supplied source files and select one spec by its P name.
    /// No source files are read implicitly. Test checks are strict by default.
    pub fn compile(sources: &[(&str, &str)], spec: &str) -> Result<Self, SetupError> {
        let mut map = SourceMap::new();
        for (name, source) in sources {
            map.add(*name, *source);
        }
        let analysis = prs_sema::compile(&map, &Default::default()).map_err(|e| {
            SetupError(match e {
                prs_sema::CompileError::Parse { file, message, .. } => format!("{}: {message}", map.file(file).name),
                prs_sema::CompileError::NotSupported { file, line, col, keyword } => {
                    format!("{}:{line}:{col}: unsupported syntax '{keyword}'", map.file(file).name)
                }
                prs_sema::CompileError::Analysis(error) => error.messages(&map).join("\n"),
            })
        })?;
        Spec::new(&analysis.program, spec).map_err(SetupError)?;
        Ok(Self {
            sources: map,
            program: analysis.program,
            spec: spec.into(),
            options: ObserveOptions { strict: true, assume_sorted: true, coverage: true, ..Default::default() },
        })
    }

    /// Opt out only for tests where observing zero relevant events is intentional.
    pub fn strict(mut self, strict: bool) -> Self {
        self.options.strict = strict;
        self
    }

    /// Reject monitor instances left in hot states. Separate from strictness.
    pub fn check_hot(mut self, check_hot: bool) -> Self {
        self.options.check_hot = check_hot;
        self
    }

    pub fn recording(&self) -> Recording<'_> {
        Recording { model: self, sink: EventSink { state: Arc::new(Mutex::new(Buffer::default())) } }
    }

    /// Includes setup and explicit cleanup performed inside the closure. A panic in
    /// the body propagates unchanged; no secondary check runs during unwinding.
    pub fn with_recording<T>(&self, body: impl FnOnce(&EventSink) -> T) -> Result<(T, CheckReport), CheckError> {
        let recording = self.recording();
        let value = body(&recording.sink);
        let report = recording.check()?;
        Ok((value, report))
    }
}

#[derive(Debug, Clone)]
struct Event {
    key: String,
    name: String,
    payload: Value,
    timestamp_ms: i64,
}
#[derive(Debug, Default)]
struct Buffer {
    events: Vec<Event>,
    closed: bool,
    errors: Vec<String>,
}

/// Thread-safe handle suitable for application callbacks. Clones share one ordered
/// recording. The mutex establishes event order; publish after the operation commits.
/// Unrelated event names are ignored at check time, like `prs observe`. An invalid
/// payload for a relevant event remains recorded and fails the check.
#[derive(Debug, Clone)]
pub struct EventSink {
    state: Arc<Mutex<Buffer>>,
}

impl EventSink {
    /// Timestamp with epoch milliseconds while holding the recording's ordering lock.
    pub fn emit(&self, key: impl Into<String>, event: impl Into<String>, payload: Value) -> Result<(), SetupError> {
        self.push(key.into(), event.into(), payload, None)
    }

    /// Explicit epoch milliseconds for deterministic tests or application event times.
    /// Events are checked in emission order; decreasing timestamps for a key fail.
    pub fn emit_at(
        &self,
        key: impl Into<String>,
        event: impl Into<String>,
        payload: Value,
        timestamp_ms: i64,
    ) -> Result<(), SetupError> {
        self.push(key.into(), event.into(), payload, Some(timestamp_ms))
    }

    fn push(&self, key: String, name: String, payload: Value, timestamp: Option<i64>) -> Result<(), SetupError> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.closed {
            return Err(SetupError("recording is closed; emit before check() and join producer threads first".into()));
        }
        let timestamp_ms = match timestamp {
            Some(timestamp) => timestamp,
            None => {
                match SystemTime::now().duration_since(UNIX_EPOCH).ok().and_then(|d| i64::try_from(d.as_millis()).ok())
                {
                    Some(timestamp) => timestamp,
                    None => {
                        let error = "could not represent event time as epoch milliseconds".to_string();
                        state.errors.push(error.clone());
                        return Err(SetupError(error));
                    }
                }
            }
        };
        state.events.push(Event { key, name, payload, timestamp_ms });
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).events.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A fresh test recording. Check consumes it and closes all cloned sink handles.
/// Finish fixture cleanup and join producers before checking. No assertion runs in Drop.
#[derive(Debug)]
pub struct Recording<'m> {
    model: &'m Model,
    sink: EventSink,
}
impl Recording<'_> {
    pub fn sink(&self) -> EventSink {
        self.sink.clone()
    }
    pub fn emit(&self, key: impl Into<String>, event: impl Into<String>, payload: Value) -> Result<(), SetupError> {
        self.sink.emit(key, event, payload)
    }
    pub fn emit_at(
        &self,
        key: impl Into<String>,
        event: impl Into<String>,
        payload: Value,
        timestamp_ms: i64,
    ) -> Result<(), SetupError> {
        self.sink.emit_at(key, event, payload, timestamp_ms)
    }
    pub fn len(&self) -> usize {
        self.sink.len()
    }
    pub fn is_empty(&self) -> bool {
        self.sink.is_empty()
    }

    pub fn check(self) -> Result<CheckReport, CheckError> {
        let (events, emission_errors) = {
            let mut buffer = self.sink.state.lock().unwrap_or_else(|e| e.into_inner());
            buffer.closed = true;
            (std::mem::take(&mut buffer.events), std::mem::take(&mut buffer.errors))
        };
        // Model::compile already validated the selected specification.
        let spec = Spec::new(&self.model.program, &self.model.spec).expect("validated model specification");
        let parser = Events(&events);
        let entries = (0..events.len()).map(|index| index.to_string()).collect();
        let started = Instant::now();
        let mut report =
            spec.observe(&[entries], &parser, &self.model.options, &mut || started.elapsed().as_secs_f64());
        for error in &emission_errors {
            report.errors.push(prs_checker::observe::ObserveError {
                kind: prs_checker::observe::ErrorKind::Parser,
                message: error.clone(),
                detail: error.clone(),
                replay: None,
                cause: Some("event emission failed".into()),
            });
            report.metrics.parser_errors += 1;
        }
        let mut diagnostics = String::new();
        for error in &report.errors {
            diagnostics.push_str(&format!("{:?}: {}\n", error.kind, error.message));
            if let Some(cause) = &error.cause {
                diagnostics.push_str(&format!("  {cause}\n"));
            }
            if let Some(replay) = &error.replay {
                diagnostics.push_str(&format!("Replay for key {}:\n", replay.key));
                for event in &replay.events {
                    diagnostics.push_str(&format!("  {}\n", event.to_java_string(&spec)));
                }
            }
        }
        for error in &emission_errors {
            diagnostics.push_str(&format!("Emission error: {error}\n"));
        }
        let failed = !report.errors.is_empty() || !emission_errors.is_empty();
        let checked = CheckReport { spec: self.model.spec.clone(), report };
        if failed {
            let trace = events
                .iter()
                .map(|e| json!({"key":e.key,"event":e.name,"payload":e.payload,"timestamp":e.timestamp_ms}))
                .collect::<Vec<_>>();
            diagnostics.push_str(&format!(
                "Recorded events:\n{}\nSources: {}\n",
                serde_json::to_string_pretty(&trace).unwrap(),
                self.model.sources.files().iter().map(|f| f.name.as_str()).collect::<Vec<_>>().join(", ")
            ));
            Err(CheckError { report: Box::new(checked), diagnostics, events: trace })
        } else {
            Ok(checked)
        }
    }
}

struct Events<'a>(&'a [Event]);
impl Parser for Events<'_> {
    fn name(&self) -> &str {
        "prs-test events"
    }
    fn parse(&self, entry: &str, spec: &Spec) -> Result<Vec<ObservedEvent>, String> {
        let index: usize = entry.parse().map_err(|e| format!("invalid recording index: {e}"))?;
        let event = self.0.get(index).ok_or("missing recorded event")?;
        // Shared event streams may include names absent from this P model or events
        // declared for another monitor. Filter before conversion, like `prs observe`.
        let Some(id) = spec.event_id(&event.name) else { return Ok(Vec::new()) };
        if !spec.observes_event(id) {
            return Ok(Vec::new());
        }
        if matches!(spec.payload_type(id), prs_sema::PType::Null) && !event.payload.is_null() {
            return Err(format!("unit event '{}' requires a null JSON payload", event.name));
        }
        // Validate null scalar payloads too; the upstream event constructor allows null
        // before type conversion, while a test adapter must report broken mappings.
        let payload = prs_compat::observe::value_from_json(spec.prog, &event.payload, spec.payload_type(id))?;
        let line = json!({"event":event.name,"key":event.key,"payload":event.payload,"timestamp":event.timestamp_ms})
            .to_string();
        let converted = spec.event(
            event.key.clone(),
            Timestamp::from_millis(event.timestamp_ms),
            &event.name,
            payload,
            Some(&line),
        )?;
        Ok(vec![converted])
    }
}

/// A checked observation, including failures when accessed through [`CheckError::report`].
#[derive(Debug)]
pub struct CheckReport {
    spec: String,
    report: Report,
}
impl CheckReport {
    pub fn passed(&self) -> bool {
        self.report.errors.is_empty()
    }
    pub fn exit_status(&self) -> i32 {
        self.report.exit_status()
    }
    pub fn events_read(&self) -> u64 {
        self.report.metrics.events_read
    }
    pub fn verified_events(&self) -> u64 {
        self.report.metrics.verified_events()
    }
    pub fn coverage(&self) -> Result<ObserveCoverageReport, SetupError> {
        ObserveCoverageReport::from_observation(&self.spec, &self.report).map_err(SetupError)
    }
}

/// Monitor/parser/strict failure. The failed prefix's coverage remains available.
#[derive(Debug)]
pub struct CheckError {
    report: Box<CheckReport>,
    diagnostics: String,
    events: Vec<Value>,
}
impl CheckError {
    pub fn report(&self) -> &CheckReport {
        &self.report
    }
    /// Owned JSON event history, suitable for persisting a failed generated case.
    pub fn events(&self) -> &[Value] {
        &self.events
    }
}
impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.diagnostics)
    }
}
impl std::error::Error for CheckError {}

/// Optional immediate coverage union. Retains coverage sets and distinct source
/// metadata, never full successful event traces. Use a fresh suite for each test run.
#[derive(Debug, Default)]
pub struct Suite {
    coverage: ObserveCoverageReport,
}
impl Suite {
    pub fn new() -> Self {
        Self::default()
    }
    /// Add a successful or failed observation's report. Incompatible sources reject atomically.
    pub fn add(&mut self, report: &CheckReport) -> Result<(), SetupError> {
        self.coverage.merge(&report.coverage()?).map_err(SetupError)
    }
    pub fn render_text(&self) -> String {
        self.coverage.render_text()
    }
    pub fn to_json(&self) -> Value {
        self.coverage.to_json()
    }
    pub fn write_json(&self, path: impl AsRef<Path>) -> Result<(), SetupError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| SetupError(e.to_string()))?;
        }
        std::fs::write(
            path,
            serde_json::to_string_pretty(&self.to_json()).map_err(|e| SetupError(e.to_string()))? + "\n",
        )
        .map_err(|e| SetupError(e.to_string()))
    }
}
