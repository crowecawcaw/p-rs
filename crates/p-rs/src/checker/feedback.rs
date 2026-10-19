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
    fn mutate(&self, mean_count: i64, mean_size: i64) -> Self {
        let n = self.data.len() as i64;
        let mean_count = (n / 3).min(mean_count).max(1);
        let mean_size = (n / 3).min(mean_size).max(1);
        let mut out =
            RandomChoices { random: self.random.clone(), pos: 0, data: self.data.clone(), generate: self.generate };
        let r = self.random.borrow_mut().next_double();
        let mut mutations = sample_geometric((1.0f32 / mean_count as f32) as f64, r);
        while mutations > 0 {
            mutations -= 1;
            let offset = self.random.borrow_mut().next_below(out.data.len() as i32) as usize;
            let r = self.random.borrow_mut().next_double();
            let size = sample_geometric((1.0f32 / mean_size as f32) as f64, r);
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

/// `Utils.SampleGeometric`.
fn sample_geometric(p: f64, random: f64) -> i32 {
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
        }
    }

    pub fn next_below(&mut self, max: i32) -> i32 {
        if max == 0 {
            return 0;
        }
        self.ints.next() % max
    }

    /// Fresh choices drawing from the same underlying generator.
    pub fn new_like(&self) -> ControlledRandom {
        ControlledRandom::from_random(self.ints.random.clone())
    }

    pub fn mutate(&self) -> ControlledRandom {
        ControlledRandom { ints: self.ints.mutate(5, 1), doubles: self.doubles.mutate(5, 1) }
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
}

impl FeedbackGuidedStrategy {
    /// `max_steps` is the unfair step bound; `seed` seeds the input generator, the
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
        }
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

    fn new_generator(&self) -> Generator {
        let g = self.generator.borrow();
        Rc::new(RefCell::new(GeneratorState { input: g.input.new_like(), scheduler: g.scheduler.new_like() }))
    }

    fn mutate_generator(prev: &Generator) -> Generator {
        let g = prev.borrow();
        Rc::new(RefCell::new(GeneratorState { input: g.input.mutate(), scheduler: g.scheduler.mutate() }))
    }

    fn prepare_next_input(&mut self) {
        self.generator.borrow_mut().scheduler.prepare_next_iteration();
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
    fn observe_timeline(&mut self, timeline: &str, minhash: &[i32], reached: &[(String, usize)], satisfied: &[String]) {
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
