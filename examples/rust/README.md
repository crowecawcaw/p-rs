# Rust: inventory reservations

Three Cargo integration-test targets exercise one real inventory service against
one P safety spec. They illustrate how to connect a project's existing tests to a
monitor, rather than how to write a P model.

## Run from this checkout

With a stable Rust toolchain installed, from the repository root:

```sh
cargo test --manifest-path examples/rust/Cargo.toml --locked
```

Or run a setup on its own:

```sh
cargo test --manifest-path examples/rust/Cargo.toml --locked --test explicit
cargo test --manifest-path examples/rust/Cargo.toml --locked --test checked_fixture
cargo test --manifest-path examples/rust/Cargo.toml --locked --test generated_sequences
```

This is an independent Cargo workspace with its own lockfile. Its only integration
helper dev-dependency is [`prs-test`](../../crates/prs-test/README.md), plus `proptest`
for generated cases. The helper owns source compilation, recording, monitor execution,
diagnostics, and coverage union. These crates currently ship from source; the published
`p-rs` crate is a placeholder. In another repository, pin a p-rs Git revision for
`prs-test`, or vendor it and use paths. The application itself has no third-party
dependencies. No `prs` binary, external services, Python, clock delays, or network
access are needed after Cargo has fetched dependencies.
The [CI workflow](../../.github/workflows/rust-examples.yml) formats, lints, and runs
all three targets against source.

## Application boundary

Read [src/lib.rs](src/lib.rs) first. `Inventory` takes a typed `FnMut(Event)` sink.
Each successful operation changes state and then publishes its domain event;
rejected operations publish nothing. Existing projects can adapt their domain-event
bus or transaction outbox instead of adding P calls throughout their service. For a
database-backed project, publish after commit. A production sink could write logs;
a caller that wants no recording can pass `|_| {}`.

The [test support](tests/support/mod.rs) only maps application domain events to JSON
payloads and sets up the inventory fixture. `prs-test` buffers events and checks them in
process. The SKU is the partition key, so each product gets independent monitor state.
A shared `Model` semantically compiles the spec once; every recording starts fresh
monitors (interpreter bytecode preparation currently repeats). The helper is strict by
default and rejects missing instrumentation, event-name mistakes, and bad payloads.

The cloned event sink works in thread callbacks. `emit` assigns epoch milliseconds
while inserting each event under a lock; `emit_at` allows deterministic timestamps.
Join worker threads and finish cleanup before `check`, which closes the recording.
No checks occur in `Drop` or while an application panic unwinds. A monitor failure
includes readable diagnostics, replay, and the recorded JSON event history. The copied
[spec](specs/inventory.p) is the same inventory contract as the Python example, with
no model machines or test drivers. It allows outstanding reservations at the end and
needs only safety checks.
Ordinary assertions still matter: a monitor cannot detect state changes that the
application never emits.

## 1. Explicit checks for selected tests

[tests/explicit.rs](tests/explicit.rs) constructs a `model().recording()` and inventory, seeds
stock while recording, runs the operation sequence, and calls `recorder.check()`.
The result is unwrapped so a violation fails Cargo tests and CI. Rejected requests
are also checked to emit no events. A passing negative control deliberately records
an overselling event and requires the specific P assertion, proving the monitor
is active without leaving a broken test in CI.

## 2. A shared checked fixture for ordinary Cargo tests

[tests/checked_fixture.rs](tests/checked_fixture.rs) wraps each ordinary `#[test]`
body in `with_inventory`. That helper connects the recorder **before** seeding,
creates fresh application state for each test, and checks after the closure returns.
Tests use normal inventory methods and normal Rust assertions.

Rust's built-in test harness has no pytest-style global fixture plugin. Checking
applies to tests that explicitly adopt this helper. Add it to your project's
existing fixture factory so a new test has one setup call. Avoid global recorders:
Cargo runs tests concurrently, and combining histories would mix unrelated state.
The helper provides a synchronized cloneable sink for thread callbacks. A threaded
application must coordinate operation/event order and join workers before checking.

A P violation panics in the helper after the body, propagating as a normal failed
Cargo test. A body that already panics stays failed. No checking runs in `Drop`,
which avoids a second panic during unwinding. Perform monitor-relevant cleanup
inside the closure before it returns.

## 3. Generated stateful sequences with proptest

[tests/generated_sequences.rs](tests/generated_sequences.rs) generates vectors of
reserve/release commands and executes them against the real service. Repeated order
IDs, invalid quantities, missing releases, and both SKUs exercise successful and
rejected paths. This uses ordinary `proptest` strategies; no separate state-machine
framework is required. P reconstructs accepted stock/reservation history independently.

The recorder and inventory are created **inside the property body**, so each case
and every shrink attempt starts with its own stock and monitor. A monitor error is
returned through `prop_assert!`, allowing proptest to shrink the command sequence
and show the smallest failing input it finds. A test-scoped recorder around the
whole property runner would incorrectly combine multiple generated executions.

CI runs 64 cases, each with at most 40 commands, and allows 2,048 shrink iterations.
Generated inputs vary between runs. Proptest prints the minimized input and persists
failure seeds beside the test in `tests/generated_sequences.proptest-regressions`;
commit that generated regression file when fixing a discovered bug so subsequent
runs replay it. To reproduce a failure directly, reuse the printed command vector
in a normal checked test. For longer local exploration, increase `cases` in the test's `ProptestConfig`.

Proptest fits domain operations well because it shrinks operation sequences and
runs under Cargo with the other integration tests. A byte-oriented `cargo-fuzz`
target is more useful for parsers/codecs. These tests explore generated sequential
calls; they do not explore concurrent schedules.

The checked fixture opts into suite coverage by adding each completed successful or
failed check to `prs_test::Suite`.
The shared report format retains observation errors and checks source compatibility.
It writes `target/coverage/model.json` under a mutex and prints the cumulative union (`cargo test -- --nocapture` shows it).
The final write contains all completed fixture checks; a panic before checking
leaves partial coverage. Explicit and generated tests do not join this report.

The helper uses strict observation by default: every checked test must emit
at least one relevant event for each selected spec. This catches accidentally
disabled instrumentation; it does not infer arbitrary missing events.
