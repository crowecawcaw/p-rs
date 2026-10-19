//! Validated, portable aggregation of observed monitor coverage. No model execution or I/O.
use std::collections::{BTreeMap, BTreeSet};

use serde_json::{json, Value};

use crate::observe_coverage::{HandlerCoverage, ObserveCoverage, TransitionCoverage};

/// A suite union and its distinct source observation metadata. Coverage does not imply that
/// verification passed: errors, exit status and completion fields survive aggregation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ObserveCoverageReport {
    pub specs: BTreeMap<String, ObserveCoverage>,
    pub observations: Vec<Value>,
}

impl ObserveCoverage {
    /// Decode and validate a coverage object. Counters must match its unique hit sets.
    pub fn from_json(value: &Value) -> Result<Self, String> {
        if value["scope"] != "observed_monitor" || value["schema_version"] != 1 {
            return Err(
                "unsupported observed coverage scope or schema_version; regenerate with observe --coverage".into()
            );
        }
        let fingerprint = string(&value["source_fingerprint"], "source_fingerprint")?;
        if !fingerprint.starts_with("sha256:")
            || fingerprint.len() != 71
            || !fingerprint[7..].bytes().all(|c| c.is_ascii_hexdigit())
        {
            return Err("source_fingerprint must be a SHA-256 identity; regenerate with observe --coverage".into());
        }
        let states = |key: &str| -> Result<BTreeSet<String>, String> {
            array(&value["states"][key], key)?.iter().map(|v| string(v, key)).collect()
        };
        let handlers = |key: &str| -> Result<BTreeSet<HandlerCoverage>, String> {
            array(&value["handlers"][key], key)?
                .iter()
                .map(|v| {
                    Ok(HandlerCoverage {
                        state: string(&v["state"], "handler.state")?,
                        event: string(&v["event"], "handler.event")?,
                    })
                })
                .collect()
        };
        let result = Self {
            source_fingerprint: Some(fingerprint),
            declared_states: states("declared")?,
            visited_states: states("visited")?,
            declared_handlers: handlers("declared")?,
            visited_handlers: handlers("visited")?,
            transitions: array(&value["transitions"], "transitions")?
                .iter()
                .map(|v| {
                    Ok(TransitionCoverage {
                        source: string(&v["source"], "transition.source")?,
                        target: string(&v["target"], "transition.target")?,
                    })
                })
                .collect::<Result<_, String>>()?,
        };
        result.validate()?;
        for (kind, declared, visited) in [
            ("states", result.declared_states.len(), result.visited_states.len()),
            ("handlers", result.declared_handlers.len(), result.visited_handlers.len()),
        ] {
            if value[kind]["total"].as_u64() != Some(declared as u64)
                || value[kind]["covered"].as_u64() != Some(visited as u64)
            {
                return Err(format!("{kind} counters do not match unique declarations/hits"));
            }
        }
        Ok(result)
    }

    fn validate(&self) -> Result<(), String> {
        let fingerprint = self
            .source_fingerprint
            .as_deref()
            .ok_or("missing source fingerprint; regenerate with observe --coverage")?;
        if !fingerprint.starts_with("sha256:")
            || fingerprint.len() != 71
            || !fingerprint[7..].bytes().all(|c| c.is_ascii_hexdigit())
        {
            return Err("invalid source fingerprint; regenerate with observe --coverage".into());
        }
        if !self.visited_states.is_subset(&self.declared_states)
            || !self.visited_handlers.is_subset(&self.declared_handlers)
        {
            return Err("coverage hits must be a subset of declarations".into());
        }
        if self.declared_handlers.iter().any(|h| !self.declared_states.contains(&h.state))
            || self
                .transitions
                .iter()
                .any(|t| !self.declared_states.contains(&t.source) || !self.declared_states.contains(&t.target))
        {
            return Err("handler or transition refers to an undeclared state".into());
        }
        Ok(())
    }

    /// Validated same-source union. On failure the receiver is unchanged.
    pub fn try_merge(&mut self, other: &Self) -> Result<(), String> {
        self.validate()?;
        other.validate()?;
        if self.source_fingerprint.is_none() || self.source_fingerprint != other.source_fingerprint {
            return Err("cannot merge coverage with missing or different source fingerprints".into());
        }
        if self.declared_states != other.declared_states || self.declared_handlers != other.declared_handlers {
            return Err("cannot merge coverage with incompatible declarations".into());
        }
        self.merge(other);
        Ok(())
    }
}

impl ObserveCoverageReport {
    /// Import an in-process observation. Completion is unknown unless supplied by a
    /// streaming supervisor; error details and event metrics are retained.
    pub fn from_observation(spec_name: &str, report: &crate::observe::Report) -> Result<Self, String> {
        let coverage = report.coverage.as_ref().ok_or("observation has no coverage; enable ObserveOptions.coverage")?;
        Self::from_json(&json!({
            "exit_code": report.exit_status(),
            "specs": [{"spec": spec_name, "coverage": coverage.to_json(),
                "events_read": report.metrics.events_read,
                "partition_keys": report.metrics.partition_keys,
                "verified_keys": report.metrics.verified_keys,
                "errors": report.errors.iter().map(|e| json!({"kind": match e.kind {
                    crate::observe::ErrorKind::SpecViolation => "spec_violation",
                    crate::observe::ErrorKind::Parser => "parser",
                    crate::observe::ErrorKind::OutOfOrder => "out_of_order",
                    _ => "unknown",
                },
                    "message": e.message, "detail": e.detail, "cause": e.cause})).collect::<Vec<_>>() }]
        }))
    }

    /// Accept a full `observe --coverage --json-report` result, a suite result, or an
    /// array of either. Missing coverage is an error, including on failed observations.
    pub fn from_json(value: &Value) -> Result<Self, String> {
        if let Some(items) = value.as_array() {
            let mut result = Self::default();
            for item in items {
                result.merge(&Self::from_json(item)?)?;
            }
            return Ok(result);
        }
        let mut result = Self::default();
        let suite = value["scope"] == "observed_monitor_suite";
        if suite && value["schema_version"] != 1 {
            return Err("unsupported suite schema_version".into());
        }
        if !suite && value.get("scope").is_some() {
            return Err("unsupported report scope".into());
        }
        let specs = array(&value["specs"], "specs")?;
        if !suite && specs.is_empty() {
            return Err("observation has no specifications".into());
        }
        for entry in specs {
            let name = string(&entry["spec"], "spec")?;
            let coverage = ObserveCoverage::from_json(&entry["coverage"]).map_err(|e| format!("{name}: {e}"))?;
            if result.specs.insert(name.clone(), coverage).is_some() {
                return Err(format!("duplicate spec: {name}"));
            }
        }
        if suite {
            for observation in array(&value["observations"], "observations")? {
                validate_observation(observation)?;
                if !result.observations.contains(observation) {
                    result.observations.push(observation.clone());
                }
            }
        } else {
            validate_observation(value)?;
            let mut metadata = value.clone();
            for spec in metadata["specs"].as_array_mut().unwrap() {
                spec.as_object_mut().unwrap().remove("coverage");
            }
            result.observations.push(metadata);
        }
        Ok(result)
    }

    pub fn from_json_str(text: &str) -> Result<Self, String> {
        Self::from_json(&serde_json::from_str(text).map_err(|e| e.to_string())?)
    }

    /// Atomic, idempotent union, retaining distinct source metadata without summing hits.
    pub fn merge(&mut self, other: &Self) -> Result<(), String> {
        let mut merged = self.clone();
        for (name, coverage) in &other.specs {
            if let Some(existing) = merged.specs.get_mut(name) {
                existing.try_merge(coverage).map_err(|e| format!("{name}: {e}"))?;
            } else {
                coverage.validate()?;
                merged.specs.insert(name.clone(), coverage.clone());
            }
        }
        for observation in &other.observations {
            if !merged.observations.contains(observation) {
                merged.observations.push(observation.clone());
            }
        }
        *self = merged;
        Ok(())
    }

    pub fn to_json(&self) -> Value {
        json!({"schema_version": 1, "scope": "observed_monitor_suite",
            "specs": self.specs.iter().map(|(spec, coverage)| json!({"spec":spec,"coverage":coverage.to_json()})).collect::<Vec<_>>(),
            "observations": self.observations})
    }

    pub fn render_text(&self) -> String {
        let failed = self.observations.iter().filter(|v| v["exit_code"] != 0).count();
        let incomplete = self.observations.iter().filter(|v| v["input_complete"] == false).count();
        let unknown = self.observations.iter().filter(|v| !v["input_complete"].is_boolean()).count();
        let mut out = format!("Observed monitor suite coverage: {} specifications\nDistinct observation metadata records: {}; failed: {failed}; incomplete: {incomplete}; completion unknown: {unknown}\nCoverage union does not imply successful verification.\n", self.specs.len(), self.observations.len());
        for (spec, coverage) in &self.specs {
            out.push_str(&format!("\n{spec}\n{}", coverage.render_text()));
        }
        out
    }
}

fn validate_observation(value: &Value) -> Result<(), String> {
    if value["exit_code"].as_i64().is_none() {
        return Err("observation exit_code must be an integer".into());
    }
    if let Some(complete) = value.get("input_complete") {
        if !complete.is_boolean() {
            return Err("input_complete must be a boolean".into());
        }
    }
    for spec in array(&value["specs"], "specs")? {
        string(&spec["spec"], "spec")?;
        array(&spec["errors"], "errors")?;
    }
    Ok(())
}
fn string(value: &Value, name: &str) -> Result<String, String> {
    value
        .as_str()
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| format!("{name} must be a nonempty string"))
}
fn array<'a>(value: &'a Value, name: &str) -> Result<&'a Vec<Value>, String> {
    value.as_array().ok_or_else(|| format!("{name} must be an array"))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn observation(visited: &[&str]) -> Value {
        let mut coverage = ObserveCoverage {
            source_fingerprint: Some(format!("sha256:{}", "a".repeat(64))),
            ..ObserveCoverage::default()
        };
        coverage.declared_states.extend(["Spec.A".into(), "Spec.B".into()]);
        coverage.visited_states.extend(visited.iter().map(|s| s.to_string()));
        json!({"exit_code":0,"specs":[{"spec":"Spec","coverage":coverage.to_json(),"errors":[]}]})
    }
    #[test]
    fn roundtrip_and_idempotent_union_preserve_failure_metadata() {
        let a = observation(&["Spec.A"]);
        let mut b = observation(&["Spec.B"]);
        b["exit_code"] = json!(1);
        b["input_complete"] = json!(false);
        b["stop_reason"] = json!("error");
        b["specs"][0]["errors"] = json!([{"kind":"spec_violation","message":"oversold"}]);
        let mut suite = ObserveCoverageReport::from_json(&json!([a, b])).unwrap();
        assert_eq!(suite.specs["Spec"].visited_states.len(), 2);
        let roundtrip = ObserveCoverageReport::from_json(&suite.to_json()).unwrap();
        suite.merge(&roundtrip).unwrap();
        assert_eq!(suite, roundtrip);
        assert!(suite.render_text().contains("failed: 1; incomplete: 1"));
        assert_eq!(suite.observations[1]["stop_reason"], "error");
    }
    #[test]
    fn incompatible_merge_is_atomic_even_if_an_earlier_spec_is_new() {
        let mut suite = ObserveCoverageReport::from_json(&observation(&[])).unwrap();
        let before = suite.clone();
        let mut other = suite.clone();
        other.specs.insert("AAA".into(), other.specs["Spec"].clone());
        other.specs.get_mut("Spec").unwrap().source_fingerprint = Some(format!("sha256:{}", "b".repeat(64)));
        assert!(suite.merge(&other).unwrap_err().contains("fingerprint"));
        assert_eq!(suite, before);
    }
    #[test]
    fn rejects_malformed_missing_and_impossible_coverage() {
        for mutation in ["missing", "counter", "outside", "declaration", "version", "fingerprint"] {
            let mut value = observation(&[]);
            let c = &mut value["specs"][0]["coverage"];
            match mutation {
                "missing" => *c = Value::Null,
                "counter" => c["states"]["covered"] = json!(1),
                "outside" => {
                    c["states"]["visited"] = json!(["Other.A"]);
                    c["states"]["covered"] = json!(1);
                }
                "declaration" => {
                    c["handlers"] =
                        json!({"total":1,"covered":0,"declared":[{"state":"Other.A","event":"E"}],"visited":[]})
                }
                "version" => c["schema_version"] = json!(999),
                "fingerprint" => c["source_fingerprint"] = Value::Null,
                _ => unreachable!(),
            }
            assert!(ObserveCoverageReport::from_json(&value).is_err(), "{mutation}");
        }
    }
    #[test]
    fn mismatching_declarations_rejected_despite_matching_fingerprint() {
        let first = ObserveCoverageReport::from_json(&observation(&[])).unwrap();
        let mut second = first.clone();
        second.specs.get_mut("Spec").unwrap().declared_states.insert("Spec.C".into());
        assert!(second.merge(&first).unwrap_err().contains("incompatible declarations"));
    }
    #[test]
    fn in_process_errors_use_observe_exit_status_and_preserve_details() {
        use crate::observe::{ErrorKind, ObserveError, Report};
        let coverage = ObserveCoverageReport::from_json(&observation(&[])).unwrap().specs["Spec"].clone();
        let mut report = Report { coverage: Some(coverage), ..Report::default() };
        report.errors.push(ObserveError {
            kind: ErrorKind::SpecViolation,
            message: "oversold".into(),
            detail: "assertion failed".into(),
            replay: None,
            cause: None,
        });
        let suite = ObserveCoverageReport::from_observation("Spec", &report).unwrap();
        assert_eq!(suite.observations[0]["exit_code"], 2);
        assert_eq!(suite.observations[0]["specs"][0]["errors"][0]["kind"], "spec_violation");
        report.errors[0].kind = ErrorKind::Parser;
        assert_eq!(ObserveCoverageReport::from_observation("Spec", &report).unwrap().observations[0]["exit_code"], 4);
        report.errors[0].kind = ErrorKind::Unknown;
        assert_eq!(ObserveCoverageReport::from_observation("Spec", &report).unwrap().observations[0]["exit_code"], 3);
    }

    #[test]
    fn empty_array_is_an_empty_aggregation() {
        let empty = ObserveCoverageReport::from_json(&json!([])).unwrap();
        assert!(empty.specs.is_empty());
        assert_eq!(ObserveCoverageReport::from_json(&empty.to_json()).unwrap(), empty);
    }
}
