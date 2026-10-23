//! Coverage of `snappea observe` runs merged across tests and processes, without the P
//! sources: per spec, the union of the code the runs covered, plus what each run reported.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::checker::code_coverage::{uncovered_lines, CodeCoverage};

/// The merged coverage of observations. Coverage says what the events exercised, not that
/// the checks passed: each observation's exit code and errors are kept with it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ObserveCoverageReport {
    pub specs: BTreeMap<String, CodeCoverage>,
    /// Distinct observation results, without their coverage.
    pub observations: Vec<Value>,
}

impl ObserveCoverageReport {
    /// The coverage of an in-process observation of spec `spec_name`.
    pub fn from_observation(spec_name: &str, report: &crate::checker::observe::Report) -> Result<Self, String> {
        Self::from_json(&json!({
            "exit_code": crate::checker::observe::Report::snappea_exit_code(report.exit_status()),
            "specs": [{"spec": spec_name, "coverage": report.coverage.to_json(true),
                "events_read": report.metrics.events_read,
                "partition_keys": report.metrics.partition_keys,
                "verified_keys": report.metrics.verified_keys,
                "errors": report.errors.iter().map(|e| json!({"kind": e.kind.name(),
                    "message": e.message, "detail": e.detail, "cause": e.cause})).collect::<Vec<_>>() }]
        }))
    }

    /// Reads a `snappea observe` JSON result, a merged report, or an array of them.
    pub fn from_json(value: &Value) -> Result<Self, String> {
        if let Some(items) = value.as_array() {
            let mut result = Self::default();
            for item in items {
                result.merge(&Self::from_json(item)?)?;
            }
            return Ok(result);
        }
        let mut result = Self::default();
        let merged = value["scope"] == "observed_monitor_suite";
        if !merged && value.get("scope").is_some() {
            return Err("not a snappea observe result or merged coverage".into());
        }
        let specs = array(&value["specs"], "specs")?;
        if !merged && specs.is_empty() {
            return Err("observation has no specs".into());
        }
        for entry in specs {
            let name = string(&entry["spec"], "spec")?;
            let coverage = CodeCoverage::from_json(&entry["coverage"]).map_err(|e| format!("{name}: {e}"))?;
            if result.specs.insert(name.clone(), coverage).is_some() {
                return Err(format!("duplicate spec: {name}"));
            }
        }
        if merged {
            for observation in array(&value["observations"], "observations")? {
                validate_observation(observation)?;
                if !result.observations.contains(observation) {
                    result.observations.push(observation.clone());
                }
            }
        } else {
            validate_observation(value)?;
            let mut metadata = value.clone();
            for spec in metadata["specs"].as_array_mut().into_iter().flatten() {
                if let Some(o) = spec.as_object_mut() {
                    o.remove("coverage");
                }
            }
            result.observations.push(metadata);
        }
        Ok(result)
    }

    pub fn from_json_str(text: &str) -> Result<Self, String> {
        Self::from_json(&serde_json::from_str(text).map_err(|e| e.to_string())?)
    }

    /// Adds `other`; on failure (coverage of different code) the receiver is unchanged.
    pub fn merge(&mut self, other: &Self) -> Result<(), String> {
        let mut merged = self.clone();
        for (name, coverage) in &other.specs {
            match merged.specs.get_mut(name) {
                Some(existing) => existing.merge(coverage).map_err(|e| format!("{name}: {e}"))?,
                None => {
                    merged.specs.insert(name.clone(), coverage.clone());
                }
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
            "specs": self.specs.iter().map(|(spec, c)| json!({"spec": spec, "coverage": c.to_json(true)})).collect::<Vec<_>>(),
            "observations": self.observations})
    }

    /// The coverage of each spec, and what no observation ran.
    pub fn render_text(&self) -> String {
        // identical results are kept once, so they are not counted
        let failed = self.observations.iter().filter(|v| v["exit_code"] != 0).count();
        let incomplete = self.observations.iter().filter(|v| v["input_complete"] == false).count();
        let mut notes = Vec::new();
        if failed > 0 {
            notes.push("some observations failed");
        }
        if incomplete > 0 {
            notes.push("some were incomplete");
        }
        let notes = if notes.is_empty() { String::new() } else { format!(" ({})", notes.join("; ")) };
        let mut out = format!("Coverage{}:\n", notes);
        let mut uncovered = Vec::new();
        for (spec, c) in &self.specs {
            out.push_str(&format!("  {}: {}\n", spec, c.summary()));
            uncovered.extend(c.uncovered());
        }
        uncovered.sort();
        uncovered.dedup();
        for l in uncovered_lines("Not covered by any observation:", &uncovered, " (--json lists all)") {
            out.push_str(&l);
            out.push('\n');
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
    use crate::checker::code_coverage::{Point, PointKind};

    fn observation(hit: &[u32], exit_code: i32) -> Value {
        let point = |line| Point {
            kind: PointKind::Handler,
            file: "spec.p".into(),
            line,
            context: format!("Spec.S, on e{line}"),
            missed: "never handled".into(),
            parent: None,
        };
        let coverage = CodeCoverage {
            fingerprint: Some("sha256:a".into()),
            points: vec![point(1), point(2)],
            hit: hit.iter().copied().collect(),
        };
        json!({"exit_code": exit_code, "specs": [{"spec": "Spec", "coverage": coverage.to_json(true), "errors": []}]})
    }

    #[test]
    fn merges_by_union_and_keeps_each_observation() {
        let mut b = observation(&[1], 1);
        b["input_complete"] = json!(false);
        let mut suite = ObserveCoverageReport::from_json(&json!([observation(&[0], 0), b])).unwrap();
        assert_eq!(suite.specs["Spec"].hit.len(), 2);
        assert_eq!(suite.observations.len(), 2);
        let roundtrip = ObserveCoverageReport::from_json(&suite.to_json()).unwrap();
        suite.merge(&roundtrip).unwrap();
        assert_eq!(suite, roundtrip);
        assert!(suite.render_text().starts_with("Coverage (some observations failed; some were incomplete):\n"));
    }

    #[test]
    fn coverage_of_other_code_does_not_merge() {
        let mut suite = ObserveCoverageReport::from_json(&observation(&[], 0)).unwrap();
        let before = suite.clone();
        let mut other = observation(&[], 0);
        other["specs"][0]["coverage"]["source_fingerprint"] = json!("sha256:b");
        let other = ObserveCoverageReport::from_json(&other).unwrap();
        assert!(suite.merge(&other).unwrap_err().contains("different P sources"));
        assert_eq!(suite, before);
    }

    #[test]
    fn rejects_malformed_results() {
        for bad in
            [json!({}), json!({"exit_code": 0, "specs": [{"spec": "Spec", "errors": []}]}), json!({"scope": "x"})]
        {
            assert!(ObserveCoverageReport::from_json(&bad).is_err(), "{bad}");
        }
        let mut hit_outside = observation(&[], 0);
        hit_outside["specs"][0]["coverage"]["hit"] = json!([7]);
        assert!(ObserveCoverageReport::from_json(&hit_outside).is_err());
    }
}
