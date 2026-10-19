# Go: inventory reservations

One small inventory service, three ways to check its events with P. These examples
use the same [InventoryCorrect spec](../python/specs/inventory.p) as Python; there
are no P models, external services, third-party Go dependencies, or sleeps.
The application has an injected event sink, which can also be backed by your
production event logger. `prstest` is example test infrastructure using the
`prs observe` CLI, not a published Go SDK.

## Run locally

From the repository root, with Rust and Go 1.22+ installed:

```sh
cargo build -p prs-cli --bin prs --locked
export PRS_BIN="$(pwd)/target/debug/prs"
cd examples/go
go test -count=1 ./...
go test ./fuzz -run='^$' -fuzz=FuzzInventory -fuzztime=5s -parallel=2
```

`PRS_BIN` must be an absolute executable path (or an executable name on `PATH`),
because Go runs each test package from its own directory. If unset, the helper
uses `prs` on `PATH`. The [CI workflow](../../.github/workflows/go-examples.yml)
builds this checkout's CLI, runs ordinary tests (including fuzz seeds), and runs
bounded native fuzzing. A missing binary, malformed event, or checker error fails
the test rather than skipping it. After the initial toolchain/build setup, checks
need no network access.

## Adapt to your project

Copy the `prstest` helper into your test support package and inject its `Emit`
sink at your application's committed-change boundary. Update the spec path in
`Recorder.Verify`: this example resolves it from the helper's source location
using `runtime.Caller`, relative to the shared Python inventory spec. Point it at
your own checked-in P spec instead. Build or install the matching `prs` CLI and
set `PRS_BIN` to its absolute path in local test setup and CI.

## 1. Explicit checks in selected tests

Start at [explicit/inventory_test.go](explicit/inventory_test.go).
`prstest.New()` creates a recorder; inject `r.Emit` when constructing the service,
then call `r.Check(t)` after the operations to verify. Create and seed the service
*after* starting observation: the spec needs the initial stock event. Recording
lives on that service instance, not in a process-wide environment variable.
Ordinary tests can construct `inventory.New(nil)` when they need no spec checks.

`TestSpecRejectsOverselling` deliberately supplies an invalid trace and requires
the expected structured `spec_violation` and diagnostic. This passes in CI while proving
the monitor is active; a missing executable cannot satisfy it.

## 2. A reusable checked fixture for ordinary Go tests

Read [checked/inventory_test.go](checked/inventory_test.go) and
[internal/prstest/recorder.go](internal/prstest/recorder.go).
The local `stocked(t)` fixture starts observation, registers a `t.Cleanup` check,
and then seeds fresh inventory. Test bodies call ordinary application methods.
Setup events are included, and checking still runs if the body calls `t.Fatal`.
Go has no global per-test hook: every participating test must call the fixture.
A `TestMain` recorder would conflate tests and is unsuitable for this setup.

Go runs cleanups in reverse registration order. Start recording first, then
register application worker shutdown/join cleanups, so work completes before
checking. The helper creates temporary artifacts during verification, retains the
trace and checker report in failure output, then removes the files through
`t.TempDir`. To retain files as CI artifacts in your own project, replace the
temporary directory policy with a project-owned report directory.

Each parallel test has a separate service, recorder, files, and checker process.
The recorder's lock preserves emission order; the inventory service itself is
single-threaded. In a concurrent production service, serialize committed changes
and their event emissions together so observed order matches transaction order.

## 3. Native Go fuzzing of action sequences

Read [fuzz/inventory_test.go](fuzz/inventory_test.go). Go's built-in fuzz engine
mutates byte sequences into reserve/release actions against two products and four
order IDs. The seeds include releases, rejected overselling, duplicate orders,
and independent products. Inputs include zero/oversized quantities and releases
without reservations: rejected requests are expected and emit nothing.

Every fuzz callback constructs fresh inventory and a fresh recorder **before
seeding stock**. The cleanup checks that invocation's trace, including minimized
replays; state never leaks across callbacks. SKU keys create independent P monitor
instances within a sequence. Sequences are capped at 32 operations to bound
checker input, and each CLI invocation has a 30-second timeout.

`go test ./...` executes the seed corpus. `-fuzztime=5s` performs a bounded campaign
in CI; increase it locally. Each case launches the CLI and compiles the spec, so
throughput is lower than an in-process property checker. `-parallel=2` bounds
worker processes independently of CPU count. Go prints a corpus path and a
`-run=FuzzInventory/<hash>` command for reproducing failures; commit useful
`fuzz/testdata/fuzz/FuzzInventory/` regression inputs. Go's minimizer may take
additional time after finding a failure.

## Application boundary and failure reports

Successful changes emit after updating application state; rejected operations
emit nothing. In a database service, emit only after commit. The P spec
reconstructs available stock and reservations per product, rejects overselling
and duplicate reservations, and checks release quantities. Go assertions also
check application state: P cannot see mutations omitted from the event stream.
The spec allows reservations to remain open, so no end-of-sequence liveness check
is needed. Generated sequences explore inputs and order, not thread schedules.

Logical timestamps are increasing integers in one recorder, so clocks cannot
reorder events. They are suitable for these ordering rules; latency specs would
need real event times instead. Before accepting success, the helper rejects empty
recordings and checks the JSON report names `InventoryCorrect` and verifies all
recorded events and product keys. This avoids a green test that checked nothing.

For example, removing the application's insufficient-stock guard makes a checked
test that reserves two lots of 2 against 3 units fail with output containing:

```text
InventoryCorrect: exit status 1
PObserve Spec Violation::
Assertion failure: .../inventory.p:27:7 reservation exceeds available stock
...
recorded JSONL:
{"event":"eStocked","key":"coffee","payload":{"units":3},"timestamp":1}
{"event":"eReserved","key":"coffee","payload":{"order":"first","units":2},"timestamp":3}
{"event":"eReserved","key":"coffee","payload":{"order":"second","units":2},"timestamp":4}
```

The fixture stocks tea too, accounting for the gap in this per-product excerpt.
The complete Go failure includes checker diagnostics and the entire input trace;
`go test` exits nonzero. See [the observation reference](../../docs/observe.md)
for the JSONL schema and report format.
