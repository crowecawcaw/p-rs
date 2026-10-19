# Go test integration

`github.com/crowecawcaw/p-rs/go/prstest` is the supported, standard-library-only
Go helper for checking real application events against P monitors. It invokes the
separately installed `prs` CLI for observation and does not bundle an executable.
Go 1.22+ is required.

Install the checker with Rust:

```sh
cargo install --git https://github.com/crowecawcaw/p-rs prs-cli
```

This module is currently consumed from source. From a project alongside a cloned
`p-rs` checkout, add:

```go
require github.com/crowecawcaw/p-rs/go v0.0.0
replace github.com/crowecawcaw/p-rs/go => ../p-rs/go
```

Adjust the replacement path to your checkout. Keep the CLI and module from the
same revision: the helper requires strict observation and structured coverage
support. See the runnable [inventory examples](../examples/go/README.md), whose
module replaces this dependency with `../../go`.

## Per-test recording

```go
import "github.com/crowecawcaw/p-rs/go/prstest"

func TestReservation(t *testing.T) {
    recorder := prstest.Start(t, prstest.Config{
        SpecFiles: []string{"specs/inventory.p"},
        Specs:     []string{"InventoryCorrect"},
    })
    app := NewInventory(recorder.Emit)
    // Seed and exercise the application after starting recording.
    // Successful changes call emit(eventName, productKey, payload).
}
```

Each test, subtest, or fuzz callback gets its own recorder and fresh application
state. Go has no automatic global per-test hook: put this setup in your fixture
and call it from participating tests. A recorder is safe for concurrent emissions;
your application must synchronize state changes and their emissions itself.
Emit only committed changes, and join all workers before checking. `Start`
registers a `t.Cleanup` check even when the test body fails. Register application
shutdown/join cleanups **after** `Start`, so Go runs them before checking.

For selected tests or generated cases with explicit boundaries, use
`recorder := prstest.New(t, cfg)` and `recorder.Check(t)` after operations finish.
`Verify(context.Background())` instead returns `(Result, error)` for callers that
need to inspect expected failures. Check its error; a successful coverage merge
never substitutes for successful verification.

Verification seals the trace, runs the checker once, and caches the result.
Repeated verification returns a copy of the report and does not duplicate suite
coverage. Emitting after sealing sets a sticky lifecycle error; later verification
and `Start`'s cleanup fail. Work continuing after the final cleanup cannot be
observed: completing application work is the caller's responsibility.

## Events, paths, and strict checking

`Emit(event, key string, payload any)` snapshots payload JSON immediately.
Supported payloads include nil for unit events, scalars, slices for positional
tuples, and maps/structs for named tuples. JSON field names must match the P event
declaration; use struct tags where needed. Encoding errors are retained and fail
verification rather than panic or silently drop events. Callers must not mutate
payloads concurrently with the `Emit` call; subsequent mutation is safe.

`Emit` samples actual Unix milliseconds while holding the emission lock. Equal
times retain file order; clock changes are not clamped or hidden. For event times
known by the application, `EmitAt(at, event, key, payload)` writes an ISO timestamp
with nanosecond precision. Core observation sorts events by timestamp, so the
recording time and supplied event times must reflect the order you want checked.
There are no sleeps or synthetic time advances in the helper.

Strict checking is enabled by default: every selected monitor must check a relevant
event. `AllowUnobserved: true` explicitly opts out, for tests intentionally allowing
empty or irrelevant observations. `CheckHot: true` additionally rejects monitors
left in hot states at completion. These policies do not prove instrumentation is
complete; ordinary state assertions and suitable P events remain necessary.

`SpecFiles` and `Specs` must both be nonempty. Relative spec paths and artifact
roots resolve against `WorkDir`, defaulting to the construction-time working
directory. Go runs each test package in its package directory; configure paths
accordingly. The executable resolves from `Config.Binary`, then `PRS_BIN`, then
`prs` on `PATH`. Relative executable paths containing a separator also resolve
against `WorkDir`. Set `PRS_BIN` to an absolute path in CI.

`Timeout` defaults to 30 seconds; negative values are invalid. Context cancellation
also stops `Verify`. Command failure, missing/malformed reports, missing selected
specs, parser errors, and inconsistent status/metrics all fail verification.
The helper checks report membership and required fields without assuming every
monitor observes every emitted event or product key.

## Diagnostics and observed monitor coverage

`Result` contains the complete raw observation report, checker output, and artifact
directory, including on failure. Verification errors include the command, checker
output, report, and recorded JSONL. Default files live in `t.TempDir` and disappear
after cleanup. Set `ArtifactRoot` to retain uniquely named directories for CI
uploads; parallel tests and repeated subtests do not overwrite each other's files.
Reports may contain application data: choose an appropriate artifact location.

Coverage collection is explicit and owned by a suite, never a package global:

```go
var coverage *prstest.Suite

func TestMain(m *testing.M) {
    var err error
    coverage, err = prstest.NewSuite(prstest.SuiteConfig{})
    if err != nil { fmt.Fprintln(os.Stderr, err); os.Exit(1) }
    code := m.Run()
    if err := coverage.WriteCoverage(context.Background(), "coverage/model.json", os.Stdout); err != nil {
        fmt.Fprintln(os.Stderr, err)
        code = 1
    }
    os.Exit(code)
}
```

Attach that collector with `cfg.Suite = coverage` in participating ordinary tests.
After `m.Run`, `WriteCoverage` snapshots the reports and calls core
`coverage merge-observe`, writing human output to the supplied writer and JSON to
the configured path. Core owns schema validation, source fingerprints, and set
unions. Failed observations contribute attempted-handler and failure metadata;
merge success means only aggregation succeeded. Preserve `m.Run`'s failure status.
The helper validates the suite envelope and publishes JSON through a temporary
file beside the destination; failed merges preserve the previous report.
A collector retains complete reports, so leave `Suite` nil in long fuzz campaigns.
Each Go package runs in its own test process; cross-package aggregation requires
merging their reports explicitly through the core CLI.

## Develop and verify

From the repository root:

```sh
cargo build -p prs-cli --bin prs --locked
export PRS_BIN="$(pwd)/target/debug/prs"
(cd go && go test -race -count=1 ./... && go vet ./...)
(cd examples/go && go test -race -count=1 ./...)
(cd examples/go && go test ./fuzz -run='^$' -fuzz=FuzzInventory -fuzztime=5s -parallel=2)
```

Unit tests use subprocess fixtures for malformed tools and timeout behavior.
Integration tests run the real CLI; they skip when no CLI is configured or found,
and fail if an explicitly supplied `PRS_BIN` is missing or broken. CI supplies
this checkout's CLI and tests Go 1.22 and stable.
