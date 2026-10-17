//! Scheduling strategies (`PChecker.SystematicTesting.Strategies`).
//!
//! Every strategy consumes random numbers from the shared [`RandomValueGenerator`] in
//! exactly the order the upstream implementation does, so a seed reproduces the same
//! schedule here as it does there.

use std::cell::RefCell;
use std::rc::Rc;

use crate::random::RandomValueGenerator;

/// Identity and status of a schedulable operation as seen by a strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpInfo {
    pub id: u64,
    pub enabled: bool,
    /// Kind of the scheduling point the operation last stopped at (`AsyncOperation.Type`).
    pub kind: crate::runtime::SchedKind,
    /// Target of the operation's most recent send (`AsyncOperation.MessageReceiver`).
    pub receiver: Option<u64>,
}

pub type SharedRandom = Rc<RefCell<RandomValueGenerator>>;

pub trait Strategy {
    /// Picks the next operation to schedule among `ops` (sorted by id). `current` is the
    /// id of the operation that reached the scheduling point.
    fn next_operation(&mut self, current: u64, ops: &[OpInfo]) -> Option<u64>;
    fn next_bool(&mut self, max: i32) -> bool;
    fn next_int(&mut self, max: i32) -> i32;
    /// Prepares the next iteration; `false` means exploration is exhausted.
    fn prepare_next_iteration(&mut self) -> bool;
    fn scheduled_steps(&self) -> usize;
    fn has_reached_max_steps(&self) -> bool;
    fn is_fair(&self) -> bool;
    fn description(&self) -> String;
    fn reset(&mut self);
    /// Error of a replay strategy whose trace could not be followed.
    fn replay_error(&self) -> Option<String> {
        None
    }
    /// Feedback from a finished schedule (`IFeedbackGuidedStrategy.ObserveRunningResults`):
    /// its abstract timeline and timeline MinHash, the states each scenario reached and the
    /// scenarios it satisfied.
    fn observe_timeline(
        &mut self,
        _timeline: &str,
        _minhash: &[i32],
        _reached: &[(String, usize)],
        _satisfied: &[String],
    ) {
    }
    /// Whether [`Strategy::observe_timeline`] is used (avoids computing MinHashes otherwise).
    fn wants_feedback(&self) -> bool {
        false
    }
    /// Whether [`Strategy::observe_state`] is used (avoids hashing the state otherwise).
    fn wants_state_hash(&self) -> bool {
        false
    }
    /// The program-state hash (see [`crate::statehash`]) at the scheduling point about to
    /// be decided by [`Strategy::next_operation`]. Returning `false` ends the schedule
    /// there, without a deadlock or liveness check (the state was explored before).
    fn observe_state(&mut self, _hash: u64) -> bool {
        true
    }
    /// Statistics of an exhaustive search.
    fn exhaustive_stats(&self) -> Option<crate::exhaustive::SearchStats> {
        None
    }
}

fn enabled_ids(ops: &[OpInfo]) -> Vec<u64> {
    ops.iter().filter(|o| o.enabled).map(|o| o.id).collect()
}

// ---------------------------------------------------------------------------

pub struct RandomStrategy {
    pub max_steps: usize,
    pub steps: usize,
    pub rng: SharedRandom,
}

impl RandomStrategy {
    pub fn new(max_steps: usize, rng: SharedRandom) -> Self {
        RandomStrategy { max_steps, steps: 0, rng }
    }
}

impl Strategy for RandomStrategy {
    fn next_operation(&mut self, _current: u64, ops: &[OpInfo]) -> Option<u64> {
        let enabled = enabled_ids(ops);
        if enabled.is_empty() {
            return None;
        }
        let idx = self.rng.borrow_mut().next_below(enabled.len() as i32) as usize;
        self.steps += 1;
        Some(enabled[idx])
    }
    fn next_bool(&mut self, max: i32) -> bool {
        let r = self.rng.borrow_mut().next_below(max) == 0;
        self.steps += 1;
        r
    }
    fn next_int(&mut self, max: i32) -> i32 {
        let r = self.rng.borrow_mut().next_below(max);
        self.steps += 1;
        r
    }
    fn prepare_next_iteration(&mut self) -> bool {
        self.steps = 0;
        true
    }
    fn scheduled_steps(&self) -> usize {
        self.steps
    }
    fn has_reached_max_steps(&self) -> bool {
        self.max_steps != 0 && self.steps >= self.max_steps
    }
    fn is_fair(&self) -> bool {
        true
    }
    fn description(&self) -> String {
        format!("random[seed '{}']", self.rng.borrow().seed)
    }
    fn reset(&mut self) {
        self.steps = 0;
    }
}

// ---------------------------------------------------------------------------

pub struct ProbabilisticRandomStrategy {
    pub inner: RandomStrategy,
    pub coin_flips: usize,
}

impl ProbabilisticRandomStrategy {
    pub fn new(max_steps: usize, coin_flips: usize, rng: SharedRandom) -> Self {
        ProbabilisticRandomStrategy { inner: RandomStrategy::new(max_steps, rng), coin_flips }
    }

    fn should_current_change(&mut self) -> bool {
        for _ in 0..self.coin_flips {
            if self.inner.rng.borrow_mut().next_below(2) == 1 {
                return false;
            }
        }
        true
    }
}

impl Strategy for ProbabilisticRandomStrategy {
    fn next_operation(&mut self, current: u64, ops: &[OpInfo]) -> Option<u64> {
        let enabled = enabled_ids(ops);
        if enabled.is_empty() {
            return None;
        }
        self.inner.steps += 1;
        if enabled.len() > 1 {
            let current_enabled = ops.iter().any(|o| o.id == current && o.enabled);
            if !self.should_current_change() && current_enabled {
                return Some(current);
            }
        }
        let idx = self.inner.rng.borrow_mut().next_below(enabled.len() as i32) as usize;
        Some(enabled[idx])
    }
    fn next_bool(&mut self, max: i32) -> bool {
        self.inner.next_bool(max)
    }
    fn next_int(&mut self, max: i32) -> i32 {
        self.inner.next_int(max)
    }
    fn prepare_next_iteration(&mut self) -> bool {
        self.inner.prepare_next_iteration()
    }
    fn scheduled_steps(&self) -> usize {
        self.inner.steps
    }
    fn has_reached_max_steps(&self) -> bool {
        self.inner.has_reached_max_steps()
    }
    fn is_fair(&self) -> bool {
        true
    }
    fn description(&self) -> String {
        format!("probabilistic[seed '{}', coin flips '{}']", self.inner.rng.borrow().seed, self.coin_flips)
    }
    fn reset(&mut self) {
        self.inner.reset()
    }
}

// ---------------------------------------------------------------------------

pub struct PctStrategy {
    pub max_steps: usize,
    pub steps: usize,
    pub max_switch_points: usize,
    pub schedule_length: usize,
    pub prioritized: Vec<u64>,
    /// Kept sorted (upstream uses a `SortedSet<int>`).
    pub change_points: Vec<usize>,
    pub rng: SharedRandom,
}

impl PctStrategy {
    pub fn new(max_steps: usize, max_switch_points: usize, rng: SharedRandom) -> Self {
        PctStrategy {
            max_steps,
            steps: 0,
            max_switch_points,
            schedule_length: 0,
            prioritized: Vec::new(),
            change_points: Vec::new(),
            rng,
        }
    }

    fn highest_priority_enabled(&self, enabled: &[u64]) -> Option<u64> {
        self.prioritized.iter().copied().find(|p| enabled.contains(p))
    }

    fn add_point(&mut self, p: usize) {
        if let Err(i) = self.change_points.binary_search(&p) {
            self.change_points.insert(i, p);
        }
    }

    fn remove_point(&mut self, p: usize) {
        if let Ok(i) = self.change_points.binary_search(&p) {
            self.change_points.remove(i);
        }
    }

    fn move_change_point_forward(&mut self) {
        self.remove_point(self.steps);
        let mut p = self.steps + 1;
        while self.change_points.binary_search(&p).is_ok() {
            p += 1;
        }
        self.add_point(p);
    }
}

impl Strategy for PctStrategy {
    fn next_operation(&mut self, current: u64, ops: &[OpInfo]) -> Option<u64> {
        let enabled = enabled_ids(ops);
        if enabled.is_empty() {
            return None;
        }
        if self.prioritized.is_empty() {
            self.prioritized.push(current);
        }
        for op in &enabled {
            if !self.prioritized.contains(op) {
                let idx = self.rng.borrow_mut().next_below(self.prioritized.len() as i32) as usize + 1;
                self.prioritized.insert(idx, *op);
            }
        }
        if self.change_points.binary_search(&self.steps).is_ok() {
            if enabled.len() == 1 {
                self.move_change_point_forward();
            } else if let Some(p) = self.highest_priority_enabled(&enabled) {
                self.prioritized.retain(|x| *x != p);
                self.prioritized.push(p);
            }
        }
        let next = self.highest_priority_enabled(&enabled);
        self.steps += 1;
        next
    }
    fn next_bool(&mut self, max: i32) -> bool {
        let r = self.rng.borrow_mut().next_below(max) == 0;
        self.steps += 1;
        r
    }
    fn next_int(&mut self, max: i32) -> i32 {
        let r = self.rng.borrow_mut().next_below(max);
        self.steps += 1;
        r
    }
    fn prepare_next_iteration(&mut self) -> bool {
        self.schedule_length = self.schedule_length.max(self.steps);
        self.steps = 0;
        self.prioritized.clear();
        self.change_points.clear();
        let mut range: Vec<usize> = (0..self.schedule_length).collect();
        let mut idx = range.len();
        while idx > 1 {
            idx -= 1;
            let point = self.rng.borrow_mut().next_below(self.schedule_length as i32) as usize;
            range.swap(idx, point);
        }
        for p in range.into_iter().take(self.max_switch_points) {
            self.add_point(p);
        }
        true
    }
    fn scheduled_steps(&self) -> usize {
        self.steps
    }
    fn has_reached_max_steps(&self) -> bool {
        self.max_steps != 0 && self.steps >= self.max_steps
    }
    fn is_fair(&self) -> bool {
        false
    }
    fn description(&self) -> String {
        let pts: Vec<String> = self.change_points.iter().map(|p| p.to_string()).collect();
        format!(
            "pct[priority change points '{}' [{}], seed '{}']",
            self.max_switch_points,
            pts.join(", "),
            self.rng.borrow().seed
        )
    }
    fn reset(&mut self) {
        self.schedule_length = 0;
        self.steps = 0;
        self.prioritized.clear();
        self.change_points.clear();
    }
}

// ---------------------------------------------------------------------------

/// A prefix strategy followed by a suffix strategy once the prefix hits its bound.
pub struct ComboStrategy {
    pub prefix: Box<dyn Strategy>,
    pub suffix: Box<dyn Strategy>,
}

impl Strategy for ComboStrategy {
    fn next_operation(&mut self, current: u64, ops: &[OpInfo]) -> Option<u64> {
        if self.prefix.has_reached_max_steps() {
            self.suffix.next_operation(current, ops)
        } else {
            self.prefix.next_operation(current, ops)
        }
    }
    fn next_bool(&mut self, max: i32) -> bool {
        if self.prefix.has_reached_max_steps() {
            self.suffix.next_bool(max)
        } else {
            self.prefix.next_bool(max)
        }
    }
    fn next_int(&mut self, max: i32) -> i32 {
        if self.prefix.has_reached_max_steps() {
            self.suffix.next_int(max)
        } else {
            self.prefix.next_int(max)
        }
    }
    fn prepare_next_iteration(&mut self) -> bool {
        let a = self.prefix.prepare_next_iteration();
        let b = self.suffix.prepare_next_iteration();
        a || b
    }
    fn scheduled_steps(&self) -> usize {
        if self.prefix.has_reached_max_steps() {
            self.suffix.scheduled_steps() + self.prefix.scheduled_steps()
        } else {
            self.prefix.scheduled_steps()
        }
    }
    fn has_reached_max_steps(&self) -> bool {
        self.suffix.has_reached_max_steps()
    }
    fn is_fair(&self) -> bool {
        self.suffix.is_fair()
    }
    fn description(&self) -> String {
        format!("combo[{},{}]", self.prefix.description(), self.suffix.description())
    }
    fn reset(&mut self) {
        self.prefix.reset();
        self.suffix.reset();
    }
}

// ---------------------------------------------------------------------------

/// One recorded decision of a schedule.
#[derive(Debug, Clone, PartialEq)]
pub enum ScheduleStep {
    Operation(u64),
    Bool(bool),
    Int(i32),
}

/// A recorded schedule (`ScheduleTrace`), replayable with [`ReplayStrategy`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ScheduleTrace {
    pub is_fair: bool,
    pub liveness_temperature_threshold: Option<usize>,
    pub test_name: Option<String>,
    /// `--prs-time:<header>`: time tracking mode and profile (see `TimeOptions`).
    pub time: Option<String>,
    /// `--prs-latency:<key>=<value>`: the resolved profile entries.
    pub latency: Vec<(String, String)>,
    pub steps: Vec<ScheduleStep>,
}

impl ScheduleTrace {
    /// The `.schedule` file format of the upstream checker.
    pub fn to_text(&self) -> String {
        let mut s = String::new();
        if self.is_fair {
            s.push_str("--fair-scheduling\n");
        }
        if let Some(t) = self.liveness_temperature_threshold {
            s.push_str(&format!("--liveness-temperature-threshold:{}\n", t));
        }
        if let Some(n) = &self.test_name {
            s.push_str(&format!("--test-method:{}\n", n));
        }
        if let Some(h) = &self.time {
            s.push_str(&format!("--prs-time:{}\n", h));
        }
        for (k, v) in &self.latency {
            s.push_str(&format!("--prs-latency:{}={}\n", k, v));
        }
        // steps are newline-separated, with no newline after the last (upstream format)
        for (i, st) in self.steps.iter().enumerate() {
            if i > 0 {
                s.push('\n');
            }
            match st {
                ScheduleStep::Operation(id) => s.push_str(&format!("({})", id)),
                ScheduleStep::Bool(b) => s.push_str(if *b { "True" } else { "False" }),
                ScheduleStep::Int(i) => s.push_str(&i.to_string()),
            }
        }
        s
    }

    pub fn parse(text: &str) -> Result<ScheduleTrace, String> {
        let mut t = ScheduleTrace::default();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if line == "--fair-scheduling" {
                t.is_fair = true;
            } else if let Some(v) = line.strip_prefix("--liveness-temperature-threshold:") {
                t.liveness_temperature_threshold = Some(v.parse().map_err(|_| format!("bad threshold {v}"))?);
            } else if let Some(v) = line.strip_prefix("--test-method:") {
                t.test_name = Some(v.to_string());
            } else if let Some(v) = line.strip_prefix("--prs-time:") {
                t.time = Some(v.to_string());
            } else if let Some(v) = line.strip_prefix("--prs-latency:") {
                let (k, v) = v.rsplit_once('=').ok_or_else(|| format!("bad latency entry {v}"))?;
                t.latency.push((k.to_string(), v.to_string()));
            } else if line.starts_with("--") {
                continue;
            } else if let Some(v) = line.strip_prefix('(').and_then(|l| l.strip_suffix(')')) {
                t.steps.push(ScheduleStep::Operation(v.parse().map_err(|_| format!("bad operation {v}"))?));
            } else if line == "True" {
                t.steps.push(ScheduleStep::Bool(true));
            } else if line == "False" {
                t.steps.push(ScheduleStep::Bool(false));
            } else {
                t.steps.push(ScheduleStep::Int(line.parse().map_err(|_| format!("bad step {line}"))?));
            }
        }
        Ok(t)
    }
}

/// Replays a recorded schedule (`ReplayStrategy`).
pub struct ReplayStrategy {
    pub trace: ScheduleTrace,
    pub index: usize,
    pub is_fair: bool,
    pub error: Option<String>,
    /// Report the step bound as reached once this many steps were replayed (used to
    /// reproduce a schedule that ended at the bound; upstream replays never hit it).
    pub stop_at: Option<usize>,
}

impl ReplayStrategy {
    pub fn new(trace: ScheduleTrace) -> Self {
        let is_fair = trace.is_fair;
        ReplayStrategy { trace, index: 0, is_fair, error: None, stop_at: None }
    }
}

impl Strategy for ReplayStrategy {
    fn next_operation(&mut self, _current: u64, ops: &[OpInfo]) -> Option<u64> {
        let enabled = enabled_ids(ops);
        if enabled.is_empty() {
            return None;
        }
        if self.index >= self.trace.steps.len() {
            self.error = Some("Trace is not reproducible: execution is longer than trace.".into());
            return None;
        }
        match &self.trace.steps[self.index] {
            ScheduleStep::Operation(id) => {
                let id = *id;
                if !enabled.contains(&id) {
                    self.error =
                        Some("Trace is not reproducible: cannot detect id '".to_string() + &id.to_string() + "'.");
                    return None;
                }
                self.index += 1;
                Some(id)
            }
            _ => {
                self.error = Some("Trace is not reproducible: next step is not a scheduling choice.".into());
                None
            }
        }
    }
    fn next_bool(&mut self, _max: i32) -> bool {
        if let Some(ScheduleStep::Bool(b)) = self.trace.steps.get(self.index) {
            self.index += 1;
            *b
        } else {
            self.error = Some("Trace is not reproducible: next step is not a nondeterministic boolean choice.".into());
            false
        }
    }
    fn next_int(&mut self, _max: i32) -> i32 {
        if let Some(ScheduleStep::Int(i)) = self.trace.steps.get(self.index) {
            self.index += 1;
            *i
        } else {
            self.error = Some("Trace is not reproducible: next step is not a nondeterministic integer choice.".into());
            0
        }
    }
    fn prepare_next_iteration(&mut self) -> bool {
        false
    }
    fn scheduled_steps(&self) -> usize {
        self.index
    }
    fn has_reached_max_steps(&self) -> bool {
        self.stop_at.map(|k| self.index >= k).unwrap_or(false)
    }
    fn is_fair(&self) -> bool {
        self.is_fair
    }
    fn description(&self) -> String {
        "replay".into()
    }
    fn reset(&mut self) {
        self.index = 0;
    }
    fn replay_error(&self) -> Option<String> {
        self.error.clone()
    }
}
