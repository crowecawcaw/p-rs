# Checking P specs from Python tests

The `p-rs` Python package checks the team's P specs against events the code emits
during its ordinary tests. Code calls `emit("eEnter", client=c)`; after each test the
package runs [`prs observe`](observe.md) on that test's events and fails the test if a
spec was broken. It needs only P specs (event declarations and `spec` machines), not a
P model.

It only sees the orderings the tests produce: it checks the implementation against the
design, it does not explore it.

```sh
pip install p-rs        # brings the prs binary; no Rust or Java needed
```

The source is in [`python/`](../python). Until wheels are on PyPI, `pip install ./python`
in a checkout builds the binary with cargo.

For runnable application projects with fixtures, configuration, and CI, see the
[Python inventory examples](../examples/python/README.md).

## Emitting events

`emit` is the only thing application code imports. It does nothing unless a test is
recording, so it is safe in production code.

```python
from prs import emit

def grant(self, client, lock):
    ...
    emit("eEnter", key=lock, client=client)
    emit("eCommitted", key=tx, at=row.commit_time)   # when it happened, not when logged
```

```python
emit(event, payload=..., /, *, key=None, at=None, **fields)
```

* **Payload**: keyword arguments, matched by name against the P event's payload type
  (`event eEnter: (client: string)`). A single positional value works too: for events
  whose payload is not a named tuple (`emit("eClose", 7)`), or whose fields are called
  `key` or `at` (`emit("ePut", {"key": k, "value": v})`). Python enums are sent by name,
  sets and tuples as lists.
* **`key`** splits events into independent streams, e.g. one per lock or account. Each
  stream gets its own instance of every spec.
* **`at`** overrides the event time: a `datetime`, or an integer of epoch nanoseconds.
  Any increasing counter works, for example a logical clock. Default: now.
* Each event records the file and line it was emitted from, for failure reports.
* Emit when a change becomes visible (after the commit), not when it is attempted;
  otherwise specs see states that never existed.

## Every test (the pytest plugin)

Installing the package adds a pytest plugin. With one setting, every test also checks
every spec:

```toml
# pyproject.toml
[tool.prs]
spec_files = ["specs/*.p"]       # globs, relative to this file; folders mean every .p in them
exclude = ["DraftSpec"]          # optional: specs not checked anywhere yet
mode = "auto"                    # optional: "auto" (every test) | "marked" | "off"
```

* Without a `[tool.prs]` table the plugin does nothing.
* A test that emits nothing passes without running prs.
* `pytest --prs off` (or `marked`, `auto`) overrides `mode` for one run.
* Events emitted while fixtures are set up count; the check runs when the test body
  ends, so a broken spec is a test failure, not a teardown error. A test that already
  failed is not checked.

Liveness (a spec left in a hot state, i.e. still waiting for something) is checked only
when asked for, because a unit test that stops mid-protocol ends in a hot state:

```python
@pytest.mark.prs(skip=True)              # don't check this test
@pytest.mark.prs(skip=["DraftSpec"])     # leave out some specs here
@pytest.mark.prs(liveness=True)          # also fail on a spec left in a hot state
```

In `mode = "marked"`, only tests with a `prs` mark are checked.

The `prs_events` fixture is the test's recorder, for ordinary assertions:

```python
def test_two_clients(prs_events):
    run_two_clients()
    assert prs_events.count("eEnter") == 2
    assert prs_events.events_named("eEnter") == [{"client": "c1"}, {"client": "c2"}]
```

## Explicit checks

Three functions make the check visible in the test. They work without the plugin.

```python
from prs import check_spec, expect_violation, check_events

def test_lease_expiry_hands_lock_to_next_client():
    with check_spec("MutualExclusion") as rec:      # no names: every spec
        server = LockServer(lease=1)
        server.acquire("c1")
        clock.advance(2)
        server.acquire("c2")
    assert rec.events_named("eEnter") == [{"client": "c1"}, {"client": "c2"}]

def test_spec_catches_double_holder():              # tests the spec, not the app
    with expect_violation("MutualExclusion", match="c2 entered"):
        check_events([
            {"event": "eEnter", "payload": {"client": "c1"}},
            {"event": "eEnter", "client": "c2"},     # payload fields directly also work
        ], "MutualExclusion")
```

* `check_spec(*specs, spec_files=None, liveness=False)` checks the events emitted inside
  the block and raises `SpecViolation` (an `AssertionError`) at the end of it, so the
  test fails on the `with` line.
* `expect_violation(spec, match=None, ...)` requires the block to break `spec`, like
  `pytest.raises`. `match` is a regular expression searched for in the spec's message.
  Enclosing checks (the plugin's, an outer `check_spec`) ignore that spec for the rest of
  the test, so a deliberate violation is not reported twice.
* `check_events(events, *specs, ...)` checks a hand-written event list (in list order,
  unless events have `at`). Those events reach no other check.
* `spec_files` defaults to `[tool.prs] spec_files`, so `check_spec` can name a spec the
  plugin does not check yet, to trial it on one test.

Checks nest: every event goes to every active recorder. The set of active recorders is
process-wide, not per thread or task, so events from threads a test starts are kept.

## Failure reports

```
FAILED test_locks.py::test_lease_expiry_hands_lock_to_next_client

E   prs.SpecViolation: P spec MutualExclusion was broken: c2 entered while c1 is inside

      Events (key "L1"):
          1  eEnter  client=c1  app/locks.py:27
          2  eEnter  client=c2  app/locks.py:27  <- broke the spec here

      Spec: specs/lock.p:19
      prs output: /tmp/pytest-of-me/pytest-4/prs/prs-check-zbjgmbqw
```

The spec's assert message comes first, then the events of the key that broke it (the
last 20) with where each was emitted, then the assert in the P file. An event whose time
was set with `at=` says so, which explains an order that differs from the emit order.
The `prs output` folder holds the events file, prs's console output and its report.

An event whose payload does not match its P declaration (a missing or misspelled field)
fails the test with a `PrsError` naming the event, the field and where it was emitted.

The exception carries the details: `.spec`, `.message`, `.key`, `.events` (each with
`.event`, `.payload`, `.src`), `.location`, `.liveness` and `.others` (further
violations found by the same check).

## Events from other processes

| Source | How its events reach the check |
| --- | --- |
| Code in the test process | `emit` adds them to the active recorders in memory |
| Python processes the test starts | They inherit `PRS_EVENTS_DIR`; `emit` appends to a file of their own there |
| Non-Python services | Write the same JSON lines to `PRS_EVENTS_DIR` (a mounted volume for containers); [`python/emitters/go`](../python/emitters/go) is a one-file Go emitter |
| Existing logs | `rec.add_log("server.log", parse=fn)`: `fn` turns a line into a dict like `emit`'s arguments, with `at`, or `None` |
| A folder a container writes to | `rec.add_events_dir(path)` |

The line format is what `prs observe` reads; `src`, `seq`, `pid` and `at` are extra
fields used by the report:

```json
{"key": "L1", "timestamp": "2026-10-18T00:00:00.123456789Z", "event": "eEnter", "payload": {"client": "c1"}, "src": "app/locks.py:42"}
```

Ordering: prs sorts each key's events by timestamp; equal times keep file order (in-process
events first, then other processes', then logs). Timestamps are ISO-8601 with
nanoseconds, which prs orders exactly (a bare integer is read as milliseconds). All
processes on the test machine share a clock. Across machines, clock skew can reorder
events: give each stream (key) one owner, or pass a logical counter as `at`.

A service started once per session (a session-scoped fixture) starts before any test
records, so it gets no `PRS_EVENTS_DIR` and its events reach no test. For now, start it
with a folder of its own and check that folder once, at the end of the session:

```python
@pytest.fixture(scope="session")
def server(tmp_path_factory):
    events = tmp_path_factory.mktemp("server-events")
    proc = start_server(env={**os.environ, "PRS_EVENTS_DIR": str(events)})
    yield proc
    proc.terminate()
    rec = Recorder()
    rec.add_events_dir(str(events))
    rec.check()
```

## Configuration reference

| Setting | Where | Effect |
| --- | --- | --- |
| `spec_files`, `exclude`, `mode` | `[tool.prs]` in the nearest `pyproject.toml` | See above |
| `PRS_SPEC_FILES` | environment | P files (separated like `PATH`), overriding `spec_files` |
| `PRS_BINARY` | environment | The `prs` binary to run instead of the installed one |
| `PRS_EVENTS_DIR` | environment, set by recorders | Where child processes write events |

## How it works

Each recorder writes its events to one JSON-lines file and runs

```sh
prs observe -pf <spec files> --all-specs [--exclude-specs ...] -l <events> \
    --json-report report.json [--check-hot]
```

`--all-specs`, `--exclude-specs`, several names after `--spec`, `--json-report` and
`--check-hot` are prs extensions to PObserve's options (see [observe.md](observe.md)).
A check costs about 5 ms; tests that emit nothing cost nothing.

### Monitor coverage

Opt into suite coverage with `pytest --prs-coverage=coverage/monitor.json`.
The terminal summary reports the union of P monitor states and handlers visited
by completed automatic checks, and writes their identities as JSON for CI.
Tests retain independent monitor state. Skipped checks, tests whose bodies fail
before checking, and tests emitting no events do not contribute coverage.
This measures spec monitors, not Python lines or P model paths. For explicit recording,
`Recorder(coverage=True)` exposes the check's JSON as `coverage_report` after checking.

### Strict integration checks

Set `strict = true` under `[tool.prs]`, or run `pytest --prs-strict`, to require
at least one successfully checked relevant event for every selected spec in each
checked test. This includes tests that emit no events; ordinary tests without
instrumentation should use `@pytest.mark.prs(skip=True)` or marked mode.
Explicit checks support `check_spec(..., strict=True)`,
`check_events(..., strict=True)`, and `Recorder(strict=True)`. Strict setup failures
raise `PrsError`; application safety violations remain `SpecViolation`.
The default remains permissive. Strict mode does not infer arbitrary missing events
or imply liveness checks.
