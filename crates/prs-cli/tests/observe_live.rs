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
        .args([
            "observe",
            "--pfiles",
            "inventory.p",
            "--spec",
            "Inventory",
            "--stdin",
            "--fail-fast",
            "--outputDir",
            "out",
            "--json-report",
            "report.json",
        ])
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
    let (code, output, report) = run(input, false, &["--replaySize", "1"]);
    assert_eq!(code, 1, "{output}");
    assert!(report.contains("\"key\": \"b\""));
    assert!(!report.contains("\"event\": \"Stock\""));
    assert!(report.contains("\"event\": \"Take\""));
}
