//! Time tracking (`design/time/PROPOSAL.md`): causal clocks, latency profiles,
//! `--timed` scheduling and schedule headers.

use std::path::Path;
use std::rc::Rc;

use pnext_checker::engine::{CheckResult, Checker, CheckerOptions};
use pnext_checker::strategy::ScheduleTrace;
use pnext_checker::time::ProfileSet;

const TIME_DECLS: &str = "
fun Now(): int;
fun NowBest(): int;
fun Elapse(lo: int, hi: int);
fun ElapseCost(name: string);
fun Mark(): int;
fun Elapsed(m: int): int;
fun ElapsedBest(m: int): int;
";

fn compile(files: &[(String, String)]) -> pnext_sema::Analysis {
    let mut sm = pnext_syntax::SourceMap::new();
    for (name, text) in files {
        sm.add(name.clone(), text.clone());
    }
    pnext_sema::compile(&sm, &pnext_sema::Options::default()).unwrap_or_else(|_| panic!("compile"))
}

fn check(a: &pnext_sema::Analysis, test: &str, opts: &CheckerOptions) -> Result<CheckResult, String> {
    let tc = a.program.test_cases().into_iter().find(|t| t.name == test).unwrap();
    Checker::new(&a.program).check(&tc, opts, &mut Vec::<String>::new())
}

fn retry_budget(edit: impl Fn(String) -> String) -> (pnext_sema::Analysis, Rc<ProfileSet>) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../design/time/upstream-friendly/RetryBudget");
    let mut files = Vec::new();
    for sub in ["PSrc", "PSpec", "PTst"] {
        let mut paths: Vec<_> = std::fs::read_dir(dir.join(sub)).unwrap().map(|e| e.unwrap().path()).collect();
        paths.sort();
        for p in paths.into_iter().filter(|p| p.extension().is_some_and(|e| e == "p")) {
            files.push((p.display().to_string(), edit(std::fs::read_to_string(&p).unwrap())));
        }
    }
    let profiles = ProfileSet::parse(&std::fs::read_to_string(dir.join("PTst/latency.toml")).unwrap()).unwrap();
    (compile(&files), Rc::new(profiles))
}

#[test]
fn retry_budget_finds_the_give_up_path_and_accepts_the_fix() {
    let (a, profiles) = retry_budget(|s| s);
    for timed in [false, true] {
        let opts = CheckerOptions { iterations: 500, seed: 1, latency: Some(profiles.clone()), timed, ..Default::default() };
        let r = check(&a, "tcRetry", &opts).unwrap();
        assert_eq!(r.report.num_bugs, 1, "timed={}", timed);
        let bug = &r.report.bug_reports[0];
        assert!(bug.contains("ok = False") && bug.contains("1050000us"), "{}", bug);
        let log = r.readable_trace.unwrap();
        assert!(log.contains("<TimeLog> 'Client(3)' clock is 1050000 us."), "{}", log);
        assert!(r.schedule_trace.unwrap().contains(&format!("--pnext-time:timed={};profile=same-az;hash=", timed as u8)));
    }

    let (fixed, _) = retry_budget(|s| s.replace("ms(300)", "ms(250)"));
    for timed in [false, true] {
        let opts = CheckerOptions { iterations: 500, seed: 1, latency: Some(profiles.clone()), timed, ..Default::default() };
        let r = check(&fixed, "tcRetry", &opts).unwrap();
        assert_eq!(r.report.num_bugs, 0, "{:?}", r.report.bug_reports);
    }
}

#[test]
fn replay_needs_the_same_time_settings() {
    let (a, profiles) = retry_budget(|s| s);
    let opts = CheckerOptions { iterations: 500, seed: 2, latency: Some(profiles.clone()), ..Default::default() };
    let r = check(&a, "tcRetry", &opts).unwrap();
    let trace = ScheduleTrace::parse(&r.schedule_trace.unwrap()).unwrap();

    let replay = |latency_profile: Option<&str>, timed: bool| CheckerOptions {
        strategy: "replay".into(),
        replay: Some(trace.clone()),
        latency: Some(profiles.clone()),
        latency_profile: latency_profile.map(String::from),
        timed,
        ..Default::default()
    };
    let again = check(&a, "tcRetry", &replay(None, false)).unwrap();
    assert_eq!(again.report.bug_reports, r.report.bug_reports);
    assert!(again.replay_error.is_none());

    let err = check(&a, "tcRetry", &replay(Some("cross-region"), false)).unwrap_err();
    assert!(err.contains("profile=cross-region"), "{}", err);
    assert!(check(&a, "tcRetry", &replay(None, true)).is_err());
}

const RACE: &str = "
event eReq: Client;
event eResp;
event eTimeout;

machine Timer {
  start state Arming {
    entry (c: Client) { Elapse(100000, 100000); send c, eTimeout; }
  }
}

machine Server {
  start state Serving {
    on eReq do (c: Client) { Elapse(1000, 5000); send c, eResp; }
  }
}

machine Client {
  start state Init {
    entry (s: Server) { send s, eReq, this; new Timer(this); }
    on eResp goto Done;
    on eTimeout do { assert false, format(\"timed out at {0}\", Now()); }
  }
  state Done { ignore eTimeout; }
}

machine Main {
  start state Init { entry { new Client(new Server()); } }
}

test tcRace [main=Main]: { Main, Client, Server, Timer };
";

#[test]
fn timed_scheduling_drops_impossible_orders() {
    let a = compile(&[("decls.p".into(), TIME_DECLS.into()), ("race.p".into(), RACE.into())]);
    let profiles = Rc::new(ProfileSet::parse("[profile.lan]\ndefault = \"1ms..2ms\"").unwrap());
    let opts = |timed| CheckerOptions {
        iterations: 300,
        seed: 7,
        latency: Some(profiles.clone()),
        latency_profile: Some("lan".into()),
        timed,
        ..Default::default()
    };
    // untimed, the 100ms timeout may be handled before the ~10ms reply
    let r = check(&a, "tcRace", &opts(false)).unwrap();
    assert_eq!(r.report.num_bugs, 1);
    assert!(r.report.bug_reports[0].contains("timed out at 102000"), "{}", r.report.bug_reports[0]);
    let r = check(&a, "tcRace", &opts(true)).unwrap();
    assert_eq!(r.report.num_bugs, 0, "{:?}", r.report.bug_reports);
}

const STOPWATCH: &str = "
machine Main {
  start state Init {
    entry {
      var m: int;
      var n: int;
      m = Mark();
      Elapse(10, 20);
      n = Mark();
      Elapse(5, 5);
      assert Elapsed(m) == 25 && ElapsedBest(m) == 15, format(\"m: {0} {1}\", ElapsedBest(m), Elapsed(m));
      // shared history cancels: n - m is exactly the second draw
      assert Elapsed(n) == 5 && ElapsedBest(n) == 5, format(\"n: {0} {1}\", ElapsedBest(n), Elapsed(n));
      assert Now() == 25 && NowBest() == 15;
      ElapseCost(\"Query\");
      assert Elapsed(n) == 5 + 8000 && ElapsedBest(n) == 5 + 2000;
      ElapseCost(\"Unpriced\");
      assert Now() == 8025;
    }
  }
}

test tcStopwatch [main=Main]: { Main };
";

#[test]
fn stopwatch_functions() {
    let a = compile(&[("decls.p".into(), TIME_DECLS.into()), ("sw.p".into(), STOPWATCH.into())]);
    let profiles = Rc::new(ProfileSet::parse("[profile.p]\n\"cost.Query\" = \"2ms..8ms\"\n[tests]\ntcStopwatch = \"p\"").unwrap());
    let opts = CheckerOptions { latency: Some(profiles), ..Default::default() };
    let r = check(&a, "tcStopwatch", &opts).unwrap();
    assert_eq!(r.report.num_bugs, 0, "{:?}", r.report.bug_reports);
}

#[test]
fn bad_calls_and_names_are_reported() {
    let bad = "machine Main { start state Init { entry { Elapse(5, 1); } } }\ntest tcBad [main=Main]: { Main };";
    let a = compile(&[("decls.p".into(), TIME_DECLS.into()), ("bad.p".into(), bad.into())]);
    let r = check(&a, "tcBad", &CheckerOptions::default()).unwrap();
    assert!(r.report.bug_reports[0].contains("expected 0 <= lo <= hi"), "{:?}", r.report.bug_reports);

    let typo = Rc::new(ProfileSet::parse("[profile.p]\neRequest = \"1ms\"").unwrap());
    let opts = CheckerOptions { latency: Some(typo), latency_profile: Some("p".into()), ..Default::default() };
    let err = check(&a, "tcBad", &opts).unwrap_err();
    assert!(err.contains("unknown event 'eRequest'"), "{}", err);
}

#[test]
fn programs_without_time_are_unchanged() {
    let plain = "
event e;
machine Main {
  start state Init { entry { send this, e; } on e do { assert false; } }
}
test tcPlain [main=Main]: { Main };";
    let a = compile(&[("plain.p".into(), plain.into())]);
    let r = check(&a, "tcPlain", &CheckerOptions::default()).unwrap();
    assert_eq!(r.report.num_bugs, 1);
    assert!(!r.readable_trace.unwrap().contains("<TimeLog>"));
    assert!(!r.schedule_trace.unwrap().contains("--pnext-time"));
}
