//! Throughput of each scheduling strategy on the tutorial workloads of `scripts/bench.py`.
//!
//! Runs a fixed number of schedules per strategy with a fixed seed, on one thread, and
//! prints one JSON line per run: time, schedules and schedules per second. Use it to see
//! what a strategy's bookkeeping costs on top of executing the model.
//!
//! Usage: strategy_bench [--strategies random,feedback,...] [--scale F] [--filter NAME] ROOT
//!
//! ROOT is tests/upstream/Tutorial. `feedback-plain` is native feedback without progress
//! checkpoints; `feedback` is the native default (checkpoints on).
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

use snappea::checker::engine::{Checker, CheckerOptions, Output};
use snappea::checker::strategy::StrategyKind;

struct Count(usize);
impl Output for Count {
    fn line(&mut self, _: &str) {}
    fn error(&mut self, _: &str) {}
    fn schedule_finished(&mut self, n: usize, _: Option<&str>) {
        self.0 = n;
    }
}

const WORKLOADS: [(&str, &str, usize); 6] = [
    ("1_ClientServer", "tcMultipleClients", 1500),
    (
        "2_TwoPhaseCommit",
        "tcParametricTests___pNumClients_3__pNumParticipants_4__pNumTransPerClient_2__pFailParticipants_0",
        1500,
    ),
    ("3_EspressoMachine", "tcResourceConstraints___waterLevel_50__beanLevel_50", 5000),
    ("4_FailureDetector", "tcTest_BalancedLoad___numNodes_5__numClients_2", 2000),
    ("5_Paxos", "testBasicPaxos3on5", 9000),
    ("6_Raft", "oneClientFiveServersReliable", 120),
];

fn p_files(path: &Path, out: &mut Vec<PathBuf>) {
    if path.is_file() {
        if path.extension().is_some_and(|x| x == "p") {
            out.push(path.to_path_buf());
        }
        return;
    }
    let Ok(rd) = std::fs::read_dir(path) else { return };
    let mut entries: Vec<_> = rd.map(|e| e.unwrap().path()).collect();
    entries.sort();
    for e in entries {
        p_files(&e, out);
    }
}

fn project_files(pproj: &Path, out: &mut Vec<PathBuf>) {
    let text = std::fs::read_to_string(pproj).unwrap();
    let dir = pproj.parent().unwrap();
    let tags = |tag: &str| -> Vec<String> {
        let (open, close) = (format!("<{tag}>"), format!("</{tag}>"));
        text.split(open.as_str()).skip(1).map(|p| p.split(close.as_str()).next().unwrap().trim().to_string()).collect()
    };
    for inc in tags("IncludeProject") {
        project_files(&dir.join(inc), out);
    }
    for entry in tags("PFile") {
        p_files(&dir.join(entry), out);
    }
}

fn main() {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut strategies = "random,feedback-plain,feedback,auto,pct,dfs".to_string();
    let mut scale = 1.0f64;
    let mut filter = String::new();
    let mut root = None;
    let mut i = 0;
    while i < raw.len() {
        match raw[i].as_str() {
            "--strategies" => {
                i += 1;
                strategies = raw[i].clone();
            }
            "--scale" => {
                i += 1;
                scale = raw[i].parse().unwrap();
            }
            "--filter" => {
                i += 1;
                filter = raw[i].clone();
            }
            r => root = Some(PathBuf::from(r)),
        }
        i += 1;
    }
    let root = root.expect("usage: strategy_bench [--strategies a,b] [--scale F] [--filter NAME] ROOT");
    for (project, test, schedules) in WORKLOADS {
        if !project.contains(&filter) {
            continue;
        }
        let dir = root.join(project);
        let pproj = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| p.extension().is_some_and(|x| x == "pproj"))
            .unwrap();
        let mut files = Vec::new();
        project_files(&pproj, &mut files);
        let mut seen = HashSet::new();
        files.retain(|f| seen.insert(f.clone()));
        let mut sm = snappea::syntax::SourceMap::new();
        for f in &files {
            sm.add(f.display().to_string(), std::fs::read_to_string(f).unwrap());
        }
        let a = snappea::sema::compile(&sm, &Default::default()).unwrap_or_else(|_| panic!("{project}: compile error"));
        let tc = a.program.test_cases().into_iter().find(|t| t.name == test).unwrap();
        let checker = Checker::with_code_coverage(&a.program);
        let n = ((schedules as f64) * scale).max(1.0) as usize;
        for name in strategies.split(',') {
            let (strategy, frontier) = match name {
                "feedback-plain" => (StrategyKind::Feedback, false),
                "feedback" => (StrategyKind::Feedback, true),
                "auto" => (StrategyKind::Auto, true),
                s => (s.parse().unwrap(), false),
            };
            let opts = CheckerOptions {
                strategy,
                feedback_frontier_guidance: frontier,
                seed: 42,
                iterations: n,
                activity_coverage: true,
                ..Default::default()
            };
            let mut out = Count(0);
            let start = Instant::now();
            let r = checker.check(&tc, &opts, &mut out).unwrap();
            let secs = start.elapsed().as_secs_f64();
            println!(
                "{{\"project\":\"{project}\",\"strategy\":\"{name}\",\"schedules\":{},\"seconds\":{secs:.4},\"schedules_per_s\":{:.1},\"bugs\":{},\"steps\":{},\"max_steps\":{},\"bound_hits\":{}}}",
                out.0,
                out.0 as f64 / secs,
                r.report.num_bugs,
                r.report.total_fair_steps,
                r.report.max_fair_steps.unwrap_or(0),
                r.report.max_fair_steps_hit_in_fair
            );
        }
    }
}
