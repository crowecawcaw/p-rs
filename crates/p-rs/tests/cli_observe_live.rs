//! Live observation must stop even when an external producer keeps stdin open.
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const SPEC: &str = r#"
fun Now(): int;
event Stock: int;
event Take: int;
event Done;
spec Inventory observes Stock, Take, Done {
  var available: int;
  start hot state Active {
    entry { assert Now() == 0, "each key starts at time zero"; }
    on Stock do (n: int) { available = available + n; }
    on Take do (n: int) { assert n <= available, "oversold"; available = available - n; }
    on Done goto Complete;
  }
  cold state Complete {}
}
spec Other observes Stock { start state Ready { on Stock do (n: int) { assert n >= 0, "negative"; } } }
"#;

fn run(input: &str, eof: bool, extra: &[&str]) -> (i32, String, String) {
    let root = std::env::temp_dir().join(format!(
        "prs-live-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("inventory.p"), SPEC).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_prs"));
    command
        .current_dir(&root)
        .args(["observe", "inventory.p", "--spec", "Inventory", "--stdin", "--json-report", "report.json"])
        .args(extra)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().unwrap();
    let mut writer = child.stdin.take().unwrap();
    writer.write_all(input.as_bytes()).unwrap();
    writer.flush().unwrap();
    let writer = if eof {
        drop(writer);
        None
    } else {
        Some(writer)
    };
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if Instant::now() > deadline {
            child.kill().unwrap();
            let output = child.wait_with_output().unwrap();
            panic!("observe waited for EOF after input: {}", String::from_utf8_lossy(&output.stdout));
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    drop(writer);
    let output = child.wait_with_output().unwrap();
    let report = std::fs::read_to_string(root.join("report.json")).unwrap();
    std::fs::remove_dir_all(root).unwrap();
    (output.status.code().unwrap(), String::from_utf8(output.stdout).unwrap(), report)
}
static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[test]
fn violation_exits_while_producer_keeps_stdin_open() {
    let (code, output, report) =
        run("{\"event\":\"Stock\",\"payload\":1}\n{\"event\":\"Take\",\"payload\":2}\n", false, &["--coverage"]);
    assert_eq!(code, 1, "{output}");
    assert!(output.contains("oversold"));
    assert!(report.contains("\"input_complete\": false"));
    assert!(report.contains("spec_violation"));
    assert!(output.contains("Observed monitor coverage"), "{output}");
    assert!(report.contains("\"coverage\""));
    assert!(report.contains("Inventory.Active"));
}

#[test]
fn parser_error_and_out_of_order_exit_before_eof() {
    let (code, _, report) = run("not json\n", false, &[]);
    assert_eq!(code, 2);
    assert!(report.contains("\"kind\": \"parser\""));
    let (code, _, report) = run(
        "{\"event\":\"Stock\",\"timestamp\":2,\"payload\":1}\n{\"event\":\"Stock\",\"timestamp\":1,\"payload\":1}\n",
        false,
        &[],
    );
    assert_eq!(code, 2);
    assert!(report.contains("out_of_order"));
}

#[test]
fn success_at_eof_and_hot_state_checked_only_at_eof() {
    let (code, output, report) =
        run("{\"event\":\"Stock\",\"payload\":1}\n{\"event\":\"Done\"}\n", true, &["--check-hot"]);
    assert_eq!(code, 0, "{output}");
    assert!(report.contains("\"input_complete\": true"));
    let (code, output, _) = run("{\"event\":\"Stock\",\"payload\":1}\n", true, &["--check-hot"]);
    assert_eq!(code, 1, "{output}");
    assert!(output.contains("hot state"));
}

#[test]
fn every_spec_observes_the_same_live_stream() {
    let (code, output, report) = run("{\"event\":\"Stock\",\"payload\":-1}\n", false, &["--spec", "Other"]);
    assert_eq!(code, 1, "{output}");
    assert!(report.contains("negative"));
    assert!(report.contains("Inventory"));
    assert!(report.contains("Other"));
}

#[test]
fn partitions_have_fresh_state_and_bounded_replays() {
    let input = concat!(
        "{\"key\":\"a\",\"event\":\"Stock\",\"timestamp\":9,\"payload\":3}\n",
        "{\"key\":\"b\",\"event\":\"Stock\",\"timestamp\":1,\"payload\":1}\n",
        "{\"key\":\"b\",\"event\":\"Take\",\"timestamp\":2,\"payload\":2}\n"
    );
    let (code, output, report) = run(input, false, &["--replay-window", "1"]);
    assert_eq!(code, 1, "{output}");
    assert!(report.contains("\"key\": \"b\""));
    assert!(!report.contains("\"event\": \"Stock\""));
    assert!(report.contains("\"event\": \"Take\""));
}

#[test]
fn strict_live_validates_at_eof_without_implying_hot_checks() {
    let (code, output, report) = run("", true, &["--strict", "--coverage"]);
    assert_eq!(code, 2, "{output}");
    assert!(report.contains("\"kind\": \"validation\""));
    assert!(report.contains("\"input_complete\": true"));
    let (code, output, _) = run("{\"event\":\"Stock\",\"payload\":1}\n", true, &["--strict"]);
    assert_eq!(code, 0, "{output}");
    // No Stock events for Other: its strict check must not mask Inventory's violation
    // or wait for EOF after the first failing event.
    let (code, output, report) = run("{\"event\":\"Take\",\"payload\":1}\n", false, &["--strict", "--spec", "Other"]);
    assert_eq!(code, 1, "{output}");
    assert!(report.contains("spec_violation"));
    assert!(!report.contains("\"kind\": \"validation\""));
}

#[test]
fn unknown_event_error_stops_at_the_first_undeclared_name() {
    let input = "{\"event\":\"Stock\",\"payload\":1}\n{\"event\":\"Stokc\",\"payload\":1}\n";
    let (code, output, report) = run(input, false, &["--unknown-events", "error"]);
    assert_eq!(code, 2, "{output}");
    assert!(output.contains("first at stdin:2); did you mean Stock?"), "{output}");
    assert!(report.contains("\"kind\": \"unknown_event\""));
    assert!(report.contains("\"input_complete\": false"));
    // declared events another spec observes are not unknown
    let (code, output, _) = run("{\"event\":\"Done\"}\n", true, &["--unknown-events", "error", "--spec", "Other"]);
    assert_eq!(code, 0, "{output}");
}

/// Two producers on named pipes, merged by sequence number while both stay open.
#[cfg(unix)]
#[test]
fn named_pipes_merge_by_sequence_and_fail_while_producers_are_open() {
    use std::fs::OpenOptions;
    let root = std::env::temp_dir().join(format!("prs-live-pipes-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("inventory.p"), SPEC).unwrap();
    for pipe in ["a", "b"] {
        assert!(Command::new("mkfifo").arg(root.join(pipe)).status().unwrap().success());
    }
    let mut child = Command::new(env!("CARGO_BIN_EXE_prs"))
        .current_dir(&root)
        .args(["observe", "inventory.p", "--spec", "Inventory"])
        .args(["--live", "a", "--live", "b", "--sequence", "seq", "--record", "events.jsonl"])
        .args(["--json-report", "report.json"])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let open = |name: &str| OpenOptions::new().write(true).open(root.join(name)).unwrap();
    let (mut a, mut b) = (open("a"), open("b"));
    // b's take arrives first but must wait for a's stock
    writeln!(b, "{{\"seq\":2,\"event\":\"Take\",\"payload\":1}}").unwrap();
    std::thread::sleep(Duration::from_millis(100));
    writeln!(a, "{{\"seq\":1,\"event\":\"Stock\",\"payload\":1}}").unwrap();
    writeln!(b, "{{\"seq\":3,\"event\":\"Take\",\"payload\":1}}").unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        assert!(Instant::now() < deadline, "observer waited for EOF");
        std::thread::sleep(Duration::from_millis(10));
    };
    let output = child.wait_with_output().unwrap();
    assert_eq!(status.code(), Some(1), "{}", String::from_utf8_lossy(&output.stdout));
    let report = std::fs::read_to_string(root.join("report.json")).unwrap();
    assert!(report.contains("oversold"), "{report}");
    assert!(report.contains("\"input_complete\": false"));
    let recorded = std::fs::read_to_string(root.join("events.jsonl")).unwrap();
    let order: Vec<&str> = recorded.lines().map(|l| &l[7..8]).collect();
    assert_eq!(order, ["1", "2", "3"], "{recorded}");
    drop((a, b));
    std::fs::remove_dir_all(root).unwrap();
}
