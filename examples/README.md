# Application integration examples

Every example checks one inventory service against one P spec,
[`model/PSpec/inventory.p`](model/PSpec/inventory.p), in two places:

1. **The design.** [`model/`](model/README.md) is a P model of the service. `prs check`
   explores every interleaving of concurrent orders and shows that the spec catches an
   overselling design that reads stock before reserving it.
2. **The implementation.** The language examples check events from real application
   code during ordinary tests, against the same spec.

The language examples need only the spec, not the model: you can check an
implementation against a spec without writing a model first. Each one's
`specs/inventory.p` is a symbolic link to the shared spec. On Windows, clone with
`git clone -c core.symlinks=true` and Developer Mode on, or use WSL.

Start with the [P model](model/README.md), then the
[Python examples](python/README.md). [Rust](rust/README.md), [Java](java/README.md)
and [Go](go/README.md) show the same three setups:

| Setup | Python | Rust | Java | Go |
| --- | --- | --- | --- | --- |
| Explicit check in selected tests | `check_spec` | `model().recording()` | `PrsObserver` | `prstest.New` |
| Every test, via a fixture or plugin | pytest plugin | `with_inventory` | `PrsExtension` | `t.Cleanup` fixture |
| Generated operation sequences | Hypothesis | proptest | jqwik | native fuzzing |

Each fixture stocks coffee=3 and tea=2.

## How the checks work

- **The application emits after commit.** A successful change emits its event after
  updating state (in a database service, after the transaction commits). Rejected
  operations emit nothing. Outside a test recording, emitting does nothing.
- **Each SKU is an event key**, so each product gets its own monitor instance. The
  spec rebuilds stock and reservations from events, rejects overselling and duplicate
  reservations, and checks that a release returns the reserved quantity.
  Reservations may stay open, so there are only safety checks.
- **Start recording before setup.** The monitor needs the initial `eStocked` events,
  so every example seeds stock after recording starts.
- **One recording per history.** Each test, generated case and shrink attempt gets
  fresh application state and a fresh recorder. A recorder shared across them would
  merge unrelated histories.
- **The same calls feed production.** With a production sink set, the same emits
  write a log that `prs observe` checks; the [Go example](go/README.md#5-production-logs-with-prs-observe)
  does it in CI. See [Testing and production](../docs/production.md).
- **Ordinary assertions still matter.** P sees only emitted events, so the tests also
  assert on the application's own state.
- **Strict observation** is on: a checked test must emit at least one relevant event
  for each selected spec. This catches disabled instrumentation, not every missing
  event.
- **A negative control** in each explicit example feeds the spec an overselling
  trace and passes only if the spec rejects it, proving the monitor is active.
- **Generated sequences explore operation orders, not thread schedules.** The P
  model is where interleavings are explored.

## Coverage

Each language's automatic/checked fixture example reports monitor coverage as
human-readable stdout and a JSON artifact uploaded by CI. These measure monitor
states and event handlers, not application source lines. Coverage is the union of
observed monitor identities across isolated tests and product keys; percentages are
never summed. Observed transitions are listed without a denominator because P
can choose goto targets dynamically. Declared but unreachable states/handlers may
remain uncovered, so this is evidence of execution rather than a proof of safety.

Suite reports are merged by core `prs coverage merge-observe`, which keeps
coverage from failing checks and writes the same format in every language.

## Across processes

For live observation across service boundaries, see [two inventory workers in
Docker](multiprocess/docker/README.md) or [two local OS
processes](multiprocess/native/README.md): both publish committed events, one
observer checks the merged history, and the supervisor cancels producers on a
violation. Both examples use the same application and monitor.
