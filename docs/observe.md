# Checking logs with `prs observe`

`prs observe` checks events recorded by a running system against the spec monitors of
a P project. It replaces [PObserve](https://github.com/p-org/P/tree/master/Src/PObserve)
and follows its options, console output, `PObserve-<date>/` output and exit codes
(0 ok, 1 command line, 2 spec violation, 3 internal error, 4 parser errors only).

The difference is where the spec comes from: there are no jars. The P sources are
type-checked in memory, from the `.pproj` in the current folder, `-pp/--pproj <file>`
or `-pf/--pfiles <files>`.

```sh
prs observe --spec BankBalanceCorrect -l logs/                 # JSON lines
prs observe --spec BankBalanceCorrect -l trace.json -i JSON    # a P JSON trace
prs observe -pf PSpec/Bank.p --spec BankBalanceCorrect -l app.log --keys 7 9
```

## prs extensions

These options are not in PObserve; runs that do not use them behave as upstream.

| Option | Effect |
| --- | --- |
| `--spec A B ...` | Check several specs in one run. Each writes its replay logs and metrics to `<outputDir>/<Spec>/`. |
| `--all-specs` | Check every spec in the P sources (instead of `--spec`). |
| `--exclude-specs A B ...` | Specs `--all-specs` leaves out. |
| `--check-hot` | Also report a key whose spec is in a hot state after its last event, with the message `prs check` uses for liveness bugs at the end of an execution. |
| `--json-report <file>` | Write the result as JSON (below). |

```json
{
  "exit_code": 2,
  "specs": [
    {
      "spec": "MutualExclusion",
      "events_read": 3,
      "partition_keys": 1,
      "verified_keys": 0,
      "errors": [
        {
          "kind": "spec_violation",
          "key": "L1",
          "message": "Assertion failure: specs/lock.p:19:7 c2 entered while c1 is inside",
          "cause": null,
          "events": [
            {"key": "L1", "timestamp": "2026-10-18T00:00:00.000000001Z", "event": "eEnter", "log_line": "{...}"},
            {"key": "L1", "timestamp": "2026-10-18T00:00:00.000000002Z", "event": "eEnter", "log_line": "{...}"}
          ]
        }
      ]
    }
  ]
}
```

`kind` is `spec_violation`, `out_of_order`, `parser` or `unknown`. `events` are the
key's last events (`--replaySize`), the failing one last; parser errors have none,
their `message` is the log entry and `cause` says why it could not be read.

[docs/python.md](python.md) builds on these to check specs from pytest.

## Log formats

Parsers turn log entries into events. Two are built in:

* `JsonLines` (default for `-i TEXT`): one JSON object per line,
  `{"key": "7", "timestamp": 1700000000000, "event": "eWithDraw", "payload": {...}}`.
  The timestamp is epoch milliseconds or ISO-8601. Payloads are matched against the
  event's payload type. `-pc '{"key": "ctx.account", "event": "type"}'` remaps the
  fields (dotted paths).
* `PTrace` (default for `-i JSON`): the `.trace.json` written by `prs check`, checking
  its sends and announcements.

## Custom parsers

Other log formats need a parser written in Rust: implement
`prs_checker::observe::Parser` and run `prs_compat::observe::run_with(args, host, parsers)`.
`crates/prs-cli/examples/observe_regression.rs` is a worked example; it ports
upstream's regression parsers.
