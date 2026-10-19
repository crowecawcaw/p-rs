# Go: inventory reservations

One small inventory service, three ways to check its events with P. These examples
use the same [InventoryCorrect spec](../model/PSpec/inventory.p) as the
[P model](../model/README.md) and the other languages. They need no external
services or sleeps.
The application has an injected event sink, which can also be backed by your
production event logger. Recording uses the supported
[`prstest` helper](../../go/README.md), which invokes
the separately installed `prs observe` CLI. The example module uses a local
`replace` directive to consume `../../go` from this checkout.

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

Use `github.com/crowecawcaw/p-rs/go/prstest` and inject its recorder's `Emit`
sink at your application's committed-change boundary. The tiny example wrapper
in `internal/prstest` supplies only inventory fixtures and explicit `SpecFiles`
and `Specs` configuration. Point those paths at your own P specs relative to each
Go test package directory (or set `Config.WorkDir`). Build or install the matching
`prs` CLI and set `PRS_BIN` to its absolute path in local test setup and CI.
See the [supported helper setup](../../go/README.md) for the source-module
`replace` directive; this example already configures it in `go.mod`.

## 1. Explicit checks in selected tests

Start at [explicit/inventory_test.go](explicit/inventory_test.go).
`prstest.New(t)` creates a recorder; inject `r.Emit` when constructing the service,
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
checking. The supported helper allocates temporary artifacts at construction and retains
the trace and checker report in failure output, then removes files through
`t.TempDir`. Set `Config.ArtifactRoot` to retain unique report directories for CI.

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

The recorder samples actual Unix milliseconds inside its emission lock; equal
timestamps retain file order. It also supports `EmitAt` for explicit event times.
The helper enables core strict observation by default, rejecting otherwise green
checks with no relevant events, and validates required report fields and the
selected spec names. Different monitors may observe different event subsets.

For example, removing the application's insufficient-stock guard makes a checked
test that reserves two lots of 2 against 3 units fail with output containing:

```text
prstest observation: exit status 1
PObserve Spec Violation::
Assertion failure: .../inventory.p:27:7 reservation exceeds available stock
...
recorded JSONL:
{"event":"eStocked","key":"coffee","payload":{"units":3},"timestamp":1700000000000}
{"event":"eReserved","key":"coffee","payload":{"order":"first","units":2},"timestamp":1700000000002}
{"event":"eReserved","key":"coffee","payload":{"order":"second","units":2},"timestamp":1700000000003}
```

The fixture stocks tea too, accounting for its omission in this per-product excerpt.
Timestamps above illustrate one possible recording; actual values come from the clock.
The complete Go failure includes checker diagnostics and the entire input trace;
`go test` exits nonzero. See [the observation reference](../../docs/observe.md)
for the JSONL schema and report format.

The checked fixture passes an explicit `Suite` to `CheckedWithCoverage` to enable
`observe --coverage`. The suite collects raw reports under a mutex. `TestMain` waits for `m.Run()` (including parallel-test cleanups),
then prints the suite summary and writes `checked/coverage/model.json` when run
from the module root. Use `go test -v` to see successful-package stdout. Explicit
and fuzz tests do not join this report. Completed observations include the executed prefix of a failing check.

Suite aggregation uses the core `prs coverage merge-observe --stdin --json-report PATH`
command. The adapter retains complete observation reports and sends one JSON array
at suite end. Core validates and unions coverage identities, preserves metadata
from failing checks, and writes the same report format across languages. A merge
or report-write failure fails the test process.

`Checked` performs observation without retaining suite coverage. Use it in fuzz
callbacks, where a long campaign should release each recording after checking.
Opt into `CheckedWithCoverage` only when the package's `TestMain` calls
`Suite.WriteCoverage` after all tests and cleanups finish.

The supported helper enables strict observation by default: every checked test must emit
at least one relevant event for each selected spec. This catches accidentally
disabled instrumentation; it does not infer arbitrary missing events.
