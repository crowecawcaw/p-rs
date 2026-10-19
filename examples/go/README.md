# Go: inventory reservations

Four test packages check one [inventory service](inventory/inventory.go) against
the [P spec](specs/inventory.p) with the [Go helper](../../go/README.md), which runs
the `prs observe` CLI. See the [shared overview](../README.md#how-the-checks-work)
for what the checks do and why.

## Run

From the repository root, with Rust and Go 1.22+:

```sh
cargo build -p prs-cli --bin prs --locked
export PRS_BIN="$(pwd)/target/debug/prs"
cd examples/go
go test -count=1 ./...
go test ./fuzz -run='^$' -fuzz=FuzzInventory -fuzztime=5s -parallel=2
```

`PRS_BIN` must be absolute (or a name on `PATH`) because Go runs each package from
its own directory. A missing binary fails the tests rather than skipping them. The
[CI workflow](../../.github/workflows/go-examples.yml) runs the tests, the fuzz seeds
and a bounded fuzz campaign.

The service takes an injected `Emit` func, so it has no test dependency: tests pass a
recording's `Emit`, and [cmd/inventory](cmd/inventory/main.go) passes `prsemit.Emit`.
[internal/fixture](internal/fixture/fixture.go) holds the spec configuration.

## 1. Explicit checks in selected tests

[explicit/inventory_test.go](explicit/inventory_test.go) creates a recorder with
`fixture.New(t)`, passes `r.Emit` to `inventory.New`, seeds stock, runs operations,
and calls `r.Check(t)`. The negative control emits an overselling trace and calls
`r.ExpectViolation(t, "InventoryCorrect", "exceeds available stock")`.

## 2. A checked fixture for ordinary tests

[checked/inventory_test.go](checked/inventory_test.go) has a `stocked(t)` fixture
that starts recording with a `t.Cleanup` check, then seeds fresh stock.

**Gotcha: cleanup order.** Go has no global per-test hook, so each test calls the
fixture; a `TestMain` recorder would merge tests. Cleanups run in reverse order:
start recording first, then register worker shutdown, so workers stop before the
check. In a concurrent service, emit inside the same critical section as the commit
so the recorded order matches.

`TestMain` is `prstest.Main(m)`: after the tests it prints
[coverage](../README.md#coverage) and writes `checked/coverage/monitor.json`
(`go test -v` shows it). Delete the file before a run if the spec changed.

## 3. Native fuzzing of action sequences

[fuzz/inventory_test.go](fuzz/inventory_test.go) turns fuzzed bytes into up to 32
reserve/release actions across two products and four order IDs, including rejected
ones.

**Gotcha:** every fuzz callback, including minimized replays, builds a fresh
inventory and recorder with `fixture.Checked`, which leaves out suite coverage.
Each case starts the CLI, so throughput is lower than an in-process property checker;
`-parallel=2` bounds worker processes. Go prints a `-run=FuzzInventory/<hash>`
command to reproduce a failure; commit useful inputs from
`fuzz/testdata/fuzz/FuzzInventory/`.

## 4. Events from processes the test starts

[process/inventory_test.go](process/inventory_test.go) builds
[cmd/inventory](cmd/inventory/main.go), a program that reads operations from stdin and
emits with `prsemit`. `TestWorkerProcesses` starts two of them with `cmd.Env =
r.Env()`, waits for them, and the cleanup checks their events. `TestProductionLog`
runs it the production way, writing a log, and checks the log with `r.AddLog`. The
same log checks from the command line:

```sh
printf 'stock coffee 3\nreserve coffee order-1 2\n' | go run ./cmd/inventory -events events.jsonl
prs observe -pf specs/inventory.p --spec InventoryCorrect -l events.jsonl
```

## Adapt to your project

Inject `Recorder.Emit` (tests) and `prsemit.Emit` (programs) at your
committed-change boundary, and set `SpecFiles` (relative to each test package) and
`Specs`. See the [helper setup](../../go/README.md) for the `replace` directive this
example's `go.mod` uses, and the [observation reference](../../docs/observe.md) for
the event and report formats.
