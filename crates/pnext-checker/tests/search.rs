//! The exhaustive search (`dfs`, `random-dfs`) and the Q-learning strategy (`rl`).

use pnext_checker::engine::{CheckResult, Checker, CheckerOptions, Output};
use pnext_checker::strategy::ScheduleTrace;

struct Quiet;
impl Output for Quiet {
    fn line(&mut self, _: &str) {}
    fn error(&mut self, _: &str) {}
}

fn check(src: &str, opts: &CheckerOptions) -> CheckResult {
    let mut sm = pnext_syntax::SourceMap::new();
    sm.add("model.p", src.to_string());
    let a = pnext_sema::compile(&sm, &pnext_sema::Options::default()).unwrap_or_else(|_| panic!("does not compile"));
    let tc = a.program.test_cases().into_iter().find(|t| t.name == "tcMain").unwrap();
    Checker::new(&a.program).check(&tc, opts, &mut Quiet).unwrap()
}

fn opts(strategy: &str, iterations: usize) -> CheckerOptions {
    CheckerOptions { strategy: strategy.into(), iterations, seed: 7, ..Default::default() }
}

/// Three clients send to a server; `ORDER` decides which delivery order fails.
fn clients(check: &str) -> String {
    format!(
        "event eHello: int;
         machine Server {{
           var got: seq[int];
           start state Init {{
             on eHello do (x: int) {{
               got += (sizeof(got), x);
               if (sizeof(got) == 3) {{ {check} }}
             }}
           }}
         }}
         machine Client {{
           start state Init {{
             entry (p: (s: Server, x: int)) {{ send p.s, eHello, p.x; }}
           }}
         }}
         machine Main {{
           start state Init {{
             entry {{
               var s: Server;
               s = new Server();
               new Client((s = s, x = 1));
               new Client((s = s, x = 2));
               new Client((s = s, x = 3));
             }}
           }}
         }}
         test tcMain [main=Main]: {{ Main, Server, Client }};"
    )
}

#[test]
fn dfs_explores_everything_of_a_correct_test() {
    let r = check(&clients("assert sizeof(got) == 3;"), &opts("dfs", 0));
    assert_eq!(r.report.num_bugs, 0);
    let x = r.exhaustive.expect("search stats");
    assert!(x.finished && x.depth_bound_hits == 0, "{:?}", x);
    assert!(r.report.explored_unfair_schedules > 6, "{:?}", r.report.explored_unfair_schedules);
    // caching: far fewer schedules than interleavings (every pruned one ends early)
    assert!(x.pruned_schedules > 0, "{:?}", x);
    // the random order explores the same state space
    let rr = check(&clients("assert sizeof(got) == 3;"), &opts("random-dfs", 0));
    let y = rr.exhaustive.unwrap();
    assert!(y.finished);
    assert_eq!(x.distinct_states, y.distinct_states);
}

#[test]
fn dfs_finds_a_specific_interleaving_and_its_schedule_replays() {
    let src = clients("assert !(got[0] == 3 && got[1] == 2 && got[2] == 1), \"reversed\";");
    for strategy in ["dfs", "random-dfs"] {
        let r = check(&src, &opts(strategy, 0));
        assert_eq!(r.report.num_bugs, 1, "{strategy}");
        assert!(r.report.bug_reports[0].contains("reversed"));
        assert!(!r.exhaustive.unwrap().finished);
        let trace = ScheduleTrace::parse(r.schedule_trace.as_ref().unwrap()).unwrap();
        let replay = check(&src, &CheckerOptions { replay: Some(trace), ..opts("replay", 1) });
        assert_eq!(replay.report.num_bugs, 1, "{strategy}");
        assert!(replay.replay_error.is_none());
    }
}

#[test]
fn dfs_branches_on_nondeterministic_choices() {
    let src = "machine Main {
                 start state Init {
                   entry {
                     var i: int;
                     var n: int;
                     while (i < 4) { if ($) { n = n + 1; } i = i + 1; }
                     assert n + choose(3) != 6, \"all\";
                   }
                 }
               }
               test tcMain [main=Main]: { Main };";
    let r = check(src, &opts("dfs", 0));
    assert_eq!(r.report.num_bugs, 1);
    let ok = check(&src.replace("!= 6", "!= 7"), &opts("dfs", 0));
    assert_eq!(ok.report.num_bugs, 0);
    assert!(ok.exhaustive.unwrap().finished);
}

#[test]
fn dfs_reports_a_hot_monitor_at_the_end() {
    let src = "event eReq; event eAck;
               spec Live observes eReq, eAck {
                 start cold state Idle { on eReq goto Waiting; }
                 hot state Waiting { on eAck goto Idle; }
               }
               machine Main {
                 start state Init {
                   entry { announce eReq; if ($) { announce eAck; } }
                 }
               }
               test tcMain [main=Main]: assert Live in { Main };";
    let r = check(src, &opts("dfs", 0));
    assert_eq!(r.report.num_bugs, 1);
    assert!(r.report.bug_reports[0].contains("hot state 'Waiting'"), "{}", r.report.bug_reports[0]);
}

#[test]
fn dfs_reports_when_the_depth_bound_cut_the_search() {
    let src = "event eTick;
               machine Main {
                 var n: int;
                 start state Init {
                   entry { send this, eTick; }
                   on eTick do { n = n + 1; send this, eTick; }
                 }
               }
               test tcMain [main=Main]: { Main };";
    let r = check(src, &CheckerOptions { max_unfair_steps: 50, ..opts("dfs", 0) });
    assert_eq!(r.report.num_bugs, 0);
    let x = r.exhaustive.unwrap();
    assert!(x.finished);
    assert_eq!(x.depth_bound_hits, 1);
}

#[test]
fn rl_is_reproducible_finds_bugs_and_replays() {
    let src = clients("assert !(got[0] == 3 && got[1] == 2 && got[2] == 1), \"reversed\";");
    let a = check(&src, &opts("rl", 1000));
    let b = check(&src, &opts("rl", 1000));
    assert_eq!(a.report.num_bugs, 1);
    assert_eq!(a.schedule_trace, b.schedule_trace);
    assert_eq!(a.report.explored_fair_schedules, b.report.explored_fair_schedules);
    assert!(a.strategy_description.starts_with("rl[seed '7']"));
    let trace = ScheduleTrace::parse(a.schedule_trace.as_ref().unwrap()).unwrap();
    let replay = check(&src, &CheckerOptions { replay: Some(trace), ..opts("replay", 1) });
    assert_eq!(replay.report.num_bugs, 1);
    assert!(replay.replay_error.is_none());
    // a correct test stays clean
    let ok = check(&clients("assert sizeof(got) == 3;"), &opts("rl", 200));
    assert_eq!(ok.report.num_bugs, 0);
    assert_eq!(ok.report.explored_fair_schedules, 200);
}
