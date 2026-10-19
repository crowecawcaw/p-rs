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
[Python inventory examples](python/README.md): one application and spec, with
explicit per-test checks, automatic pytest checks, and generated operation sequences
using Hypothesis stateful testing. [Rust](rust/README.md), [Java](java/README.md) and
[Go](go/README.md) show the same three setups.

Each language's automatic/checked fixture example reports P model coverage as
human-readable stdout and a JSON artifact uploaded by CI. These measure monitor
states and event handlers, not application source lines. Coverage is the union of
executed model identities across isolated tests and product keys; percentages are
never summed. Observed transitions are listed without a denominator because P
can choose goto targets dynamically. Declared but unreachable states/handlers may
remain uncovered, so this is evidence of execution rather than a proof of safety.

For live observation across service boundaries, see [two inventory workers in
Docker](multiprocess/docker/README.md) or [two local OS
processes](multiprocess/native/README.md): both publish committed events, one
observer checks the merged history, and the supervisor cancels producers on a
violation. Both examples use the same application and monitor.
