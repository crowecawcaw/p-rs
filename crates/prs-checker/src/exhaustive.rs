//! Exhaustive search (PEx mode): explores every behaviour of a test instead of sampling.
//!
//! The search is a depth-first search over the choice points the runtime already
//! controls: which operation runs at a scheduling point, and every `$`/`choose` value.
//! It is stateless in the sense that each schedule re-runs the test from the start,
//! replaying the choices of the path to the branch being explored, then takes the next
//! untried alternative there (so no runtime state has to be cloned).
//!
//! Visited-state caching cuts the search down: at every scheduling point the program
//! state is hashed ([`crate::statehash`]) and a schedule stops when it reaches a state
//! that was already reached at the same or a smaller depth, since everything after it is
//! (or is being) explored from the earlier visit. Remembering the smallest depth keeps the
//! search complete under the depth bound.
//!
//! When the search runs out of alternatives with no bound hit, every behaviour of the
//! test within the step bound was explored: "no bug found" then means there is none (up
//! to 64-bit hash collisions). Liveness is checked for schedules that end (a hot monitor
//! at the end of a terminating schedule); the temperature-based check needs fair
//! schedules and is off.

use std::collections::HashMap;

use crate::strategy::{OpInfo, SharedRandom, Strategy};

/// Statistics of an exhaustive search.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchStats {
    /// Distinct program states reached at scheduling points.
    pub distinct_states: usize,
    /// Schedules cut short at an already explored state.
    pub pruned_schedules: usize,
    /// Schedules stopped by the step bound (the search is not exhaustive then); counted
    /// by the engine.
    pub depth_bound_hits: usize,
    /// Most choices made in one schedule.
    pub max_depth: usize,
    /// Every alternative was explored.
    pub finished: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Operation,
    Bool,
    Int,
}

/// A choice point of the current path and the alternatives left there.
struct Choice {
    kind: Kind,
    options: Vec<u64>,
    taken: usize,
}

/// Depth-first search with visited-state caching (`--sch-pex dfs`); with a random
/// generator the alternatives at each new choice point are tried in random order
/// (`--sch-pex random`, the default).
pub struct ExhaustiveStrategy {
    max_steps: usize,
    rng: Option<SharedRandom>,
    path: Vec<Choice>,
    /// Position in `path` of the current schedule.
    pos: usize,
    /// Smallest depth each state was reached at.
    visited: HashMap<u64, usize>,
    stats: SearchStats,
    error: Option<String>,
}

impl ExhaustiveStrategy {
    pub fn new(max_steps: usize, rng: Option<SharedRandom>) -> Self {
        ExhaustiveStrategy {
            max_steps,
            rng,
            path: Vec::new(),
            pos: 0,
            visited: HashMap::new(),
            stats: SearchStats::default(),
            error: None,
        }
    }

    /// The next decision of the current schedule: the path's choice while replaying it,
    /// else a new choice point with `options` (never empty).
    fn decide(&mut self, kind: Kind, mut options: Vec<u64>) -> u64 {
        if self.pos < self.path.len() {
            let c = &self.path[self.pos];
            let v = c.options[c.taken];
            if c.kind != kind || !options.contains(&v) {
                // only possible if the program is not deterministic given its choices
                self.error.get_or_insert_with(|| {
                    "Exhaustive search: the program did not repeat a schedule given the same choices.".into()
                });
            }
            self.pos += 1;
            return v;
        }
        if let Some(rng) = &self.rng {
            let mut i = options.len();
            while i > 1 {
                let j = rng.borrow_mut().next_below(i as i32) as usize;
                i -= 1;
                options.swap(i, j);
            }
        }
        let v = options[0];
        self.path.push(Choice { kind, options, taken: 0 });
        self.pos += 1;
        v
    }
}

impl Strategy for ExhaustiveStrategy {
    fn next_operation(&mut self, _current: u64, ops: &[OpInfo]) -> Option<u64> {
        let enabled: Vec<u64> = ops.iter().filter(|o| o.enabled).map(|o| o.id).collect();
        if enabled.is_empty() {
            return None;
        }
        Some(self.decide(Kind::Operation, enabled))
    }

    fn next_bool(&mut self, _max: i32) -> bool {
        self.decide(Kind::Bool, vec![0, 1]) == 1
    }

    fn next_int(&mut self, max: i32) -> i32 {
        let n = max.max(1) as u64;
        self.decide(Kind::Int, (0..n).collect()) as i32
    }

    fn prepare_next_iteration(&mut self) -> bool {
        self.stats.max_depth = self.stats.max_depth.max(self.pos);
        self.pos = 0;
        // backtrack to the deepest choice point with an untried alternative
        while let Some(c) = self.path.last_mut() {
            if c.taken + 1 < c.options.len() {
                c.taken += 1;
                return true;
            }
            self.path.pop();
        }
        self.stats.finished = true;
        false
    }

    fn scheduled_steps(&self) -> usize {
        self.pos
    }

    fn has_reached_max_steps(&self) -> bool {
        self.max_steps != 0 && self.pos >= self.max_steps
    }

    fn is_fair(&self) -> bool {
        false
    }

    fn description(&self) -> String {
        match &self.rng {
            Some(r) => format!("pex[random, seed '{}']", r.borrow().seed),
            None => "pex[dfs]".into(),
        }
    }

    fn replay_error(&self) -> Option<String> {
        self.error.clone()
    }

    fn wants_state_hash(&self) -> bool {
        // states on the path to the branch being explored are already recorded
        self.pos >= self.path.len()
    }

    fn observe_state(&mut self, hash: u64) -> bool {
        match self.visited.get_mut(&hash) {
            Some(d) if *d <= self.pos => {
                self.stats.pruned_schedules += 1;
                false
            }
            Some(d) => {
                *d = self.pos;
                true
            }
            None => {
                self.visited.insert(hash, self.pos);
                self.stats.distinct_states += 1;
                true
            }
        }
    }

    fn exhaustive_stats(&self) -> Option<SearchStats> {
        // the last schedule counts even when no next iteration was prepared (a bug, the
        // schedule limit or a timeout ended the search)
        Some(SearchStats { max_depth: self.stats.max_depth.max(self.pos), ..self.stats.clone() })
    }
}

impl SearchStats {
    /// The report lines of the search.
    pub fn text(&self, schedules: usize, max_steps: usize, prefix: &str) -> String {
        let mut out = format!(
            "{} Exhaustive search: explored {} distinct state{} in {} schedule{} ({} cut at an explored state), max depth {}.",
            prefix,
            self.distinct_states,
            if self.distinct_states == 1 { "" } else { "s" },
            schedules,
            if schedules == 1 { "" } else { "s" },
            self.pruned_schedules,
            self.max_depth
        );
        if self.finished && self.depth_bound_hits == 0 {
            out.push_str(&format!("\n{} Search finished: every behaviour of the test was explored.", prefix));
        } else if self.finished {
            out.push_str(&format!(
                "\n{} Search finished, but {} schedule{} hit the max-steps bound of '{}': behaviours beyond it were not explored.",
                prefix,
                self.depth_bound_hits,
                if self.depth_bound_hits == 1 { "" } else { "s" },
                max_steps
            ));
        } else {
            out.push_str(&format!(
                "\n{} Search incomplete: stopped before exploring every behaviour (bug found, schedule limit or timeout).",
                prefix
            ));
        }
        out
    }
}
