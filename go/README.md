# Go helper

Module `github.com/crowecawcaw/snappea` (this repository's root), Go 1.22+, with two
standard-library-only packages:

- [`github.com/crowecawcaw/snappea/go/snappeatest`](snappeatest): check events from Go tests
  against P specs. It runs the `snappea` binary.
- [`github.com/crowecawcaw/snappea/go/snappeaemit`](snappeaemit): emit events from application
  code, in the test process or in processes a test starts; in production, write them
  as a log for `snappea observe`.

Both follow the [language helper requirements](../docs/language-support.md). The
runnable [inventory example](../examples/go/README.md) uses everything below.

## Setup

```sh
go get github.com/crowecawcaw/snappea
```

`snappeatest` runs the `snappea` checker: `SNAPPEA_BIN`, else `snappea` on `PATH`, else the release
binary of the module's version, downloaded once into the user cache
(`os.UserCacheDir()/snappea/<version>/`) and verified against checksums committed in the
module. Offline, or with `SNAPPEA_DOWNLOAD=0`, install it yourself (`pip install pysnappea`
also brings it):

```sh
cargo install snappea
```

To use a checkout instead of a release, `replace` the module and build `snappea` from the
same revision (a checkout downloads nothing):

```go
require github.com/crowecawcaw/snappea v0.0.0
replace github.com/crowecawcaw/snappea => ../snappea
```

## Check a test's events

```go
func TestReservation(t *testing.T) {
    r := snappeatest.Start(t, snappeatest.Config{
        SpecFiles: []string{"../specs/inventory.p"},
        Specs:     []string{"InventoryCorrect"},
    })
    app := inventory.New(r.Emit) // the application takes an emit func
    // seed and exercise app; the events are checked when the test ends
}
```

- `Start` checks in a `t.Cleanup`. Register cleanups that stop workers **after**
  `Start`, so they run first. A test that already failed is not checked.
- `New` plus `r.Check(t)` checks where you call it; `r.Verify(ctx)` returns
  `(Result, error)` instead.
- Each test, subtest and fuzz callback needs its own recording (call `Start` inside
  `f.Fuzz` with its `t`). Go has no global per-test hook, so put `Start` in your
  fixture.
- `Emit(event, key, payload)` / `EmitAt(time, event, key, payload)`: `payload` is a
  struct or map whose JSON names match the P named tuple, a single value, a slice for
  a tuple, or nil. It is encoded at once. The call site is recorded for reports.
- Emitting after the check fails it: join workers and stop child processes first.

## Errors

`Verify` returns, and `Check` reports with `t.Errorf`:

- `*snappeatest.SpecViolation`: `Spec`, `Key`, `Message`, `Location` (the P assert),
  `Events` (the key's last events with `Event`, `Payload`, `Src`), `Liveness`,
  `Others`, `ArtifactsDir`.
- `*snappeatest.SetupError`: `Kind` is `binary_missing`, `config`, `parser`, `validation`
  (strict found no event for a spec), `unknown_event`, `out_of_order` or `timeout`.

```go
var v *snappeatest.SpecViolation
if errors.As(err, &v) && v.Key == "coffee" { ... }
```

A failure prints the spec's message, the key's events with where each was emitted,
the P assert and the folder that keeps the events, snappea output and report:

```
P spec InventoryCorrect was broken: reservation exceeds available stock

  Events (key "coffee"):
      1  eStocked   units=3                inventory.go:32
      2  eReserved  order=order-1 units=2  inventory.go:46
      3  eReserved  order=order-2 units=2  inventory.go:46  <- broke the spec here

  Spec: ../specs/inventory.p:27
  snappea output: /tmp/snappeatest-TestOversell-3765897326
```

Negative control: `r.ExpectViolation(t, "InventoryCorrect", "exceeds available")`
checks now and fails the test unless that spec (and only it) is violated with a
message matching the regular expression. It returns the `*SpecViolation`; later
checks of `r` ignore that spec.

## Options

| `Config` field | Environment | Default |
| --- | --- | --- |
| `SpecFiles`: files, folders (every `.p`) or globs | `SNAPPEA_SPEC_FILES` (separated like `PATH`) | required |
| `Specs` | | every spec in `SpecFiles` |
| `Exclude` | | none |
| `Strict` (`snappeatest.Bool(false)`) | | true when `Specs` is set, else false |
| `CheckHot`: fail a key left in a hot state | | false |
| `UnknownEvents`: `ignore`, `warn`, `error` | | `error` |
| `Timeout` | | 30 s |
| `Artifacts`: where failed checks are kept | `SNAPPEA_ARTIFACTS` | a temp folder |
| `Coverage`: suite coverage JSON | `SNAPPEA_COVERAGE` | off |
| `Mode`: `auto`/`marked` (same in Go) or `off` | `SNAPPEA_MODE` | `auto` |
| `Binary` | `SNAPPEA_BIN`, then `snappea` on `PATH` | the release binary, downloaded (`SNAPPEA_DOWNLOAD=0`: off) |

Environment variables override fields, except `SNAPPEA_BIN`, which `Binary` overrides.
`Config` paths are relative to `WorkDir` (default: the package folder `go test` runs
in); environment paths to the working directory. `SNAPPEA_MODE=off` makes recordings
inert. A missing binary is a `binary_missing` error saying how to install one.
`AllowUnobserved` and `ArtifactRoot` are deprecated aliases for
`Strict: snappeatest.Bool(false)` and `Artifacts`.

## Events from other processes

A recording has its own events folder. Start a child with `r.Env()` (the environment
plus `SNAPPEA_EVENTS_DIR`); if it uses `snappeaemit`, its events join the check. Wait for the
child before the check.

```go
cmd := exec.Command(workerBinary)
cmd.Env = r.Env()
if err := cmd.Run(); err != nil { t.Fatal(err) }
```

- `r.EventsDir()` is the folder, for anything that writes the
  [events folder format](../docs/observe.md#events-folder-format) itself.
- `r.AddEventsDir(path)` adds a folder the recording did not create (a container
  volume, a service started once for the package).
- `r.AddLog(path, parse)` adds a log; `parse(line) (snappeatest.Event, bool)` returns an
  event with `Time` set, or false to skip the line. `nil` reads JSON lines as they are.

## snappeaemit

```go
import "github.com/crowecawcaw/snappea/go/snappeaemit"

app := inventory.New(snappeaemit.Emit)
```

| Process | Events go to |
| --- | --- |
| A test process with a `snappeatest` recording | that recording |
| Started by a test (`SNAPPEA_EVENTS_DIR` set) | its own `<program>-<pid>-<random>.jsonl` in that folder, one append per line |
| After `snappeaemit.SetSink(...)` | JSON lines for `snappea observe`, written by a background goroutine |
| Otherwise | nowhere: one atomic load per `Emit` |

In a test process `snappeaemit.Emit` reaches the recording without injecting `r.Emit`,
until the test ends. It cannot tell parallel tests apart: with several recordings
active, an `Emit` fails them all with a `config` error. Pass a context instead:
`snappeaemit.EmitCtx(ctx, ...)` reaches the recording `ctx` came from
(`ctx := r.Context(context.Background())`), or inject `r.Emit`.

### Production sink

```go
sink, err := snappeaemit.File("/var/log/app/events/inventory.jsonl") // or Stdout(), Writer(w), Callback(f)
if err != nil { log.Fatal(err) }
snappeaemit.SetSink(sink)
defer snappeaemit.Close() // writes what is queued, closes the file
```

`SetSink(nil)` turns it off; `snappeaemit.Flush(timeout)` waits for the queue. In a
process a test started (`SNAPPEA_EVENTS_DIR` set), `SetSink` does nothing: events go to
the test. `Emit` never panics or blocks: a line that cannot be encoded or written, or
that finds 4096 lines already queued, is dropped and counted by `snappeaemit.Dropped()`.
Then check the log:

```sh
snappea observe specs/inventory.p '/var/log/app/events/*.jsonl' --spec InventoryCorrect
```

[Testing and production](../docs/production.md) covers deploying this.

## Suite coverage

Set `Config.Coverage` (or `SNAPPEA_COVERAGE`) and add

```go
func TestMain(m *testing.M) { snappeatest.Main(m) }
```

`Main` merges every check's observed monitor coverage into the file once, after the
tests, and prints the summary. Without `Main`, each check merges as it completes.
Either way the file is merged under a lock, so `go test ./...` packages can share it;
delete a stale file before a run. `Config.Suite` with `NewSuite` and
`Suite.WriteCoverage` remain for full control.

## Develop

```sh
cargo build -p snappea --bin snappea --locked
export SNAPPEA_BIN="$(pwd)/target/debug/snappea"
go test -race -count=1 ./... && go vet ./...   # the module is the repository root
(cd examples/go && go test -race -count=1 ./...)
```

Tests that need the checker skip when no `snappea` is found. `TestConformance` replays
[tests/conformance](../tests/conformance/README.md).
