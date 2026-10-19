# Checking logs with `prs observe`

`prs observe` checks events recorded by a running system against the spec monitors of
a P project. It replaces [PObserve](https://github.com/p-org/P/tree/master/Src/PObserve)
and follows its options, console output and `PObserve-<date>/` output. The
`prs-compat` process and the structured report's `exit_code` preserve PObserve
status codes (0 ok, 1 command line, 2 spec violation, 3 internal error, 4 parser
errors only). The native `prs` process uses 0 for success, 1 for a spec violation,
and 2 for tool or input errors; supervisors should check the process status and
read the report for detail.

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
| `--strict` | Require at least one successfully checked relevant event per selected spec, after key filtering. |
| `--unknown-events ignore\|warn\|error` | What to do with event names no loaded P source declares (default `ignore`, as PObserve). See [Unknown event names](#unknown-event-names). |
| `--coverage` | Report observed monitor state and handler coverage, including in JSON when requested. |

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

## Time and latency budgets

Specs can declare the same global foreign time functions used by `prs check`:

```p
fun Now(): int;
fun NowBest(): int;
fun Mark(): int;
fun Elapsed(m: int): int;
fun ElapsedBest(m: int): int;
```

Time tracking is enabled automatically by these declarations; signatures are checked
as in `prs check`. During each observed event, `Now()` and `NowBest()` return its
log timestamp in **epoch microseconds**. ISO-8601 timestamps retain microsecond
precision (nanoseconds are rounded down); numeric timestamps remain epoch
milliseconds in the log format and are converted to microseconds. Values outside
the P `int` range saturate. Ordering still uses the full timestamp precision.

`Mark()` saves the current clock and returns a handle. `Elapsed(m)` and
`ElapsedBest(m)` return the exact difference from that mark in microseconds, so an
existing budget assertion such as `assert Elapsed(m) <= 2000` checks the recorded
duration. Each partition key has a fresh monitor and its own handles, starting at
zero. The spec's start entry action reads time zero; mark request starts in their
event handlers to measure request duration. State transitions and raised events
within a handler keep that observed event's clock.

`Elapse` and `ElapseCost` cannot be called from a monitor, as in `prs check`.
Observation uses recorded timestamps and takes no latency profile or `--timed`
option. Logs without timestamps keep the parser's existing default of epoch zero;
provide timestamps to check meaningful budgets. `PTrace` currently has no time
entries, so its observations also use zero.

## Observed monitor coverage

Add `--coverage` to print state and handler coverage to standard output, including
unvisited states and uncovered handlers. With `--json-report report.json`, each
selected spec also gets a `coverage` object:

```sh
prs observe -pf PSpec/Bank.p --spec BankBalanceCorrect -l app.log \
  --coverage --json-report report.json
```

This measures monitor executions over the selected log entries. It does **not**
measure full P model paths, branches, schedules, or correctness of event emission.
The denominator contains every state and every explicit do, goto, or ignore
handler in the selected monitor, including unreachable and unvisited declarations.
Deferred events are excluded because the Java monitor semantics do not generate
handlers for them. The initial state counts only when a partition actually starts
its monitor; an empty log has no state hits. A failing event's attempted handler
and all states entered before failure are included.

The JSON object's `scope` is `observed_monitor`. Both `states` and `handlers`
contain `total`, `covered`, `declared`, and `visited`. State identities are
`Spec.State`; handler identities are objects with `state` and `event` strings.
`transitions` is a list of actual `{source, target}` transitions, including jumps
from action bodies. It has no denominator because such jumps can be dynamic.
These arrays are sorted and deduplicated. Union the hit sets across test cases;
do not sum their covered counts. Only combine reports from the same specification
version. Coverage collection is opt-in; existing output and JSON are unchanged
without `--coverage`.

Rust callers can set `ObserveOptions.coverage = true` and read `Report.coverage`.
`prs_checker::observe_coverage::ObserveCoverage` exposes the declaration and hit
sets, `merge(&other)` for test-suite unions, `render_text()` for human output, and
`to_json()` for the same structured coverage object as the CLI.

## Live checks for external tests

Use `--stdin --fail-fast` to check newline-delimited events while a test or service is
still running:

```sh
producer | prs observe --pfiles inventory.p --spec Inventory \
  --stdin --fail-fast --check-hot --json-report observe.json
```

The producer must flush each complete line. Unlike batch log observation, live
observation does not sort: timestamps must be nondecreasing **within each partition
key** (equal timestamps are allowed). Merge events from several producers into one
ordered stream before writing stdin. Separate keys may interleave freely.

The observer checks each line before requesting the next one. Its first parser,
ordering, monitor initialization or spec error stops the run and produces the usual
human diagnostics and JSON report, even if stdin remains open. Every selected spec
sees the current line before the observer stops. Replay storage retains only the last
`--replaySize` events per key, alongside each key's monitor state. The observer retains
monitor state for every encountered key until the run ends.

At EOF, `--check-hot` checks monitors for unfinished hot states. Hot states do not
fail a live, incomplete stream. An early failure reports only the consumed prefix;
JSON includes `input_complete: false` and `stop_reason: "error"`. At EOF these are
`true` and `"eof"`, including when an EOF liveness check fails. Coverage, when enabled,
describes the checked prefix.

Live mode requires both flags, TEXT input, and a newline-delimited parser; it cannot
be combined with `--logs`. Existing batch observation retains its sorting behavior.
`prs` returns 1 for a spec violation and 2 for parser, ordering or transport errors.

For external tests, run the observer under a supervisor that streams events, watches
its exit status, terminates child processes on failure, and closes stdin only after
all producers finish successfully. The observer does not terminate the producers
itself. A shell pipeline alone cannot reliably supervise long-running children.

Embedding applications can use `ObserveSession` to push one entry at a time and
inspect `failed()`, then `finish(eof, now)` to collect a report. Give each selected
spec its own session and feed all sessions before reading another line.

### Combining coverage across tests and processes

Coverage collection and suite union belong to the core library. A report keeps declared
and visited states, state/event handlers, and observed transitions. Merge takes their
set union: repeated inputs do not inflate counts. Counts are derived from the sets.
It does not run a P model or recheck events.

Save each complete observation with `--coverage --json-report`, then combine reports:

```sh
prs coverage merge-observe reports/*.json --json-report suite-coverage.json
```

The command prints a human-readable summary, including uncovered declarations. Use
`--json` for structured stdout instead. `--json-report` writes structured data without
mixing JSON into the human-readable stdout. The existing `prs coverage merge` command
continues to merge scenario coverage and is separate from observed monitor coverage.

Test adapters can avoid an extra subprocess per test by collecting full observation
reports in memory and passing their JSON array once at suite completion:

```sh
prs coverage merge-observe --stdin --json-report suite-coverage.json < observations.json
```

An empty array produces an empty coverage aggregation. Each input must contain coverage;
missing, malformed, unsupported-version, or internally inconsistent coverage is rejected.
Merged reports can themselves be merged again. The core Rust API is
`prs_checker::observe_report::ObserveCoverageReport`: use `from_observation` for an
in-process `Report`, `from_json` / `from_json_str` for stored reports, `merge` for an
atomic validated union, and `render_text` / `to_json` to report the result.

The versioned suite format has `scope: "observed_monitor_suite"`, `schema_version: 1`,
a `specs` array containing each specification's union, and an `observations` array
retaining distinct source report metadata (exit status, errors, and input completion
when supplied). A successful merge means only that aggregation succeeded. Failed and
partial observations still contribute useful coverage; their failure and completion
metadata remains visible. File observations from older tools may have unknown completion.
Identical metadata entries are deduplicated, so the observation count is not a test count.

Each specification's coverage carries a SHA-256 fingerprint of all compilation source
contents, sorted with explicit file-count and length delimiters. Source paths do not
participate: reports from different checkout directories can merge. Changes to formatting
or unrelated compilation sources deliberately change the conservative fingerprint.
Reports for the same specification must have the same fingerprint and identical
declarations; otherwise the merge fails before changing the result. Regenerate legacy
reports with `observe --coverage` to obtain current version and fingerprint fields.


## Strict observation

Use `--strict` in integration tests to catch accidentally disabled instrumentation,
empty recordings, or filters that discard all relevant events. Every selected spec
must successfully check at least one event. A spec with an existing parser, ordering,
or safety error keeps that error instead of receiving a secondary validation error.
A missing explicitly requested spec is already an error without strict mode.

Strict failures have JSON error kind `validation` and native `prs` exit code 2.
Live streams validate nonempty input only at EOF. An actual violation still stops
`--fail-fast` immediately, with prefix coverage and `input_complete: false`.
Strict mode does not imply `--check-hot`, check every key named by a filter, or
detect arbitrary missing events: use model assertions or explicit completion events
for stronger application-specific guarantees.

Strict mode does not catch a misspelled or undeclared event name: such events are
dropped before checking. Combine it with `--unknown-events error` for that.

Rust callers set `ObserveOptions.strict = true`. A strict `ObserveSession.finish(false,
now)` rejects an otherwise clean incomplete prefix. Supervisors finishing several
sessions after a known error may use `finish_after_failure(now)` to preserve the
original error without secondary validation failures; they must retain that original
error in the overall test result.

## Unknown event names

By default, as in PObserve, an event whose name no loaded P source declares is
dropped silently, so a misspelled `emit` (`eReserverd` for `eReserved`) hides the
very bug the spec should catch. `--unknown-events` decides what happens to them:

* `ignore` (default): drop them. Production logs often hold events the spec does
  not model.
* `warn`: print one line per distinct name to stderr, with its count, where it first
  appeared and a likely intended name; the result and exit status are unchanged. The
  JSON report lists the names under `unknown_events` and the lines under `warnings`.
* `error`: report each distinct name once as an error of kind `unknown_event`, with
  the same details. The exit status is the tool/input error (native `prs` 2, PObserve
  3), even alongside a spec violation, since the input cannot be trusted. Known events
  are still checked. With `--stdin --fail-fast` the run stops at the first unknown
  name.

```text
warning: Unknown event 'eReserverd': not declared in the P sources (1 occurrence, first at typo.jsonl:2); did you mean eReserved?
```

Events that a loaded P source declares but the selected spec does not observe are
always ignored: one recording can feed several specs, each watching a subset. To
accept names that only other tooling uses, declare them in a P file
(`event eAuditTrail;`) or use `warn`. `--strict` does not imply `--unknown-events`.

Locations are `<file>:<line>` (or `<file> entry <n>` for parsers whose entries span
lines), and `stdin:<line>` for live input. The built-in parsers report undeclared
names; a custom Rust `Parser` does so by overriding `parse_entry`. Rust callers set
`ObserveOptions.unknown_events` (with `input_names` for locations), for both
`Spec::observe` and `ObserveSession`.
