//! Coverage of monitor executions over observed logs. This measures states entered and
//! declared state/event handlers exercised, not P model paths or scheduler coverage.
//! Transition hits are reported without a denominator: action bodies can jump dynamically.

use std::collections::BTreeSet;

use prs_sema::{HandlerAction, MachineId, Program, StateId};

use crate::coverage::{CoverageInfo, CoverageTables};

/// Stable state/event identity, suitable for union across processes or test cases.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HandlerCoverage {
    pub state: String,
    pub event: String,
}

/// A transition actually taken, including transitions from action bodies.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TransitionCoverage {
    pub source: String,
    pub target: String,
}

/// Mergeable coverage for the selected monitor declarations. Union only reports built
/// from the same version of a specification; identities do not fingerprint source code.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ObserveCoverage {
    pub declared_states: BTreeSet<String>,
    pub visited_states: BTreeSet<String>,
    /// Explicit do, goto and ignore handlers. Deferred events are not Java handlers.
    pub declared_handlers: BTreeSet<HandlerCoverage>,
    /// Includes the handler attempted when an assertion fails.
    pub visited_handlers: BTreeSet<HandlerCoverage>,
    pub transitions: BTreeSet<TransitionCoverage>,
}

fn state_name(prog: &Program, state: StateId) -> String {
    let s = prog.state(state);
    format!("{}.{}", prog.machine(s.machine).name, s.name)
}

impl ObserveCoverage {
    pub fn new(prog: &Program, spec: MachineId) -> Self {
        let mut result = Self::default();
        for &state in &prog.machine(spec).states {
            let name = state_name(prog, state);
            result.declared_states.insert(name.clone());
            for handler in &prog.state(state).handlers {
                if !matches!(handler.action, HandlerAction::Defer) {
                    result
                        .declared_handlers
                        .insert(HandlerCoverage { state: name.clone(), event: prog.event(handler.event).name.clone() });
                }
            }
        }
        result
    }

    /// Union hit sets and declarations, without summing duplicate hits.
    pub fn merge(&mut self, other: &Self) {
        self.declared_states.extend(other.declared_states.iter().cloned());
        self.visited_states.extend(other.visited_states.iter().cloned());
        self.declared_handlers.extend(other.declared_handlers.iter().cloned());
        self.visited_handlers.extend(other.visited_handlers.iter().cloned());
        self.transitions.extend(other.transitions.iter().cloned());
    }

    pub(crate) fn record(&mut self, prog: &Program, tables: &CoverageTables, info: &CoverageInfo) {
        self.visited_states.extend(info.monitor_states.iter().map(|&s| state_name(prog, s)));
        for h in &self.declared_handlers {
            let hit = prog.states.iter().enumerate().any(|(i, s)| {
                state_name(prog, StateId(i as u32)) == h.state
                    && s.handlers.iter().any(|handler| {
                        prog.event(handler.event).name == h.event
                            && info.received.contains(&(tables.plain(StateId(i as u32)), handler.event.0))
                    })
            });
            if hit {
                self.visited_handlers.insert(h.clone());
            }
        }
        self.transitions.extend(
            info.monitor_transitions
                .iter()
                .map(|&(a, b)| TransitionCoverage { source: state_name(prog, a), target: state_name(prog, b) }),
        );
    }

    /// Stable machine-readable representation shared with `observe --json-report`.
    pub fn to_json(&self) -> serde_json::Value {
        let handlers = |items: &BTreeSet<HandlerCoverage>| {
            items.iter().map(|h| serde_json::json!({"state": h.state, "event": h.event})).collect::<Vec<_>>()
        };
        serde_json::json!({
            "scope": "observed_monitor",
            "states": {"total": self.declared_states.len(), "covered": self.visited_states.len(), "declared": self.declared_states, "visited": self.visited_states},
            "handlers": {"total": self.declared_handlers.len(), "covered": self.visited_handlers.len(), "declared": handlers(&self.declared_handlers), "visited": handlers(&self.visited_handlers)},
            "transitions": self.transitions.iter().map(|t| serde_json::json!({"source": t.source, "target": t.target})).collect::<Vec<_>>()
        })
    }

    /// Human-readable summary with every missing state and handler listed.
    pub fn render_text(&self) -> String {
        let mut out = format!(
            "Observed monitor coverage (not full model or schedule coverage)\nStates: {}/{}\nHandlers: {}/{}\nTransitions observed: {}\n",
            self.visited_states.len(), self.declared_states.len(), self.visited_handlers.len(),
            self.declared_handlers.len(), self.transitions.len()
        );
        for state in self.declared_states.difference(&self.visited_states) {
            out.push_str(&format!("  Unvisited state: {state}\n"));
        }
        for h in self.declared_handlers.difference(&self.visited_handlers) {
            out.push_str(&format!("  Uncovered handler: {} / {}\n", h.state, h.event));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suite_union_keeps_missing_declarations_and_deduplicates_hits() {
        let mut first = ObserveCoverage::default();
        first.declared_states.extend(["Spec.A".into(), "Spec.B".into(), "Spec.Unseen".into()]);
        first.visited_states.insert("Spec.A".into());
        let mut second = first.clone();
        second.visited_states.insert("Spec.B".into());
        first.merge(&second);
        first.merge(&second);
        assert_eq!(first.to_json()["states"]["covered"], 2);
        assert_eq!(first.to_json()["states"]["total"], 3);
        assert!(first.render_text().contains("Unvisited state: Spec.Unseen"));
    }
}
