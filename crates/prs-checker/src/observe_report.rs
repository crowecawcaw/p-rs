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
        let mut out = format!("Observed monitor suite coverage: {} specifications\nDistinct source observations: {}; failed: {failed}; incomplete: {incomplete}; completion unknown: {unknown}\nCoverage union does not imply successful verification.\n", self.specs.len(), self.observations.len());
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
