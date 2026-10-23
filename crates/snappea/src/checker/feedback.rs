//! POS and feedback-guided strategies (`Strategies/Feedback`): the `pos` strategy
//! (`ScheduleAndInputStrategy` + `POSScheduler`) and `feedback`, `feedbackpct`,
//! `feedbackpos` (`FeedbackGuidedStrategy` over a random, PCT or POS scheduler with
//! mutable, replayable random choices).

use std::cell::RefCell;
use std::collections::{BTreeSet, HashSet};
use std::rc::Rc;

use crate::checker::random::DotNetRandom;
use crate::checker::runtime::SchedKind;
use crate::checker::strategy::{OpInfo, SharedRandom, Strategy};

/// Bounded prefix-context exploration. Counts are hints, never reasons to prune:
/// revisiting the same prefix can still discover a different continuation.
struct ChoiceGuide {
    prefix: u64,
    visits: rustc_hash::FxHashMap<(u64, u8, i32), Vec<u32>>,
    counters: usize,
    random: DotNetRandom,
}

impl ChoiceGuide {
    fn new(seed: u32) -> Self {
        Self { prefix: 0, visits: Default::default(), counters: 0, random: DotNetRandom::new(seed as i32) }
    }

    fn choose(&mut self, channel: u8, max: i32, proposed: i32, fresh: bool) -> i32 {
        let key = (self.prefix, channel, max);
        let mut chosen = proposed;
        if !self.visits.contains_key(&key) && self.counters + max as usize <= 262_144 {
            self.visits.insert(key, vec![0; max as usize]);
            self.counters += max as usize;
        }
        if let Some(counts) = self.visits.get_mut(&key) {
            // Preserve recorded prefixes; occasionally guide a fresh continuation.
            if fresh && self.random.next_below(4) == 0 {
                let least = *counts.iter().min().unwrap();
                let ties = counts.iter().filter(|n| **n == least).count();
                let rank = self.random.next_below(ties as i32) as usize;
                chosen = counts.iter().enumerate().filter(|(_, n)| **n == least).nth(rank).unwrap().0 as i32;
            }
            counts[chosen as usize] = counts[chosen as usize].saturating_add(1);
        }
        // Include domain and stream identity: input and scheduler indices differ.
        for value in [u64::from(channel), max as u64, chosen as u64] {
            self.prefix = (self.prefix ^ value.wrapping_add(1)).wrapping_mul(0x100000001b3);
        }
        chosen
    }
}

/// A failure site, independent of formatted values and machine instance numbers.
pub(crate) fn failure_class(bug: &str) -> String {
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

// ---- random sources ----------------------------------------------------------------

/// `RandomChoices<T>`: a replayable, mutable list of random values drawn on demand.
#[derive(Clone)]
struct RandomChoices<T: Copy> {
    random: Rc<RefCell<DotNetRandom>>,
    pos: usize,
    data: Vec<T>,
    generate: fn(&mut DotNetRandom) -> T,
}

impl<T: Copy> RandomChoices<T> {
    fn new(random: Rc<RefCell<DotNetRandom>>, generate: fn(&mut DotNetRandom) -> T) -> Self {
        RandomChoices { random, pos: 0, data: Vec::new(), generate }
    }

    fn next(&mut self) -> T {
        if self.pos == self.data.len() {
            let v = self.generate_new();
            self.data.push(v);
        }
        let v = self.data[self.pos];
        self.pos += 1;
        v
    }

    fn generate_new(&self) -> T {
        (self.generate)(&mut self.random.borrow_mut())
    }

    /// `Utils.MutateRandomChoices(choices, meanMutationCount, meanMutationSize, random)`.
    fn mutate(&self, mean_count: i64, mean_size: i64, legacy: bool, suffix: bool) -> Self {
        let n = self.data.len() as i64;
        let mean_count = (n / 3).min(mean_count).max(1);
        let mean_size = (n / 3).min(mean_size).max(1);
        let mut out =
            RandomChoices { random: self.random.clone(), pos: 0, data: self.data.clone(), generate: self.generate };
        if n == 0 && !legacy {
            return out;
        }
        // Resample a suffix as well as changing individual choices. This preserves a
        // useful prefix while allowing the continuation to change length and shape.
        if suffix && !legacy && self.random.borrow_mut().next_below(2) == 0 {
            let offset = self.random.borrow_mut().next_below(n as i32) as usize;
            out.data.truncate(offset);
            return out;
        }
        let geometric = if legacy { sample_geometric_upstream } else { sample_geometric };
        let r = self.random.borrow_mut().next_double();
        let mut mutations = geometric((1.0f32 / mean_count as f32) as f64, r);
        while mutations > 0 {
            mutations -= 1;
            let offset = self.random.borrow_mut().next_below(out.data.len() as i32) as usize;
            let r = self.random.borrow_mut().next_double();
            let size = geometric((1.0f32 / mean_size as f32) as f64, r);
            let mut i = offset;
            while (i as i64) < offset as i64 + size as i64 {
                if i >= out.data.len() {
                    break;
                }
                out.data[i] = out.generate_new();
                i += 1;
            }
        }
        out
    }
}

/// Geometric distribution on 1.., including its degenerate p=1 case. Mutation
/// segments of mean one must change one choice, rather than silently changing none.
fn sample_geometric(p: f64, random: f64) -> i32 {
    if p == 1.0 {
        return 1;
    }
    sample_geometric_upstream(p, random).max(1)
}

/// `Utils.SampleGeometric`, including its p=1 zero-length mutation bug. Keep the
/// upstream random stream for snappea-compat; native checking uses the corrected form.
fn sample_geometric_upstream(p: f64, random: f64) -> i32 {
    let r = ((1.0 - random).ln() / (1.0 - p).ln()).ceil();
    if r.is_nan() {
        0
    } else {
        r as i32
    }
}

/// `ControlledRandom`: random ints and doubles recorded so a schedule can be mutated.
#[derive(Clone)]
pub struct ControlledRandom {
    ints: RandomChoices<i32>,
    doubles: RandomChoices<f64>,
    legacy_mutations: bool,
    local_mutations: bool,
    guide: Option<(Rc<RefCell<ChoiceGuide>>, u8)>,
}

impl ControlledRandom {
    /// `new ControlledRandom(new System.Random(seed))`.
    pub fn new(seed: u32) -> ControlledRandom {
        ControlledRandom::from_random(Rc::new(RefCell::new(DotNetRandom::new(seed as i32))))
    }

    fn from_random(r: Rc<RefCell<DotNetRandom>>) -> ControlledRandom {
        ControlledRandom {
            ints: RandomChoices::new(r.clone(), |r| r.next()),
            doubles: RandomChoices::new(r, |r| r.next_double()),
            legacy_mutations: false,
            local_mutations: false,
            guide: None,
        }
    }

    pub fn next_below(&mut self, max: i32) -> i32 {
        if max == 0 {
            return 0;
        }
        let fresh = self.ints.pos == self.ints.data.len();
        let proposed = self.ints.next() % max;
        if let Some((guide, channel)) = &self.guide {
            let chosen = guide.borrow_mut().choose(*channel, max, proposed, fresh);
            // Retain the actual guided decision so corpus mutations start from
            // the choices that ran, rather than the discarded proposal.
            if chosen != proposed {
                self.ints.data[self.ints.pos - 1] = chosen;
            }
            chosen
        } else {
            proposed
        }
    }

    /// Fresh choices drawing from the same underlying generator.
    pub fn new_like(&self) -> ControlledRandom {
        let mut out =
            ControlledRandom::from_random(self.ints.random.clone()).with_legacy_mutations(self.legacy_mutations);
        out.local_mutations = self.local_mutations;
        out.guide = self.guide.clone();
        out
    }

    pub fn mutate(&self) -> ControlledRandom {
        ControlledRandom {
            ints: self.ints.mutate(
                if self.local_mutations { 1 } else { 5 },
                1,
                self.legacy_mutations,
                self.local_mutations,
            ),
            doubles: self.doubles.mutate(
                if self.local_mutations { 1 } else { 5 },
                1,
                self.legacy_mutations,
                self.local_mutations,
            ),
            legacy_mutations: self.legacy_mutations,
            local_mutations: self.local_mutations,
            guide: self.guide.clone(),
        }
    }

    fn prefix(&self, rewind: bool) -> Self {
        let mut out = self.clone();
        out.ints.data.truncate(out.ints.pos);
        out.doubles.data.truncate(out.doubles.pos);
        if rewind {
            out.ints.pos = 0;
            out.doubles.pos = 0;
        }
        out
    }

    fn choice_count(&self) -> usize {
        self.ints.data.len() + self.doubles.data.len()
    }

    /// Preserve upstream's zero-length mutations for seed compatibility.
    pub fn with_legacy_mutations(mut self, legacy: bool) -> Self {
        self.legacy_mutations = legacy;
        self
    }
}

/// The random source of a scheduler: the engine's generator (`pos`) or controlled choices
/// (feedback strategies).
#[derive(Clone)]
pub enum Source {
    Shared(SharedRandom),
    Controlled(ControlledRandom),
}

impl Source {
    fn next_below(&mut self, max: i32) -> i32 {
        match self {
            Source::Shared(r) => r.borrow_mut().next_below(max),
            Source::Controlled(c) => c.next_below(max),
        }
    }

    fn mutate(&self) -> Source {
        match self {
            Source::Controlled(c) => Source::Controlled(c.mutate()),
            other => other.clone(),
        }
    }

    fn new_like(&self) -> Source {
        match self {
            Source::Controlled(c) => Source::Controlled(c.new_like()),
            other => other.clone(),
        }
    }
}

// ---- schedulers ---------------------------------------------------------------------

/// `IScheduler`.
pub trait Scheduler {
    fn next_operation(&mut self, current: u64, ops: &[OpInfo]) -> Option<u64>;
    fn prepare_next_iteration(&mut self) -> bool;
    fn mutate(&self) -> Box<dyn Scheduler>;
    fn new_like(&self) -> Box<dyn Scheduler>;
    /// Replayable consumed prefix, and the number of retained random choices.
    fn checkpoint(&self, _rewind: bool) -> Option<(Box<dyn Scheduler>, usize)> {
        None
    }
    fn fresh_with_pos(&self, _pos: bool) -> Box<dyn Scheduler> {
        self.new_like()
    }
}

fn highest_priority(prioritized: &[u64], choices: &[&OpInfo]) -> Option<u64> {
    prioritized.iter().copied().find(|p| choices.iter().any(|c| c.id == *p))
}

/// `RandomScheduler`.
pub struct RandomScheduler {
    rng: Source,
}

impl RandomScheduler {
    pub fn new(rng: Source) -> RandomScheduler {
        RandomScheduler { rng }
    }
}

impl Scheduler for RandomScheduler {
    fn next_operation(&mut self, _current: u64, ops: &[OpInfo]) -> Option<u64> {
        let enabled: Vec<&OpInfo> = ops.iter().filter(|o| o.enabled).collect();
        match enabled.len() {
            0 => None,
            1 => Some(enabled[0].id),
            n => Some(enabled[self.rng.next_below(n as i32) as usize].id),
        }
    }
    fn prepare_next_iteration(&mut self) -> bool {
        true
    }
    fn mutate(&self) -> Box<dyn Scheduler> {
        Box::new(RandomScheduler { rng: self.rng.mutate() })
    }
    fn new_like(&self) -> Box<dyn Scheduler> {
        Box::new(RandomScheduler { rng: self.rng.new_like() })
    }
    fn checkpoint(&self, rewind: bool) -> Option<(Box<dyn Scheduler>, usize)> {
        let Source::Controlled(random) = &self.rng else { return None };
        let prefix = random.prefix(rewind);
        let count = prefix.choice_count();
        Some((Box::new(RandomScheduler::new(Source::Controlled(prefix))), count))
    }
    fn fresh_with_pos(&self, pos: bool) -> Box<dyn Scheduler> {
        if pos {
            Box::new(PosScheduler::new(self.rng.new_like()))
        } else {
            self.new_like()
        }
    }
}

/// `POSScheduler`: partial order sampling; sends to the same receiver race, everything
/// else is preferred.
pub struct PosScheduler {
    rng: Source,
    prioritized: Vec<u64>,
}

impl PosScheduler {
    pub fn new(rng: Source) -> PosScheduler {
        PosScheduler { rng, prioritized: Vec::new() }
    }

    fn prioritized_operation(&mut self, ops: &[&OpInfo], current: u64) -> u64 {
        if self.prioritized.is_empty() {
            self.prioritized.push(current);
        }
        for op in ops {
            if !self.prioritized.contains(&op.id) {
                let idx = self.rng.next_below(self.prioritized.len() as i32) as usize + 1;
                self.prioritized.insert(idx, op.id);
            }
        }
        let non_racing: Vec<&OpInfo> = ops.iter().copied().filter(|o| o.kind != SchedKind::Send).collect();
        match non_racing.len() {
            0 => {}
            1 => return non_racing[0].id,
            _ => {
                if let Some(n) = highest_priority(&self.prioritized, &non_racing) {
                    return n;
                }
            }
        }
        highest_priority(&self.prioritized, ops).expect("an enabled operation is prioritized")
    }
}

impl Scheduler for PosScheduler {
    fn next_operation(&mut self, current: u64, ops: &[OpInfo]) -> Option<u64> {
        let enabled: Vec<&OpInfo> = ops.iter().filter(|o| o.enabled).collect();
        if enabled.is_empty() {
            return None;
        }
        let next = self.prioritized_operation(&enabled, current);
        let info = enabled.iter().find(|o| o.id == next).copied().copied().unwrap();
        if info.kind == SchedKind::Send {
            for op in &enabled {
                if op.kind == SchedKind::Send && op.receiver == info.receiver {
                    self.prioritized.retain(|p| *p != op.id);
                }
            }
            self.prioritized.retain(|p| *p != next);
        }
        Some(next)
    }
    fn prepare_next_iteration(&mut self) -> bool {
        self.prioritized.clear();
        true
    }
    fn mutate(&self) -> Box<dyn Scheduler> {
        Box::new(PosScheduler::new(self.rng.mutate()))
    }
    fn new_like(&self) -> Box<dyn Scheduler> {
        Box::new(PosScheduler::new(self.rng.new_like()))
    }
    fn checkpoint(&self, rewind: bool) -> Option<(Box<dyn Scheduler>, usize)> {
        let Source::Controlled(random) = &self.rng else { return None };
        let prefix = random.prefix(rewind);
        let count = prefix.choice_count();
        Some((Box::new(PosScheduler::new(Source::Controlled(prefix))), count))
    }
    fn fresh_with_pos(&self, pos: bool) -> Box<dyn Scheduler> {
        if pos {
            self.new_like()
        } else {
            Box::new(RandomScheduler::new(self.rng.new_like()))
        }
    }
}

/// `PCTScheduler` (the feedback variant of PCT).
pub struct PctScheduler {
    rng: Source,
    steps: i64,
    max_switch_points: usize,
    schedule_length: i64,
    prioritized: Vec<u64>,
    change_points: BTreeSet<i64>,
}

impl PctScheduler {
    pub fn new(max_switch_points: usize, schedule_length: i64, rng: Source) -> PctScheduler {
        let mut s = PctScheduler {
            rng,
            steps: 0,
            max_switch_points,
            schedule_length,
            prioritized: Vec::new(),
            change_points: BTreeSet::new(),
        };
        if schedule_length != 0 {
            s.prepare_change_points(schedule_length);
        }
        s
    }

    fn prepare_change_points(&mut self, steps: i64) {
        let mut list: Vec<i64> = (0..steps).collect();
        for _ in 0..self.max_switch_points {
            let idx = self.rng.next_below(list.len() as i32) as usize;
            self.change_points.insert(list[idx]);
            list.remove(idx);
            if list.is_empty() {
                break;
            }
        }
    }

    fn move_change_point_forward(&mut self) {
        self.change_points.remove(&self.steps);
        let mut p = self.steps + 1;
        while self.change_points.contains(&p) {
            p += 1;
        }
        self.change_points.insert(p);
    }
}

impl Scheduler for PctScheduler {
    fn next_operation(&mut self, current: u64, ops: &[OpInfo]) -> Option<u64> {
        self.steps += 1;
        let enabled: Vec<&OpInfo> = ops.iter().filter(|o| o.enabled).collect();
        if enabled.is_empty() {
            if self.change_points.contains(&self.steps) {
                self.move_change_point_forward();
            }
            return None;
        }
        if self.prioritized.is_empty() {
            self.prioritized.push(current);
        }
        for op in &enabled {
            if !self.prioritized.contains(&op.id) {
                let idx = self.rng.next_below(self.prioritized.len() as i32) as usize + 1;
                self.prioritized.insert(idx, op.id);
            }
        }
        if self.change_points.contains(&self.steps) {
            if enabled.len() == 1 {
                self.move_change_point_forward();
            } else if let Some(p) = highest_priority(&self.prioritized, &enabled) {
                self.prioritized.retain(|x| *x != p);
                self.prioritized.push(p);
            }
        }
        highest_priority(&self.prioritized, &enabled)
    }
    fn prepare_next_iteration(&mut self) -> bool {
        self.schedule_length = self.schedule_length.max(self.steps);
        self.steps = 0;
        self.prioritized.clear();
        self.change_points.clear();
        true
    }
    fn mutate(&self) -> Box<dyn Scheduler> {
        Box::new(PctScheduler::new(self.max_switch_points, self.schedule_length, self.rng.mutate()))
    }
    fn new_like(&self) -> Box<dyn Scheduler> {
        Box::new(PctScheduler::new(self.max_switch_points, self.schedule_length, self.rng.new_like()))
    }
}

// ---- strategies ---------------------------------------------------------------------

/// `ScheduleAndInputStrategy` (the `pos` strategy): a scheduler for interleavings, the
/// engine's generator for data choices.
pub struct ScheduleAndInputStrategy {
    rng: SharedRandom,
    max_steps: usize,
    steps: usize,
    scheduler: Box<dyn Scheduler>,
}

impl ScheduleAndInputStrategy {
    pub fn new(max_steps: usize, rng: SharedRandom, scheduler: Box<dyn Scheduler>) -> Self {
        ScheduleAndInputStrategy { rng, max_steps, steps: 0, scheduler }
    }
}

impl Strategy for ScheduleAndInputStrategy {
    fn next_operation(&mut self, current: u64, ops: &[OpInfo]) -> Option<u64> {
        self.steps += 1;
        self.scheduler.next_operation(current, ops)
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
        self.scheduler.prepare_next_iteration()
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
        // sic: upstream reports this strategy as pct
        format!("pct[seed '{}']", self.rng.borrow().seed)
    }
}

struct GeneratorState {
    input: ControlledRandom,
    scheduler: Box<dyn Scheduler>,
}

type Generator = Rc<RefCell<GeneratorState>>;

type FrontierKey = (u32, u32, u32);

struct FrontierRecord {
    key: FrontierKey,
    count: usize,
    choices: usize,
    generator: Generator,
    exhausted: bool,
}

#[derive(Default)]
struct Frontiers {
    counts: rustc_hash::FxHashMap<FrontierKey, usize>,
    saved: Vec<FrontierRecord>,
    retained: usize,
    parent: Option<(FrontierKey, usize, usize)>,
}

struct GeneratorRecord {
    id: usize,
    priority: f64,
    mutation_budget: i32,
    generator: Generator,
    minhash: Vec<i32>,
}

const DEFAULT_MUTATION_BUDGET: i32 = 50;

/// `FeedbackGuidedStrategy`: keeps the generators of schedules with new timelines and
/// mutates them, prioritizing the most diverse.
pub struct FeedbackGuidedStrategy {
    generator: Generator,
    max_steps: usize,
    steps: usize,
    visited_timelines: HashSet<String>,
    saved: Vec<Rc<GeneratorRecord>>,
    pending_mutations: i32,
    should_explore_new: bool,
    visited: HashSet<usize>,
    current_parent: Option<Rc<GeneratorRecord>>,
    rnd: DotNetRandom,
    next_record_id: usize,
    suite_best_reached: std::collections::HashMap<String, usize>,
    suite_satisfied: HashSet<String>,
    fresh_percent: u8,
    code_guidance: bool,
    stalled: usize,
    native_features: HashSet<String>,
    failures: HashSet<String>,
    failure_seeds: Vec<Generator>,
    choice_guide: Option<Rc<RefCell<ChoiceGuide>>>,
    frontiers: Option<Frontiers>,
    state_frontiers: bool,
    pos_percent: u8,
}

impl FeedbackGuidedStrategy {
    /// `max_steps` is the step bound; `seed` seeds the input generator, the
    /// scheduler's generator and the explore/exploit choices independently.
    pub fn new(max_steps: usize, seed: u32, scheduler: Box<dyn Scheduler>) -> Self {
        FeedbackGuidedStrategy {
            generator: Rc::new(RefCell::new(GeneratorState { input: ControlledRandom::new(seed), scheduler })),
            max_steps,
            steps: 0,
            visited_timelines: HashSet::new(),
            saved: Vec::new(),
            pending_mutations: 0,
            should_explore_new: false,
            visited: HashSet::new(),
            current_parent: None,
            rnd: DotNetRandom::new(seed as i32),
            next_record_id: 0,
            suite_best_reached: std::collections::HashMap::new(),
            suite_satisfied: HashSet::new(),
            fresh_percent: 0,
            code_guidance: false,
            stalled: 0,
            native_features: HashSet::new(),
            failures: HashSet::new(),
            failure_seeds: Vec::new(),
            choice_guide: None,
            frontiers: None,
            state_frontiers: false,
            pos_percent: 0,
        }
    }

    /// Match the input generator's mutation semantics to the scheduler's source.
    pub fn with_legacy_mutations(self, legacy: bool) -> Self {
        self.generator.borrow_mut().input.legacy_mutations = legacy;
        self
    }

    /// Reserve a fraction of schedules for fresh choices, retaining the learned corpus
    /// and mutation budget between them. Zero preserves the original exploration policy.
    pub fn with_fresh_exploration(mut self, percent: u8) -> Self {
        self.fresh_percent = percent.min(100);
        self
    }

    /// Use coverage novelty instead of timeline MinHash; retain upstream behavior by default.
    pub fn with_code_guidance(mut self, enabled: bool) -> Self {
        self.code_guidance = enabled;
        self.generator.borrow_mut().input.local_mutations = enabled;
        self
    }

    /// Experimental next-choice exploration for random-scheduler feedback.
    /// Prefix contexts distinguish paths, not complete semantic program states.
    /// Call before running: replaces the scheduler with a controlled random one.
    pub fn with_choice_guidance(mut self, seed: u32) -> Self {
        let guide = Rc::new(RefCell::new(ChoiceGuide::new(seed)));
        let mut scheduler_random = ControlledRandom::new(seed);
        scheduler_random.guide = Some((guide.clone(), 0));
        let mut generator = self.generator.borrow_mut();
        generator.input.guide = Some((guide.clone(), 1));
        generator.scheduler = Box::new(RandomScheduler::new(Source::Controlled(scheduler_random)));
        drop(generator);
        self.choice_guide = Some(guide);
        self
    }

    /// Preserve input/schedule prefixes at increasing delivery counts, then sample
    /// fresh continuations. Requires a scheduler that supports checkpoints.
    pub fn with_frontier_guidance(mut self) -> Self {
        self.frontiers = Some(Frontiers::default());
        self
    }
    pub fn with_state_frontiers(mut self) -> Self {
        self.state_frontiers = true;
        self
    }
    pub fn with_pos_exploration(mut self, percent: u8) -> Self {
        self.pos_percent = percent.min(100);
        self
    }

    fn compute_diversity(&mut self, timeline: &str, hash: &[i32]) -> f64 {
        if !self.visited_timelines.insert(timeline.to_string()) {
            return 0.0;
        }
        if self.saved.is_empty() || hash.is_empty() {
            return 1.0;
        }
        let mut max_sim = 0;
        for r in &self.saved {
            let sim = hash.iter().zip(r.minhash.iter()).filter(|(a, b)| a == b).count();
            max_sim = max_sim.max(sim);
        }
        1.0 - max_sim as f64 / hash.len() as f64
    }

    fn new_generator(&mut self) -> Generator {
        let g = self.generator.borrow();
        let scheduler = if self.pos_percent > 0 {
            g.scheduler.fresh_with_pos(self.rnd.next_below(100) < i32::from(self.pos_percent))
        } else {
            g.scheduler.new_like()
        };
        Rc::new(RefCell::new(GeneratorState { input: g.input.new_like(), scheduler }))
    }

    fn mutate_generator(prev: &Generator) -> Generator {
        let g = prev.borrow();
        Rc::new(RefCell::new(GeneratorState { input: g.input.mutate(), scheduler: g.scheduler.mutate() }))
    }

    fn prepare_next_input(&mut self) {
        self.generator.borrow_mut().scheduler.prepare_next_iteration();
        if let Some(frontiers) = &mut self.frontiers {
            if let Some((key, count, prefix_choices)) = frontiers.parent.take() {
                let generator = self.generator.borrow();
                let (_, scheduling_choices) = generator.scheduler.checkpoint(false).expect("checkpoint scheduler");
                let choices = scheduling_choices + generator.input.ints.pos + generator.input.doubles.pos;
                if choices == prefix_choices {
                    // A prefix with no further choices has no continuation to sample.
                    if let Some(record) = frontiers.saved.iter_mut().find(|r| r.key == key && r.count == count) {
                        record.exhausted = true;
                    }
                }
            }
            frontiers.counts.clear();
            let available: Vec<_> = frontiers.saved.iter().filter(|record| !record.exhausted).collect();
            if !available.is_empty() && self.rnd.next_below(2) == 0 {
                let record = available[self.rnd.next_below(available.len() as i32) as usize];
                frontiers.parent = Some((record.key, record.count, record.choices));
                let parent = record.generator.borrow();
                self.generator = Rc::new(RefCell::new(GeneratorState {
                    input: parent.input.prefix(true),
                    scheduler: parent.scheduler.checkpoint(true).expect("frontier scheduler has checkpoints").0,
                }));
                return;
            }
        }
        if self.code_guidance {
            // Exploit recent progress promptly, share attempts with older frontiers, and
            // fall back to independent sampling when coverage stops being informative.
            let fresh = self.fresh_percent.max(if self.stalled >= 128 { 75 } else { 25 });
            if self.saved.is_empty() || self.rnd.next_below(100) < i32::from(fresh) {
                self.generator = self.new_generator();
            } else if !self.failure_seeds.is_empty() && self.rnd.next_below(2) == 0 {
                let index = self.rnd.next_below(self.failure_seeds.len() as i32) as usize;
                self.generator = Self::mutate_generator(&self.failure_seeds[index]);
            } else {
                let index = if self.rnd.next_below(4) != 0 {
                    self.saved.len() - 1
                } else {
                    self.rnd.next_below(self.saved.len() as i32) as usize
                };
                self.generator = Self::mutate_generator(&self.saved[index].generator);
            }
            return;
        }
        if self.fresh_percent > 0 && self.rnd.next_double() < f64::from(self.fresh_percent) / 100.0 {
            self.generator = self.new_generator();
            return;
        }
        if self.saved.is_empty() {
            self.generator = self.new_generator();
            return;
        }
        if self.current_parent.is_none() && !self.should_explore_new {
            let first = self.saved[0].clone();
            self.visited.insert(first.id);
            self.pending_mutations = first.mutation_budget;
            self.current_parent = Some(first);
        }
        if self.pending_mutations == 0 {
            self.should_explore_new = false;
            let next = self.saved.iter().find(|g| !self.visited.contains(&g.id)).cloned();
            match next {
                Some(g) => {
                    self.visited.insert(g.id);
                    self.pending_mutations = g.mutation_budget;
                    self.current_parent = Some(g);
                }
                None => {
                    if self.rnd.next_double() < 0.5 {
                        self.visited.clear();
                        let first = self.saved[0].clone();
                        self.visited.insert(first.id);
                        self.pending_mutations = first.mutation_budget;
                        self.current_parent = Some(first);
                    } else {
                        self.should_explore_new = true;
                        self.current_parent = None;
                        self.pending_mutations = DEFAULT_MUTATION_BUDGET;
                    }
                }
            }
        }
        self.generator = if self.should_explore_new {
            self.new_generator()
        } else {
            Self::mutate_generator(&self.current_parent.as_ref().unwrap().generator)
        };
        self.pending_mutations -= 1;
    }
}

impl Strategy for FeedbackGuidedStrategy {
    fn wants_state_frontiers(&self) -> bool {
        self.frontiers.is_some() && self.state_frontiers
    }
    fn wants_frontiers(&self) -> bool {
        self.frontiers.is_some()
    }
    fn observe_frontier(&mut self, machine: u32, state: u32, event: u32) {
        let Some(frontiers) = &mut self.frontiers else { return };
        let key = (machine, state, event);
        if !frontiers.counts.contains_key(&key) && frontiers.counts.len() == 256 {
            return;
        }
        let count = frontiers.counts.entry(key).or_default();
        *count = count.saturating_add(1);
        // Fine-grained early stages, sparse milestones for long-running loops.
        if *count > 64 && !count.is_power_of_two() {
            return;
        }
        let old = frontiers.saved.iter().position(|record| record.key == key);
        if old.is_some_and(|index| frontiers.saved[index].count >= *count)
            || (old.is_none() && frontiers.saved.len() == 256)
        {
            return;
        }
        let generator = self.generator.borrow();
        let Some((scheduler, scheduling_choices)) = generator.scheduler.checkpoint(false) else { return };
        let input = generator.input.prefix(false);
        let choices = scheduling_choices + input.choice_count();
        let retained = frontiers.retained - old.map_or(0, |index| frontiers.saved[index].choices) + choices;
        if retained > 262_144 {
            return;
        }
        let record = FrontierRecord {
            key,
            count: *count,
            choices,
            generator: Rc::new(RefCell::new(GeneratorState { input, scheduler })),
            exhausted: false,
        };
        if let Some(index) = old {
            frontiers.saved[index] = record;
        } else {
            frontiers.saved.push(record);
        }
        frontiers.retained = retained;
    }
    fn next_operation(&mut self, current: u64, ops: &[OpInfo]) -> Option<u64> {
        let r = self.generator.borrow_mut().scheduler.next_operation(current, ops);
        self.steps += 1;
        r
    }
    fn next_bool(&mut self, max: i32) -> bool {
        let r = self.generator.borrow_mut().input.next_below(max) == 0;
        self.steps += 1;
        r
    }
    fn next_int(&mut self, max: i32) -> i32 {
        let r = self.generator.borrow_mut().input.next_below(max);
        self.steps += 1;
        r
    }
    fn prepare_next_iteration(&mut self) -> bool {
        self.steps = 0;
        self.prepare_next_input();
        if let Some(guide) = &self.choice_guide {
            guide.borrow_mut().prefix = 0;
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
        true
    }
    fn description(&self) -> String {
        "feedback".into()
    }
    fn wants_feedback(&self) -> bool {
        true
    }
    fn wants_minhash(&self) -> bool {
        !self.code_guidance
    }
    fn observe_progress(&mut self, new_coverage: usize, bug: Option<&str>) {
        if !self.code_guidance {
            return;
        }
        let new_failure = bug.is_some_and(|bug| self.failures.len() < 64 && self.failures.insert(failure_class(bug)));
        // Reuse failing data prefixes to vary inputs near a failure site. Pure
        // interleavings already use the branch/event corpus; repeatedly replaying a
        // failed interleaving can crowd out other delivery orders.
        if new_failure && !self.generator.borrow().input.ints.data.is_empty() {
            self.failure_seeds.push(self.generator.clone());
        }
        self.stalled = self.stalled.saturating_add(1);
        if new_coverage == 0 && !new_failure {
            return;
        }
        self.stalled = 0;
        // Only productive seeds enter the corpus; cap memory for long checks.
        if self.saved.len() == 256 {
            self.saved.remove(0);
        }
        self.saved.push(Rc::new(GeneratorRecord {
            id: self.next_record_id,
            priority: 1.,
            mutation_budget: 1,
            generator: self.generator.clone(),
            minhash: Vec::new(),
        }));
        self.next_record_id += 1;
    }
    fn observe_timeline(&mut self, timeline: &str, minhash: &[i32], reached: &[(String, usize)], satisfied: &[String]) {
        if self.code_guidance {
            let mut progress = false;
            for token in timeline.split(';').filter(|s| !s.is_empty()) {
                if self.native_features.len() < 100_000 {
                    progress |= self.native_features.insert(token.to_string());
                }
            }
            for (name, count) in reached {
                let best = self.suite_best_reached.entry(name.clone()).or_default();
                if *count > *best {
                    *best = *count;
                    progress = true;
                }
            }
            for name in satisfied {
                progress |= self.suite_satisfied.insert(name.clone());
            }
            if progress {
                self.observe_progress(1, None);
            }
            return;
        }
        let diversity = self.compute_diversity(timeline, minhash);
        if diversity <= 0.0 {
            return;
        }
        // ScenarioSteering.NoveltyCompliance: new scenario progress boosts the priority
        let mut compliance = 0.0;
        for s in satisfied {
            if self.suite_satisfied.insert(s.clone()) {
                compliance = 1.0;
            }
        }
        for (s, n) in reached {
            if self.suite_satisfied.contains(s) {
                continue;
            }
            let best = self.suite_best_reached.get(s).copied().unwrap_or(0);
            if *n > best {
                self.suite_best_reached.insert(s.clone(), *n);
                compliance = 1.0;
            }
        }
        let priority = diversity * (1.0 + compliance);
        let budget = ((diversity * DEFAULT_MUTATION_BUDGET as f64).round_ties_even() as i32).max(1);
        let record = Rc::new(GeneratorRecord {
            id: self.next_record_id,
            priority,
            mutation_budget: budget,
            generator: self.generator.clone(),
            minhash: minhash.to_vec(),
        });
        self.next_record_id += 1;
        let mut index = 0;
        while index < self.saved.len() && priority < self.saved[index].priority {
            index += 1;
        }
        self.saved.insert(index, record);
    }
}

/// `TimelineObserver.GetTimelineMinhash` over a set of timeline tokens.
pub fn timeline_minhash<'a>(tokens: impl Iterator<Item = &'a str>) -> Vec<i32> {
    thread_local! {
        static COEFFICIENTS: Vec<(i32, i32)> = {
            let mut r = DotNetRandom::new(0);
            (0..50).map(|_| { let a = r.next(); let b = r.next(); (a, b) }).collect()
        };
    }
    let hashes: Vec<i32> = tokens.map(stable_hash).collect();
    COEFFICIENTS.with(|cs| {
        cs.iter()
            .map(|(a, b)| hashes.iter().map(|v| a.wrapping_mul(*v).wrapping_add(*b)).fold(i32::MAX, i32::min))
            .collect()
    })
}

/// FNV-1a over UTF-8 (`TimelineObserver.StableHash`).
pub fn stable_hash(token: &str) -> i32 {
    let mut h: u32 = 2166136261;
    for b in token.bytes() {
        h = (h ^ b as u32).wrapping_mul(16777619);
    }
    h as i32
}

#[cfg(test)]
mod mutation_tests {
    use super::*;

    #[test]
    fn pos_checkpoints_reconstruct_priorities_by_replaying_consumed_random_choices() {
        let mut scheduler = PosScheduler::new(Source::Controlled(ControlledRandom::new(7)));
        let ops: Vec<_> =
            (1..=3).map(|id| OpInfo { id, enabled: true, kind: SchedKind::Send, receiver: Some(9) }).collect();
        let expected: Vec<_> = (0..8).map(|_| scheduler.next_operation(99, &ops)).collect();
        let (saved, count) = scheduler.checkpoint(false).unwrap();
        assert!(count > 0);
        let (mut replay, replay_count) = saved.checkpoint(true).unwrap();
        assert_eq!(replay_count, count);
        assert_eq!((0..8).map(|_| replay.next_operation(99, &ops)).collect::<Vec<_>>(), expected);
    }

    #[test]
    fn checkpoints_drop_unconsumed_mutation_suffixes_and_rewind_only_for_replay() {
        let mut random = ControlledRandom::new(7);
        let values: Vec<_> = (0..8).map(|_| random.next_below(100)).collect();
        random.ints.pos = 3;
        let saved = random.prefix(false);
        assert_eq!(saved.ints.pos, 3);
        assert_eq!(saved.ints.data.len(), 3);
        let mut replay = saved.prefix(true);
        assert_eq!((0..3).map(|_| replay.next_below(100)).collect::<Vec<_>>(), values[..3]);
        assert_eq!(random.ints.data.len(), 8);
    }

    #[test]
    fn frontier_records_are_bounded_and_terminal_prefixes_stop_being_sampled() {
        let source = Source::Controlled(ControlledRandom::new(7));
        let mut strategy = FeedbackGuidedStrategy::new(100, 7, Box::new(RandomScheduler::new(source)))
            .with_code_guidance(true)
            .with_frontier_guidance();
        strategy.next_int(2);
        for event in 0..1000 {
            strategy.observe_frontier(0, 0, event);
        }
        assert_eq!(strategy.frontiers.as_ref().unwrap().saved.len(), 256);
        assert!(strategy.frontiers.as_ref().unwrap().retained <= 262_144);
        for _ in 0..10000 {
            strategy.prepare_next_iteration();
            strategy.next_int(2);
        }
        assert!(strategy.frontiers.as_ref().unwrap().saved.iter().all(|record| record.exhausted));
    }

    #[test]
    fn choice_guide_explores_repeated_contexts_and_bounds_memory() {
        let mut guide = ChoiceGuide::new(7);
        for _ in 0..128 {
            guide.prefix = 0;
            guide.choose(1, 3, 0, true);
        }
        assert!(guide.visits[&(0, 1, 3)].iter().all(|count| *count > 0));
        for prefix in 1..100 {
            guide.prefix = prefix;
            assert!((0..10_000).contains(&guide.choose(1, 10_000, 17, true)));
        }
        assert!(guide.counters <= 262_144);
        assert_eq!(guide.choose(1, 0x7fffffff, 17, true), 17);
    }

    #[test]
    fn guided_random_records_executed_choices_and_keeps_guide_when_mutated() {
        let guide = Rc::new(RefCell::new(ChoiceGuide::new(7)));
        let mut source = ControlledRandom::new(1);
        source.guide = Some((guide.clone(), 1));
        for _ in 0..100 {
            let value = source.next_below(13);
            assert_eq!(source.ints.data[source.ints.pos - 1] % 13, value);
        }
        for next in [source.new_like(), source.mutate()] {
            assert!(Rc::ptr_eq(&next.guide.unwrap().0, &guide));
        }
        // Corpus replay preserves a productive prefix even as visit counts grow.
        let recorded = source.ints.data.clone();
        for _ in 0..20 {
            source.ints.pos = 0;
            guide.borrow_mut().prefix = 0;
            for expected in &recorded {
                assert_eq!(source.next_below(13), expected % 13);
            }
        }
    }

    #[test]
    fn code_novelty_retains_seeds_even_with_identical_timelines_and_bounds_the_corpus() {
        let source = Source::Controlled(ControlledRandom::new(7));
        let mut strategy =
            FeedbackGuidedStrategy::new(100, 7, Box::new(RandomScheduler::new(source))).with_code_guidance(true);
        for _ in 0..300 {
            strategy.next_int(100);
            strategy.observe_progress(1, None);
            strategy.observe_timeline("same", &[], &[], &[]);
            strategy.prepare_next_iteration();
        }
        assert_eq!(strategy.saved.len(), 256);
        assert!(!strategy.wants_minhash());
        for _ in 0..128 {
            strategy.observe_progress(0, None);
        }
        let mut fresh = 0;
        for _ in 0..100 {
            strategy.prepare_next_iteration();
            fresh += usize::from(strategy.generator.borrow().input.ints.data.is_empty());
        }
        assert!(fresh > 50 && fresh < 100, "stalls favor fresh sampling but keep exploiting: {fresh}");
        assert_eq!(strategy.saved.len(), 256);
    }

    #[test]
    fn failure_data_seeds_are_distinct_by_site_and_do_not_need_new_coverage() {
        let source = Source::Controlled(ControlledRandom::new(7));
        let mut strategy =
            FeedbackGuidedStrategy::new(100, 7, Box::new(RandomScheduler::new(source))).with_code_guidance(true);
        strategy.next_int(100);
        strategy.observe_progress(0, Some("Assertion Failed: model.p:3:2 value 123"));
        strategy.observe_progress(0, Some("Assertion Failed: model.p:3:2 value 456"));
        assert_eq!(strategy.failure_seeds.len(), 1);
        assert_eq!(strategy.saved.len(), 1);
        strategy.observe_progress(0, Some("Assertion Failed: model.p:4:2 value 456"));
        assert_eq!(strategy.failure_seeds.len(), 2);
    }

    #[test]
    fn geometric_mutation_sizes_are_positive_even_at_mean_one() {
        for random in [0.0, 0.1, 0.5, 0.999999] {
            assert_eq!(sample_geometric(1.0, random), 1);
            assert!(sample_geometric(0.2, random) >= 1);
            assert_eq!(sample_geometric_upstream(1.0, random), 0);
        }
        assert_eq!(sample_geometric(0.5, 0.75), 2);
    }

    #[test]
    fn mutating_recorded_choices_changes_short_and_long_inputs_without_changing_parent() {
        for count in [1, 3, 30] {
            let mut source = ControlledRandom::new(7);
            for _ in 0..count {
                source.next_below(i32::MAX);
            }
            let original = source.ints.data.clone();
            let mutation = source.mutate();
            assert_ne!(mutation.ints.data, original, "{count} choices");
            assert_eq!(source.ints.data, original);
            assert_eq!(mutation.ints.pos, 0);
            assert!(mutation.doubles.data.is_empty());
        }
        // No recorded input is also safe to mutate and subsequently extend.
        let mut empty = ControlledRandom::new(7).mutate();
        assert!((0..10).contains(&empty.next_below(10)));
    }

    #[test]
    fn legacy_mutations_and_fresh_generators_keep_the_upstream_mode() {
        let mut source = ControlledRandom::new(7).with_legacy_mutations(true);
        for _ in 0..30 {
            source.next_below(i32::MAX);
        }
        let mutation = source.mutate();
        assert_eq!(mutation.ints.data, source.ints.data);
        assert!(mutation.legacy_mutations);
        assert!(source.new_like().legacy_mutations);
    }

    #[test]
    fn fresh_exploration_keeps_the_corpus_and_resumes_mutations() {
        let source = Source::Controlled(ControlledRandom::new(7));
        let mut strategy =
            FeedbackGuidedStrategy::new(100, 7, Box::new(RandomScheduler::new(source))).with_fresh_exploration(100);
        strategy.next_int(1000);
        strategy.observe_timeline("timeline", &[1, 2], &[], &[]);
        assert_eq!(strategy.saved.len(), 1);
        let saved = strategy.saved[0].clone();
        strategy.prepare_next_iteration();
        assert!(strategy.generator.borrow().input.ints.data.is_empty());
        assert!(Rc::ptr_eq(&strategy.saved[0], &saved));
        strategy = strategy.with_fresh_exploration(0);
        strategy.prepare_next_iteration();
        assert_eq!(strategy.current_parent.as_ref().unwrap().id, saved.id);
        assert!(!strategy.generator.borrow().input.ints.data.is_empty());
    }
}
