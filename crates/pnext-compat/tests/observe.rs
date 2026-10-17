//! `p observe` on an in-memory host.

mod support;

use std::path::Path;

use pnext_compat::host::{Host, MemHost};

const SPEC: &str = r#"
type tTxn = (account: int, amount: int);
event eDeposit: tTxn;
event eWithdraw: tTxn;
event eClose: int;
event eUnrelated;

spec NeverOverdrawn observes eDeposit, eWithdraw, eClose {
  var balance: int;
  start state Open {
    on eDeposit do (t: tTxn) { balance = balance + t.amount; }
    on eWithdraw do (t: tTxn) {
      balance = balance - t.amount;
      assert balance >= 0, format("account {0} overdrawn: balance {1}", t.account, balance);
    }
    on eClose goto Closed;
  }
  state Closed {}
}
"#;

const PPROJ: &str = "<Project><ProjectName>Bank</ProjectName><InputFiles><PFile>./PSpec/</PFile></InputFiles></Project>";

fn host() -> MemHost {
    let mut h = MemHost::new("/work");
    h.add_file("Bank.pproj", PPROJ);
    h.add_file("PSpec/Bank.p", SPEC);
    h
}

fn run(h: &mut MemHost, args: &[&str]) -> i32 {
    h.stdout.clear();
    h.stderr.clear();
    let args: Vec<String> = std::iter::once("observe").chain(args.iter().copied()).map(String::from).collect();
    pnext_compat::run(&args, h)
}

fn line(key: &str, ts: i64, event: &str, account: i64, amount: i64) -> String {
    format!(r#"{{"key": "{}", "timestamp": {}, "event": "{}", "payload": {{"account": {}, "amount": {}}}}}"#, key, ts, event, account, amount)
}

fn read(h: &MemHost, p: &str) -> String {
    h.read_to_string(Path::new(p)).unwrap_or_else(|e| panic!("{}: {}", p, e))
}

#[test]
fn clean_logs_pass() {
    let mut h = host();
    let log = [line("a", 1, "eDeposit", 1, 10), line("b", 2, "eDeposit", 2, 5), line("a", 3, "eWithdraw", 1, 10)].join("\n");
    h.add_file("logs/run.log", &log);
    let code = run(&mut h, &["--spec", "NeverOverdrawn", "-l", "logs", "-o", "out"]);
    assert_eq!(code, 0, "{}", h.stdout);
    let expected_head = "Current PObserve configuration:
PObserveConfig {
  outputDir            = out
  inputKind            = TEXT
  pproj                = /work/Bank.pproj
  parserName           = null
  specificationName    = NeverOverdrawn
  replayWindowSize     = 100
  logDelimiter         = \\n
}
-------------------------------------
Success! No bugs found.
-------------------------------------
";
    assert!(h.stdout.starts_with(expected_head), "{}", h.stdout);
    assert!(h.stdout.contains("3                        3                        2                        2"), "{}", h.stdout);
    assert!(h.stdout.ends_with("PObserve run completed with 0 error(s)\nRun details can be found in /work/out\n"), "{}", h.stdout);
    let metrics = read(&h, "out/PObserveMetrics.txt");
    assert!(metrics.contains("Event Type"), "{}", metrics);
    assert!(metrics.contains("eDeposit                        2                     0.000000              0"), "{}", metrics);
}

#[test]
fn violations_are_reported_per_key_with_replay_logs() {
    let mut h = host();
    // out of time order on purpose: each key is sorted before checking
    let log = [
        line("acct-1", 5, "eWithdraw", 1, 30),
        line("acct-1", 1, "eDeposit", 1, 20),
        line("acct-2", 2, "eDeposit", 2, 20),
        "not json".to_string(),
    ]
    .join("\n");
    h.add_file("run.log", &log);
    let code = run(&mut h, &["--spec", "NeverOverdrawn", "-l", "run.log", "-o", "out", "--replaySize", "1"]);
    assert_eq!(code, 2, "{}", h.stdout);
    assert!(h.stdout.contains("Parser Exception::\nException occurred while parsing log line: not json\n"), "{}", h.stdout);
    assert!(
        h.stdout.contains("PObserve Spec Violation::\nAssertion failure: PSpec/Bank.p:14:7 account 1 overdrawn: balance -10\n"),
        "{}",
        h.stdout
    );
    assert!(h.stdout.contains("PObserve run completed with 1 error(s)"), "{}", h.stdout);
    let replay = read(&h, "out/replayEvents_acct-1.log");
    assert_eq!(
        replay,
        "errorKey=acct-1\nerrorTimeStamp=5\nerrorMessage=Assertion failure: PSpec/Bank.p:14:7 account 1 overdrawn: balance -10\nreplayEvents=\n\
         PObserveEvent{partitionKey='acct-1', timestamp=1970-01-01T00:00:00.005000000Z, event=eWithdraw[PTuple_accnt_amnt[account=1, amount=30]], \
         logLine='{\"key\": \"acct-1\", \"timestamp\": 5, \"event\": \"eWithdraw\", \"payload\": {\"account\": 1, \"amount\": 30}}', customPayload=null}\n"
    );
    let parser_error = read(&h, "out/ParserError_0.txt");
    assert!(parser_error.starts_with("errorMessage=pobserve.commons.exceptions.PObserveLogParsingException: not json"));
}

#[test]
fn parser_errors_alone_exit_with_4() {
    let mut h = host();
    h.add_file("run.log", &format!("{}\n{{oops", line("k", 1, "eDeposit", 1, 1)));
    assert_eq!(run(&mut h, &["--spec", "NeverOverdrawn", "-l", "run.log", "-o", "out"]), 4, "{}", h.stdout);
    assert!(h.stdout.contains("PObserve run completed with 0 error(s)"), "{}", h.stdout);
}

#[test]
fn unhandled_events_and_out_of_order_inputs_are_internal_errors() {
    let mut h = host();
    h.add_file("run.log", &[r#"{"key":"k","timestamp":1,"event":"eClose","payload":7}"#.to_string(), line("k", 2, "eDeposit", 1, 1)].join("\n"));
    assert_eq!(run(&mut h, &["--spec", "NeverOverdrawn", "-l", "run.log", "-o", "out"]), 3, "{}", h.stdout);
    assert!(h.stdout.contains("prt.State[Closed] missing event handler for eDeposit\n"), "{}", h.stdout);
    assert!(h.stdout.contains("Failed with an unknown exception:: null\nPlease report this issue to the P team\n"), "{}", h.stdout);
    let unknown = read(&h, "out/UnknownError_0.txt");
    assert!(unknown.starts_with("errorMessage=prt.State prt.State[Closed] has no handler for class Bank.pobserve.PEvents$eDeposit"), "{}", unknown);

    h.add_file("run.log", &[line("k", 2, "eDeposit", 1, 1), line("k", 1, "eDeposit", 1, 1)].join("\n"));
    assert_eq!(run(&mut h, &["--spec", "NeverOverdrawn", "-l", "run.log", "-o", "out2", "--assumeInputsAreSorted"]), 3);
    assert!(h.stdout.contains("PObserve Event Out of Order Exception::\nEncountered an out of order event.\n"), "{}", h.stdout);
    assert!(h.stdout.contains("Timestamp of last processed event: 1970-01-01T00:00:00.002Z\n"), "{}", h.stdout);
}

#[test]
fn keys_filter_and_parser_configuration() {
    let mut h = host();
    let log = [
        r#"{"msg": {"kind": "eWithdraw", "acct": 9, "at": "2024-02-02T17:27:34.309Z", "body": [9, 100]}}"#,
        r#"{"msg": {"kind": "eWithdraw", "acct": 8, "at": "2024-02-02T17:27:35Z", "body": {"account": 8, "amount": 1}}}"#,
    ]
    .join("\n");
    h.add_file("run.log", &log);
    let pc = r#"{"event": "msg.kind", "key": "msg.acct", "timestamp": "msg.at", "payload": "msg.body"}"#;
    let code = run(&mut h, &["--spec", "NeverOverdrawn", "-l", "run.log", "-o", "out", "-pc", pc, "--keys", "9"]);
    assert_eq!(code, 2, "{}", h.stdout);
    let replay = read(&h, "out/replayEvents_9.log");
    assert!(replay.contains("errorTimeStamp=1706894854309\n"), "{}", replay);
    assert!(replay.contains("timestamp=2024-02-02T17:27:34.309000000Z"), "{}", replay);
    assert!(!h.is_file(Path::new("out/replayEvents_8.log")));
}

#[test]
fn json_traces() {
    let mut h = host();
    let trace = r#"[
      {"type": "CreateStateMachine", "details": {"log": "created"}},
      {"type": "SendEvent", "details": {"log": "send 1", "event": "eDeposit", "payload": {"account": 1, "amount": 3}}},
      {"type": "Announce", "details": {"log": "announce 2", "event": "PImplementation.eWithdraw", "payload": {"account": 1, "amount": 4}}}
    ]"#;
    h.add_file("trace.json", trace);
    let code = run(&mut h, &["--spec", "NeverOverdrawn", "-l", "trace.json", "-o", "out", "-ik", "json"]);
    assert_eq!(code, 2, "{}", h.stdout);
    assert!(h.stdout.contains("  inputKind            = JSON\n"));
    assert!(read(&h, "out/replayEvents_.log").contains("logLine='announce 2'"));
}

#[test]
fn command_line_errors() {
    let mut h = host();
    h.add_file("run.log", "");
    let usage = |h: &MemHost| h.stdout.contains("Usage: p observe [options]");
    let cases: &[(&[&str], &str)] = &[
        (&["-l", "run.log"], "The following option is required: [--spec]"),
        (&["--spec", "NeverOverdrawn", "--jars", "x.jar"], "Was passed main parameter '--jars' but no main parameter was defined in your arg class"),
        (&["--spec", "Bad.Name"], "The specified class name spec (Bad.Name) is invalid."),
        (&["--spec", "NeverOverdrawn"], "No log files specified. Please provide log file or directory with --logs parameter."),
        (&["--spec", "NeverOverdrawn", "-l", "nope"], "The specified log location(nope) does not exist."),
        (&["--spec", "NeverOverdrawn", "-l", "run.log", "--replaySize", "0"], "The specified replaySize (0) is not valid (Must be an integer > 0)"),
        (&["--spec", "NeverOverdrawn", "-l", "run.log", "-ik", "xml"], "The specified input kind (xml) is not valid."),
        (&["--spec", "NeverOverdrawn", "-l", "run.log", "--keys"], "Expected a value after parameter --keys"),
        (&["--spec", "Missing", "-l", "run.log"], "SPECIFICATION_SUPPLIER not found in the specified jar."),
        (&["--spec", "NeverOverdrawn", "-l", "run.log", "-p", "Nope"], "PARSER not found in the specified jar."),
    ];
    for (args, msg) in cases {
        assert_eq!(run(&mut h, args), 1, "{:?}", args);
        assert!(h.stdout.starts_with(&format!("Failed parsing ::\n{}\n", msg)), "{:?}\n{}", args, h.stdout);
        assert!(usage(&h));
    }
    assert_eq!(run(&mut h, &["-h"]), 0);
    assert!(h.stdout.starts_with("Usage: p observe [options]") && !h.stdout.contains("--jars"));

    // no project to take the spec from
    let mut bare = MemHost::new("/work");
    bare.add_file("run.log", "");
    assert_eq!(run(&mut bare, &["--spec", "S", "-l", "run.log"]), 1);
    assert!(bare.stdout.contains("No P project file (*.pproj) found in the current folder"), "{}", bare.stdout);
    // type errors are shown
    bare.add_file("bad.p", "spec S observes e { start state A {} }");
    assert_eq!(run(&mut bare, &["--spec", "S", "-l", "run.log", "-pf", "bad.p"]), 1);
    assert!(bare.stdout.contains("[Error:]"), "{}", bare.stdout);
}

#[test]
fn custom_parsers_plug_in() {
    use pnext_checker::observe::{ObservedEvent, Parser, Spec, Timestamp};
    use pnext_checker::value::Value;

    struct Csv;
    impl Parser for Csv {
        fn name(&self) -> &str {
            "Csv"
        }
        fn parse(&self, entry: &str, spec: &Spec) -> Result<Vec<ObservedEvent>, String> {
            let f: Vec<&str> = entry.split(',').collect();
            let n = |i: usize| f[i].trim().parse::<i64>().map_err(|e| e.to_string());
            let ev = spec.tuple_event(f[1], Timestamp::from_millis(n(0)?), f[2], vec![Value::Int(n(3)?), Value::Int(n(4)?)], Some(entry))?;
            Ok(vec![ev])
        }
    }
    let mut h = host();
    h.add_file("run.csv", "1,k,eDeposit,1,5\n2,k,eWithdraw,1,6");
    let args: Vec<String> = ["--spec", "NeverOverdrawn", "-l", "run.csv", "-o", "out"].iter().map(|s| s.to_string()).collect();
    let code = pnext_compat::observe::run_with(&args, &mut h, vec![Box::new(Csv)]);
    assert_eq!(code, 2, "{}", h.stdout);
    assert!(h.stdout.contains("account 1 overdrawn: balance -1"), "{}", h.stdout);
}

#[test]
fn regression_parsers_compile() {
    assert_eq!(support::pobserve_parsers::all().len(), 6);
}
