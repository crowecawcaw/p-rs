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

The source is in [`python/`](../python); `pip install ./python` in a checkout builds the
binary with cargo. Names, defaults and failure behaviour match the other language
helpers ([language-support.md](language-support.md)). Runnable projects:
[Python examples](../examples/python/README.md).

## Emitting events

`emit` is the only thing application code imports. It does nothing unless a test is
recording (or a [production sink](#in-production) is set), so it is safe in production
code.

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
* An event name no P file declares (a typo) fails the check, naming the `emit` line and
  the closest declared name. See `unknown_events` [below](#configuration-reference).

### In production

The same `emit` calls can feed [`prs observe`](observe.md) on production logs. Set a
sink once at start-up:

```python
import prs

prs.set_sink("/var/log/app/events.jsonl")   # a file, appended to
prs.set_sink("-")                           # or stdout (or any text stream)
prs.set_sink(logger.info)                   # or a callable taking one JSON line
```

```sh
prs observe -pf specs/*.p --spec InventoryCorrect -l /var/log/app/events.jsonl
```

* Lines are the [events folder format](observe.md#events-folder-format): one JSON object
  per event, with `src`, `seq` and `pid`.
* A background thread writes them. `emit` never raises or blocks: when its queue (10,000
  lines) is full, a write fails or a payload cannot be encoded, the event is dropped and
  counted in `sink.dropped` (`sink = prs.set_sink(...)`).
* Queued lines are written at exit; `sink.flush()` waits for them earlier.
  `prs.set_sink(None)` turns the sink off.
* Tests still get the events: a recording test, or a process it started, takes
  precedence over the sink.
* Wall-clock times never decrease within a process. Across hosts, give each key one
  writer or pass `at=` from a logical clock ([ordering](observe.md#events-folder-format)).
* [Testing and production](production.md) covers deploying and checking these logs.

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

* Without a `[tool.prs]` table (or `PRS_SPEC_FILES` / `PRS_MODE`) the plugin does nothing.
* A test that emits nothing passes without running prs, unless it is strict.
* `pytest --prs off` (or `marked`, `auto`) overrides `PRS_MODE` and `mode` for one run.
* Events emitted while fixtures are set up count; the check runs when the test body
  ends, so a broken spec is a test failure, not a teardown error. A test that already
  failed is not checked.

Hot states (a spec still waiting for something) are checked only when asked for, because
a unit test that stops mid-protocol ends in a hot state:

```python
@pytest.mark.prs(skip=True)              # don't check this test
@pytest.mark.prs(skip=["DraftSpec"])     # leave out some specs here
@pytest.mark.prs(check_hot=True)         # also fail on a spec left in a hot state
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

* `check_spec(*specs, spec_files=None, check_hot=None, strict=None, unknown_events=None,
  timeout=None, artifacts=None)` checks the events emitted inside the block and raises
  `SpecViolation` (an `AssertionError`) at the end of it, so the test fails on the `with`
  line. Options left as `None` come from `[tool.prs]`, then the
  [defaults](#configuration-reference).
* `expect_violation(spec, match=None, ...)` requires the block to break `spec`, like
  `pytest.raises`. `match` is a regular expression searched for in the spec's message.
  Enclosing checks (the plugin's, an outer `check_spec`) ignore that spec for the rest of
  the test, so a deliberate violation is not reported twice.
* `check_events(events, *specs, ...)` checks a hand-written event list (in list order,
  unless events have `at`). Those events reach no other check.
* `Recorder(specs, ...)` is the manual form: `start()`, `stop()`, `check()`. It takes the
  same options, plus `coverage=True`.
* `spec_files` defaults to `[tool.prs] spec_files`, so `check_spec` can name a spec the
  plugin does not check yet, to trial it on one test.

Checks nest: every event goes to every active recorder. The set of active recorders is
process-wide, not per thread or task, so events from threads a test starts are kept.
Join those threads before the check: a thread started during a recording that emits
after its check, with no recording active, gets a `SetupError` of kind `config` from
`emit`, and under the plugin the test fails in teardown.

For Hypothesis, give each generated case its own recording: `with check_spec(...)`
inside the `@given` body, or a `Recorder` per `RuleBasedStateMachine`
([example](../examples/python/hypothesis/test_inventory.py)). The machine's `teardown`
also runs after a failed step; check only when `sys.exc_info()[1] is None`, so the
step's failure is the one reported.

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
The `prs output` folder holds the events, prs's console output and its report; it is
kept only for failed checks.

The exception carries the details: `.spec`, `.message`, `.key`, `.events` (each with
`.event`, `.payload`, `.src`), `.location`, `.liveness` (a hot state at the end, found
with `check_hot`), `.others` (further violations found by the same check) and
`.artifacts` (the `prs output` folder; `SetupError` has it too once the checker ran).

Anything else is a `prs.SetupError` with a `.kind`, for assertions without matching text:

| `kind` | Cause |
| --- | --- |
| `binary_missing` | No `prs` binary |
| `config` | Bad options or `[tool.prs]`, missing spec files, P compile errors, unknown spec name |
| `parser` | A payload that does not match its P declaration (named with its `emit` line), or a line that is not valid JSON |
| `validation` | `strict`: a selected spec checked no relevant event |
| `unknown_event` | An event name no P file declares, under `unknown_events = "error"` |
| `out_of_order` | Live input out of order |
| `timeout` | The check took longer than `timeout` |

## Events from other processes

| Source | How its events reach the check |
| --- | --- |
| Code in the test process | `emit` adds them to the active recorders in memory |
| Python processes the test starts | They inherit `PRS_EVENTS_DIR` (or pass `env=rec.env()`); `emit` appends to a file of their own there |
| Non-Python services | Write the same JSON lines to `rec.events_dir()` (a mounted volume for containers); [`go/prsemit`](../go/prsemit) is the Go emitter |
| Existing logs | `rec.add_log("server.log", parse=fn)`: `fn` turns a line into a dict like `emit`'s arguments, with `at`, or `None`. Without `parse`, the file holds these JSON lines |
| A folder a container writes to | `rec.add_events_dir(path)` |

```python
with check_spec("MutualExclusion") as rec:
    subprocess.run(["./worker"], env=rec.env(), check=True)
```

Child processes and services write the [events folder format](observe.md#events-folder-format):
one file per process, `<name>-<pid>-<random>.jsonl`, holding the JSON lines
`prs observe` reads plus `src`, `seq` and `pid` for reports. That section also covers
ordering (each key's events sorted by timestamp, equal times keeping in-process events
first), clock skew, and when a folder is complete: stop or join child processes before
the check. A truncated line is a `parser` error, never dropped.

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

Every option can be set in `[tool.prs]` (the nearest `pyproject.toml`; paths relative to
it) or passed to `check_spec`, `check_events`, `expect_violation` and `Recorder`.
Precedence: argument, then environment variable, then `[tool.prs]`, then default.
Unknown keys are an error.

| Setting | Default | Effect |
| --- | --- | --- |
| `spec_files` | required | P files: globs, or folders meaning every `.p` in them. `PRS_SPEC_FILES` (separated like `PATH`) overrides it |
| `specs` | every spec | Specs to check |
| `exclude` | none | Specs left out when `specs` defaults to every spec |
| `strict` | true with named `specs`, else false | Require a relevant event for every selected spec ([below](#strict-checks)) |
| `check_hot` | `false` | Also fail on a spec left in a hot state |
| `unknown_events` | `"error"` | Event names no P file declares: `"error"`, `"warn"` (a warning) or `"ignore"` |
| `timeout` | `30` | Seconds a check may take |
| `artifacts` | a temp folder | Where failed checks keep their events and output. `PRS_ARTIFACTS` overrides it |
| `mode` | `"auto"` | Plugin only: `"auto"`, `"marked"` or `"off"`. `PRS_MODE` overrides it; `pytest --prs` overrides both |

| Environment variable | Effect |
| --- | --- |
| `PRS_BIN` | The `prs` binary, instead of the installed one |
| `PRS_SPEC_FILES`, `PRS_ARTIFACTS`, `PRS_MODE` | Override the settings above |
| `PRS_COVERAGE` | Suite coverage JSON path, as `--prs-coverage` ([below](#monitor-coverage)) |
| `PRS_EVENTS_DIR` | Set by recordings: where child processes write events |

Relative paths in environment variables resolve against the working directory.

## How it works

Each check writes the test process's events to a file and runs

```sh
prs observe -pf <spec files> --all-specs [--exclude-specs ...] \
    -l <events file> <events folders> <logs> --json-report report.json \
    --unknown-events error [--strict] [--check-hot]
```

with `--spec <names>` instead of `--all-specs` when specs are named. These are prs
extensions to PObserve's options (see [observe.md](observe.md)). A check costs about
5 ms; tests that emit nothing cost nothing.

### Monitor coverage

Opt into suite coverage with `pytest --prs-coverage=coverage/monitor.json` (or
`PRS_COVERAGE`). The terminal summary reports the union of P monitor states and handlers
visited by completed automatic checks, and writes their identities as JSON for CI.
Tests retain independent monitor state. Skipped checks, tests whose bodies fail
before checking, and tests emitting no events do not contribute coverage.
This measures spec monitors, not Python lines or P model paths. For explicit recording,
`Recorder(coverage=True)` exposes the check's JSON as `coverage_report` after checking.

The run merges into an existing file under a lock (`<path>.lock`), so pytest-xdist
workers and several pytest runs can share one path. Delete the file to start over (CI
starts clean).

### Strict checks

Strict means every selected spec checked at least one relevant event, which catches
instrumentation that was removed or never ran. It does not imply `check_hot`.

* With named specs (`check_spec("Name")`, `Recorder(["Name"])`, `[tool.prs] specs`),
  `strict` defaults to true: a named spec that checked no event fails the check with a
  `SetupError` of kind `validation`. Pass `strict=False` to allow that.
* With every spec selected (the plugin's default), `strict` defaults to false, since
  most tests exercise only some specs. Turn it on with `strict = true` under
  `[tool.prs]` or `pytest --prs-strict`; this includes tests that emit no events, so mark
  uninstrumented tests `@pytest.mark.prs(skip=True)` or use marked mode.
