# Application integration examples

These examples check events from real application code during ordinary tests.
They use P event declarations and spec monitors, without a P model or model checker.

Start with the [Python inventory examples](python/README.md): one application and
spec, with explicit per-test checks, automatic pytest checks, and generated
operation sequences using Hypothesis stateful testing.

Each language's automatic/checked fixture example reports P model coverage as
human-readable stdout and a JSON artifact uploaded by CI. These measure monitor
states and event handlers, not application source lines. Coverage is the union of
executed model identities across isolated tests and product keys; percentages are
never summed. Observed transitions are listed without a denominator because P
can choose goto targets dynamically. Declared but unreachable states/handlers may
remain uncovered, so this is evidence of execution rather than a proof of safety.
