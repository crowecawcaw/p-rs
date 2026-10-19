# Observe two local inventory processes

This integration test starts two independent Python application processes with
`subprocess.Popen`, sharing a SQLite inventory database, and a third process
running `prs observe`. Both applications emit events from committed inventory
transactions. The parent streams their combined history into the observer and
owns every process's lifetime. Python's standard library is the only application
dependency; no Docker daemon or Python `prs` package is required.

The application, worker protocol, P specification, and observer supervisor are
shared with the [Docker example](../docker/README.md). The local adapter is
[`SubprocessBackend`](run.py); switching launch method does not change the
domain rules or expected failure.

## Run from a checkout

From the repository root:

```sh
cargo build -p prs-cli --bin prs --locked
export PRS_BIN="$PWD/target/debug/prs"
python -m unittest discover -s examples/multiprocess/native -v
```

The two integration tests launch actual workers and validate these histories:

* **Healthy:** worker A stocks three units and reserves two. Worker B's attempt
  to reserve two is rejected. After A releases its reservation, B reserves and
  releases successfully. Both workers stop, then the supervisor closes observer
  stdin. The observer exits successfully at EOF.
* **Oversell:** the service's stock guard is disabled. After A reserves two,
  B incorrectly reserves another two. The P monitor fails with
  `reservation exceeds available stock`. The supervisor verifies the observer
  exited with both workers still alive and stdin still open, then terminates
  and reaps both workers. A parser error, missing event, timeout, or different
  assertion cannot satisfy this test.

Run either as a standalone command as well:

```sh
python examples/multiprocess/native/run.py --scenario healthy
python examples/multiprocess/native/run.py --scenario oversell
```

The healthy command exits **0**. The oversell command exits **1**, preserving
the observer's failure for a test runner in another process. The unittest suite
expects that specific failure, so the complete suite exits 0. Both commands
print the observer's human-readable diagnostics and coverage, followed by a JSON
summary and artifact location. Use `--prs /absolute/path/to/prs` to select the
binary explicitly or `--artifacts /path/to/new-directory` to choose output.
Each default run uses a fresh directory under `native/artifacts`; existing
output directories are never overwritten.

## Setup to reuse in a project

[`common/harness.py`](../common/harness.py) owns the testing boundary. It starts
the observer before the applications, waits for ready messages, and drives
operations using command acknowledgements. Queue reads, worker waits, and
shutdown have timeouts; readiness and observer failure require no sleeps.
Cleanup runs in `finally` even when startup or checking fails. The adapter
terminates each still-running worker, escalates to kill after five seconds, and
waits to reap it. These workers do not launch child processes.

To integrate your own service, replace the launch command in `SubprocessBackend`
and the commands/readiness adapter in the supervisor. Keep the observer's exit
watcher and process ownership in the outer test fixture. That fixture can run
around an external test executable as well: stop it on an observer failure,
and close observer stdin only after the producers have finished successfully.
The observer checks events; the supervisor cancels the producers.

The service accepts an event sink, keeping transport out of its business logic.
[`common/inventory.py`](../common/inventory.py) writes each event to an outbox
in the same SQLite transaction as the inventory change and publishes only after
commit. A shared outbox sequence captures commit order across workers. The
supervisor buffers arrivals by that sequence before flushing newline-delimited
JSON to `prs observe --stdin --fail-fast --strict --coverage --json-report ...`.
This ordering matters because two stdout pipes can deliver committed events in
a different order. Rejected operations emit no state-change event. The monitor
checks the shared product key, so it can detect a violation caused by actions
from different processes.

The sequence also supplies a logical timestamp for this safety property. If your
monitor checks elapsed time, record real timestamps and define an ordering policy
appropriate to your application. This small publisher does not retry deliveries;
a production outbox dispatcher also needs delivery tracking and a deduplication
policy. The supervisor detects duplicate sequence numbers and gaps before a
successful EOF.

## Reports and CI

Each scenario preserves:

| Artifact | Purpose |
| --- | --- |
| `observer.stdout` / `observer.stderr` | Human-readable checker diagnostics and monitor coverage |
| `report.json` | Structured checker result, errors, completion status, and coverage |
| `result.json` | Validated scenario summary, final stock, both emitters, cleanup, and whether workers were alive at failure |
| `events.jsonl` | The ordered event stream sent to the observer |
| `inventory.sqlite` | Committed service state and outbox |
| `worker-*.stdout`, `worker-*.stderr`, `observe/` | Worker protocol logs, diagnostics, and checker traces |

`report.json` includes each spec's `coverage` with `scope: "observed_monitor"`,
declared and visited states and handlers, and observed transitions. Healthy runs
cover both declared states and three of six declared handlers; the remaining
handlers reject malformed histories. Faulty runs report
coverage of the consumed prefix with `input_complete: false` and
`stop_reason: "error"`. This measures executed P monitor states/handlers rather
than exhaustive P model paths. See [the coverage format](../../../docs/observe.md#observed-monitor-coverage).

The [native workflow](../../../.github/workflows/native-multiprocess.yml) builds
this checkout's checker, executes both scenarios, and uploads reports even on a
test failure. `MULTIPROCESS_ARTIFACTS` can set the test suite's artifact parent.

The observer also uses `--strict`, so a completed stream containing no checked
relevant events fails instead of silently passing.
