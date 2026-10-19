# Python: inventory reservations

A small inventory service reserves stock for orders and releases it on cancellation.
All three examples run the same [application](inventory_app/__init__.py) against the same
[P spec](specs/inventory.p). The difference is how tests start recording and check
that spec. These are application integration examples; there is no P model to run.

## Install and run

From the repository root, with Python 3.9+ and a Rust toolchain (the local `p-rs`
package builds the `prs` binary):

```sh
python -m venv examples/.venv
. examples/.venv/bin/activate
python -m pip install ./python -e './examples/python[test]'
```

Run each project separately so pytest uses that project's configuration:

```sh
(cd examples/python/explicit && python -m pytest)
(cd examples/python/pytest && python -m pytest)
(cd examples/python/hypothesis && python -m pytest)
```

On Windows, activate with `examples\.venv\Scripts\activate` and run `cd` and
`python -m pytest` separately in each example directory. After installation, running
the examples needs no Rust commands, external services, network access, or sleeps.
The [Python CI workflow](../../.github/workflows/python.yml) runs all three projects
against this checkout's package on Linux and macOS with Python 3.9 and 3.13.

## Explicit checks in selected tests

Read [explicit/conftest.py](explicit/conftest.py), then
[explicit/test_inventory.py](explicit/test_inventory.py).

`check_spec("InventoryCorrect")` records events only inside its `with` block and
checks them when the block exits. `mode = "off"` in
[explicit/pyproject.toml](explicit/pyproject.toml) disables automatic checking;
explicit calls still load the configured spec files. Paths in `spec_files` are
relative to that configuration file.

The factory fixture creates and seeds a fresh inventory **inside** the block. A
pre-seeded fixture would emit initial stock before recording began, leaving the
monitor without its starting state. Start recording before any setup whose events
the spec needs.

The final test uses `check_events` and `expect_violation` to prove the spec rejects
an overselling trace. It passes only if the expected violation is detected, keeping
CI green while testing the spec itself.

## Automatic checking across pytest tests

Read [pytest/pyproject.toml](pytest/pyproject.toml),
[pytest/conftest.py](pytest/conftest.py), and
[pytest/test_inventory.py](pytest/test_inventory.py).

Installing `p-rs` registers its pytest plugin. `[tool.prs]` with `mode = "auto"`
enables checks for every test in this project, with a fresh recorder per test.
Tests call ordinary application methods and make ordinary assertions. The plugin
records fixture setup too, so a function-scoped fixture can seed the inventory
before the test body. `prs_events` is optional; one test uses it to assert that
rejected requests emitted no reservation event.

Checks run when the test body ends, before fixture teardown. Complete any
spec-relevant work in setup or the body. Session-scoped application state would
outlive a test's recorder: use fresh state per test for this setup. “Automatic”
means checking each test, not combining unrelated tests into one event history.
Tests that already fail are not checked again.

For example, if a regression allows two reservations of 2 units against 3 units
of stock, a test that completes those calls fails with a report like:

```text
prs.SpecViolation: P spec InventoryCorrect was broken: reservation exceeds available stock

  Events (key "coffee"):
      1  eStocked   units=3
      2  eReserved  order=order-1 units=2
      3  eReserved  order=order-2 units=2  <- broke the spec here
```

The full report includes the application emit location, the P assertion location,
and a folder containing the events and checker output. A spec failure makes pytest
(and CI) exit nonzero. The explicit project reports the same failure when its
`check_spec` block exits.

## Generated histories with Hypothesis

Read [hypothesis/pyproject.toml](hypothesis/pyproject.toml), then
[hypothesis/test_inventory.py](hypothesis/test_inventory.py). Hypothesis chooses
sequences of reservations, releases, and rejected requests against the real
`Inventory` service. P checks the emitted history; an ordinary invariant also
checks the service's returned stock against successful operations. This catches
state errors even if instrumentation omits an event.

The key setup difference is the **lifetime of a generated history**. A single
pytest test runs many Hypothesis examples, including retries while shrinking a
failure. `RuleBasedStateMachine.__init__` enters a fresh `check_spec` context,
then `@initialize` creates fresh application state and seeds products **inside**
that context. Hypothesis calls `teardown` even if initialization fails, so a setup
exception cannot leave recording active. `teardown` exits the context and checks
the history before another machine starts. In a real
project, create a clean database namespace or reset your service at this same
boundary, and stop workers before exiting the recording context.

Automatic pytest checking is disabled here: one recorder around the whole pytest
test would concatenate unrelated histories and report duplicate stock setup.
Do not use a function-scoped pytest fixture to own the inventory or recorder;
Hypothesis reuses that fixture across generated histories. The state machine owns
both so every replay and shrinking attempt begins in the same state.

CI explores 30 histories of up to 25 operations. `deadline=None` accommodates
checker process startup on different machines; the example and operation limits
bound routine exploration. Open reservations are valid at the end of a history.
When a check fails, `SpecViolation` is an assertion failure that Hypothesis can
shrink to a short sequence, printed as a failing state-machine example including
the drawn SKUs, quantities, and reservations. Pytest exits nonzero.

Hypothesis saves failures in `.hypothesis/examples` and replays them on the next
run from the same directory. Keep that directory locally while investigating;
it is not committed. To reproduce a particular random run, pass a seed:

```sh
(cd examples/python/hypothesis && python -m pytest --hypothesis-seed=12345)
```

For longer exploration, raise `max_examples` and `stateful_step_count` in the
test's `settings`. This is sequential property-based testing, so it explores
operation sequences rather than concurrent thread schedules.

## Application boundary and spec

Application code imports only `emit`. Successful stock changes emit events after
updating state; rejected operations emit nothing. In a database-backed service,
emit after the transaction commits. Outside recording, `emit` does nothing.

Each SKU is an event `key`, giving it a separate monitor instance. The spec
reconstructs available stock and outstanding reservations from events, rejects
overselling and duplicate reservations, and checks that a release returns exactly
the reserved quantity. Ordinary assertions also check the application's returned
state: a monitor cannot detect a state change the application never emits.

The tests exercise cancellation, rejected reservations, and independent products.
Reservations may remain open at the end of a test, so this spec has only safety
rules. These checks cover event orderings the Python tests actually execute; they
do not explore other schedules.

See [the Python integration reference](../../docs/python.md) for selective marks,
liveness checks, subprocesses, and failure-report details.

The automatic pytest project enables `--prs-coverage=coverage/model.json` in its
pytest options. The terminal summary prints P model state and handler coverage even
when pytest captures test output; the JSON contains the union of completed checks,
including checks that find violations. Each test still gets fresh monitor state.
Use the same option in your own project; leave it out to keep coverage disabled.

Suite aggregation uses the core `prs coverage merge-observe --stdin --json-report PATH`
command. The adapter retains complete observation reports and sends one JSON array
at suite end. Core validates and unions coverage identities, preserves metadata
from failing checks, and writes the same report format across languages. A merge
or report-write failure fails the test process.


The global test setup opts into strict observation: every checked test must emit
at least one relevant event for each selected spec. This catches accidentally
disabled instrumentation; it does not infer arbitrary missing events.
