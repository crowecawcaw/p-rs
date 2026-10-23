//! Automatic exploration: persistent arms, progress per choice, and bounded exhaustion.
//!
//! Progress changes allocation, never the stopping rule. Each enabled arm retains a
//! minimum work share even when it stops finding novelty. Only a completed, unbounded
//! exhaustive search can end checking early; cyclic hot monitors still need fair runs.
use std::cell::RefCell;
use std::rc::Rc;

use rustc_hash::FxHashSet;

use super::exhaustive::{ExhaustiveStrategy, SearchStats};
use super::feedback::{ControlledRandom, FeedbackGuidedStrategy, RandomScheduler, Source};
use super::random::RandomValueGenerator;
use super::strategy::{OpInfo, PctStrategy, RandomStrategy, SharedRandom, Strategy};

const RANDOM: usize = 0;
#[cfg(test)]
const GUIDED: usize = 1;
const DFS: usize = 3;
const WINDOW: u64 = 16_384;
const MAX_STATES: usize = 100_000;
const MAX_ALTERNATIVES: usize = 1_000_000;
const MAX_FEATURES: usize = 100_000;

/// Credit-based allocation in units of controlled choices, rather than executions or
/// seconds. Credits carry over between epochs, so repeatedly changing the preferred
/// arm cannot erase the other arms' entitlement to work.
struct Allocator {
    credit: [f64; 4],
    work: [u64; 4],
    reward: [f64; 4],
    rate: [f64; 4],
    weights: [f64; 4],
    enabled: [bool; 4],
}
impl Allocator {
    fn new(exhaustive: bool) -> Self {
        let mut s = Self {
            credit: [0.; 4],
            work: [0; 4],
            reward: [0.; 4],
            rate: [0.; 4],
            weights: [0.4, 0.4, 0.1, 0.1],
            enabled: [true, true, true, exhaustive],
        };
        s.reweight();
        s
    }
    fn reweight(&mut self) {
        // 80% reserved work: 30% random, 30% guided, 10% PCT, 10% DFS.
        // Allocate the remaining 20% by recent novelty/work, with equal priors.
        let floor = [0.3, 0.3, 0.1, 0.1];
        let floor_sum: f64 = (0..4).filter(|&i| self.enabled[i]).map(|i| floor[i]).sum();
        let best = (0..4).filter(|&i| self.enabled[i]).map(|i| self.rate[i]).fold(0., f64::max);
        let scores: [f64; 4] = std::array::from_fn(|i| {
            if !self.enabled[i] {
                0.
            } else if best > 0. {
                0.1 + self.rate[i] / best
            } else {
                1.
            }
        });
        let sum: f64 = scores.iter().sum();
        self.weights =
            std::array::from_fn(|i| if self.enabled[i] { floor[i] + (1. - floor_sum) * scores[i] / sum } else { 0. });
    }
    fn charge(&mut self, arm: usize, work: u64, reward: f64) {
        let work = work.max(1);
        for i in 0..4 {
            if self.enabled[i] {
                self.credit[i] += work as f64 * self.weights[i];
            }
        }
        self.credit[arm] -= work as f64;
        self.work[arm] += work;
        self.reward[arm] += reward;
        if self.work[arm] >= WINDOW {
            let observed = self.reward[arm] / self.work[arm] as f64;
            self.rate[arm] = self.rate[arm] * 0.5 + observed * 0.5;
            self.work[arm] = 0;
            self.reward[arm] = 0.;
            self.reweight();
        }
    }
    fn next(&self) -> usize {
        (0..4)
            .filter(|&i| self.enabled[i])
            .max_by(|&a, &b| self.credit[a].total_cmp(&self.credit[b]).then_with(|| b.cmp(&a)))
            .expect("sampled arms always remain enabled")
    }
}

pub(crate) struct AutoStrategy {
    arms: Vec<Box<dyn Strategy>>,
    allocation: Allocator,
    active: usize,
    choices: u64,
    reward: f64,
    probing: bool,
    probe_work: u64,
    alternatives: usize,
    dfs_bounds: usize,
    dfs_prunes: usize,
    has_hot: bool,
    complete: bool,
    features: FxHashSet<u64>,
    failures: FxHashSet<String>,
}
impl AutoStrategy {
    pub(crate) fn new(seed: u32, unfair_steps: usize, fair_steps: usize, has_hot: bool, exhaustive: bool) -> Self {
        let rng = || -> SharedRandom { Rc::new(RefCell::new(RandomValueGenerator::new(seed))) };
        Self {
            arms: vec![
                Box::new(RandomStrategy::new(fair_steps, rng())),
                Box::new(
                    FeedbackGuidedStrategy::new(
                        fair_steps,
                        seed,
                        Box::new(RandomScheduler::new(Source::Controlled(ControlledRandom::new(seed)))),
                    )
                    .with_fresh_exploration(25),
                ),
                Box::new(PctStrategy::new(unfair_steps, 10, rng())),
                Box::new(ExhaustiveStrategy::new(unfair_steps, Some(rng()))),
            ],
            allocation: Allocator::new(exhaustive),
            active: if exhaustive { DFS } else { RANDOM },
            choices: 0,
            reward: 0.,
            probing: exhaustive,
            probe_work: 0,
            alternatives: 0,
            dfs_bounds: 0,
            dfs_prunes: 0,
            has_hot,
            complete: false,
            features: FxHashSet::default(),
            failures: FxHashSet::default(),
        }
    }
    fn choice(&mut self, alternatives: usize) -> bool {
        self.choices += 1;
        if self.active == DFS && self.arms[DFS].wants_state_hash() {
            self.alternatives = self.alternatives.saturating_add(alternatives);
            if self.alternatives > MAX_ALTERNATIVES {
                self.disable_dfs();
                return false;
            }
        }
        true
    }
    fn disable_dfs(&mut self) {
        self.allocation.enabled[DFS] = false;
        self.allocation.reweight();
    }
    fn feature(&mut self, value: impl std::hash::Hash) {
        use std::hash::Hasher;
        if self.features.len() < MAX_FEATURES {
            let mut h = rustc_hash::FxHasher::default();
            value.hash(&mut h);
            if self.features.insert(h.finish()) {
                self.reward += 1.;
            }
        }
    }
}

impl Strategy for AutoStrategy {
    fn next_operation(&mut self, current: u64, ops: &[OpInfo]) -> Option<u64> {
        if !self.choice(ops.iter().filter(|o| o.enabled).count()) {
            return None;
        }
        self.arms[self.active].next_operation(current, ops)
    }
    fn next_bool(&mut self, max: i32) -> bool {
        if !self.choice(2) {
            return false;
        }
        self.arms[self.active].next_bool(max)
    }
    fn next_int(&mut self, max: i32) -> i32 {
        if !self.choice(max.max(1) as usize) {
            return 0;
        }
        self.arms[self.active].next_int(max)
    }
    fn prepare_next_iteration(&mut self) -> bool {
        let bounded = self.arms[self.active].has_reached_max_steps();
        let more = self.arms[self.active].prepare_next_iteration();
        if self.active == DFS {
            self.dfs_bounds += usize::from(bounded);
            if let Some(s) = self.arms[DFS].exhaustive_stats() {
                // Closed repeated states represent useful graph convergence; merely
                // generating fresh states is not evidence that exhaustion is close.
                self.reward += s.pruned_schedules.saturating_sub(self.dfs_prunes) as f64;
                self.dfs_prunes = s.pruned_schedules;
                if !more
                    && s.finished
                    && self.allocation.enabled[DFS]
                    && self.dfs_bounds == 0
                    && (!self.has_hot || s.cycle_prunes == 0)
                {
                    self.complete = true;
                    return false;
                }
            }
            if !more {
                self.disable_dfs();
            }
        }
        if self.probing {
            self.probe_work += self.choices.max(1);
            if self.probe_work < WINDOW && self.allocation.enabled[DFS] {
                self.choices = 0;
                self.reward = 0.;
                return true;
            }
            self.probing = false;
        } else {
            self.allocation.charge(self.active, self.choices, self.reward);
        }
        self.choices = 0;
        self.reward = 0.;
        self.active = self.allocation.next();
        true
    }
    fn scheduled_steps(&self) -> usize {
        self.arms[self.active].scheduled_steps()
    }
    fn has_reached_max_steps(&self) -> bool {
        self.arms[self.active].has_reached_max_steps()
    }
    fn is_fair(&self) -> bool {
        self.arms[self.active].is_fair()
    }
    fn description(&self) -> String {
        "auto".into()
    }
    fn diverged(&self) -> bool {
        (self.active == DFS && !self.allocation.enabled[DFS]) || self.arms[self.active].diverged()
    }
    fn replay_error(&self) -> Option<String> {
        self.arms[self.active].replay_error()
    }
    fn wants_minhash(&self) -> bool {
        self.active == 1
    }
    fn wants_feedback(&self) -> bool {
        true
    }
    fn observe_timeline(&mut self, timeline: &str, minhash: &[i32], reached: &[(String, usize)], satisfied: &[String]) {
        for token in timeline.split(';').filter(|s| !s.is_empty()) {
            self.feature((0, token));
        }
        for (name, states) in reached {
            self.feature((1, name, states));
        }
        self.arms[self.active].observe_timeline(timeline, minhash, reached, satisfied);
    }
    fn observe_progress(&mut self, new_coverage: usize, bug: Option<&str>) {
        self.reward += new_coverage as f64;
        if let Some(bug) = bug {
            let key = failure_class(bug);
            if self.failures.len() < MAX_FEATURES && self.failures.insert(key) {
                self.reward += 16.;
            }
        }
    }
    fn wants_state_hash(&self) -> bool {
        self.active == DFS && self.allocation.enabled[DFS] && self.arms[DFS].wants_state_hash()
    }
    fn observe_state(&mut self, hash: u64) -> bool {
        if self.arms[DFS].exhaustive_stats().is_some_and(|s| s.distinct_states >= MAX_STATES) {
            self.disable_dfs();
            return false;
        }
        self.arms[DFS].observe_state(hash)
    }
    fn exhaustive_stats(&self) -> Option<SearchStats> {
        if !self.allocation.enabled[DFS] && self.alternatives == 0 {
            return None;
        }
        self.arms[DFS].exhaustive_stats().map(|s| SearchStats {
            finished: self.complete,
            depth_bound_hits: self.dfs_bounds,
            ..s
        })
    }
    fn search_complete(&self) -> bool {
        self.complete
    }
}

/// A failure site, independent of formatted values and machine instance numbers.
fn failure_class(bug: &str) -> String {
    if let Some(rest) = bug.strip_prefix("Assertion Failed: ") {
        let end = rest
            .char_indices()
            .find_map(|(end, c)| {
                if !c.is_whitespace() {
                    return None;
                }
                let mut location = rest[..end].rsplitn(3, ':');
                location.next()?.parse::<usize>().ok()?;
                location.next()?.parse::<usize>().ok()?;
                Some(end)
            })
            .unwrap_or(rest.len());
        return format!("assertion:{}", &rest[..end]);
    }
    let mut key = String::new();
    let mut number = false;
    for c in bug.chars() {
        if c.is_ascii_digit() {
            if !number {
                key.push('#');
            }
            number = true;
        } else {
            key.push(c);
            number = false;
        }
    }
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn allocator_preserves_floors_under_stalls_and_changing_rewards() {
        let mut a = Allocator::new(true);
        let mut totals = [0u64; 4];
        for epoch in 0..100_000 {
            let arm = a.next();
            let cost = [1, 7, 13, 3][arm];
            totals[arm] += cost;
            a.charge(arm, cost, if arm == (epoch / 1000) % 4 { 1. } else { 0. });
        }
        let total = totals.iter().sum::<u64>() as f64;
        for (i, floor) in [0.3, 0.3, 0.1, 0.1].iter().enumerate() {
            assert!(totals[i] as f64 / total >= floor - 0.001, "{totals:?}");
        }
        assert!(a.rate.iter().any(|v| *v > 0.));
    }
    #[test]
    fn allocator_rewards_progress_but_not_repeated_failures() {
        assert_eq!(
            failure_class("Assertion Failed: dir with spaces/x.p:3:2 value 123"),
            failure_class("Assertion Failed: dir with spaces/x.p:3:2 value 456")
        );
        assert_ne!(
            failure_class("Assertion Failed: dir with spaces/x.p:3:2 value"),
            failure_class("Assertion Failed: dir with spaces/x.p:4:2 value")
        );

        let mut a = Allocator::new(true);
        a.charge(GUIDED, WINDOW, 100.);
        assert!(a.weights[GUIDED] > a.weights[RANDOM]);
        assert_eq!(
            failure_class("Assertion Failed: x.p:3:2 value 123"),
            failure_class("Assertion Failed: x.p:3:2 value 456")
        );
        assert_ne!(failure_class("Assertion Failed: x.p:3:2 value"), failure_class("Assertion Failed: x.p:4:2 value"));
    }
}
