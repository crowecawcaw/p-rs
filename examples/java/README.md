# Java inventory integration

Three scenarios check one real inventory service with a P safety monitor, using
the supported [Java helpers](../../java/README.md). The
[spec](specs/inventory.p) links to the [shared spec](../model/PSpec/inventory.p) that
the [P model](../model/README.md) also checks; these tests use only the spec.

Requires JDK 17+, Maven 3.9.9, and the repository's Rust toolchain. From the root:

```sh
cargo build --locked -p prs-cli
export PRS_BIN="$PWD/target/debug/prs"
mvn --batch-mode --no-transfer-progress -f java/pom.xml install
mvn --batch-mode --no-transfer-progress -f examples/java/pom.xml test
```

The first Maven command tests and installs the source reactor into your local
repository. The example depends on `io.github.crowecawcaw:prs-junit5:0.1.0-SNAPSHOT`;
these artifacts are not published to Maven Central. CI builds the checkout's CLI,
tests/installs the helper, and runs the examples with Java 17 and Maven 3.9.9.

- `ExplicitInventoryTest` configures a `PrsObserver` and opens a recording before
  fixture construction. Try-with-resources checks selected tests at close. A
  negative control simulates a bad commit and asserts the structured
  `spec_violation` error and overselling message, proving that P is active.
- `AutomaticInventoryTest` registers `PrsExtension` with `@RegisterExtension`.
  Its injected `Recording` opens before `@BeforeEach` stocks products and reserves
  fixture inventory, then checks after `@AfterEach` releases that reservation.
  Each test gets fresh application and monitor state; product keys let the same
  order reserve different products independently. Suite coverage is printed and
  written to `target/coverage/model.json` at Jupiter root teardown.
- `GeneratedInventoryTest` uses jqwik to generate 60 sequences of 1–40 reserve and
  release operations, including invalid requests. A reference ledger asserts
  application state/rejections; P independently checks emitted history. Each
  property invocation and shrink candidate creates fresh inventory, ledger and
  recording. A supported `CoverageCollector` accumulates reports and closes in
  jqwik's `@AfterContainer`, writing `target/coverage/generated.json`.

From `examples/java`, run one scenario with
`mvn test -Dtest=GeneratedInventoryTest` (substitute the other class names). jqwik
prints its seed and smallest failing operation list and retains replay state in
`.jqwik-database`. Add `seed = "<reported seed>"` to `@Property` to reproduce a
sequence; keep the random default for normal CI exploration. Fixed trial and
sequence bounds keep routine runs small.

Jupiter callbacks do not surround jqwik trials: never move the generated test's
recording into a Jupiter fixture. Both generated state and the recording must be
created inside the property invocation so shrinking replays an independent case.

## Adapting the setup

`Inventory` has no testing dependencies. Its injected `Inventory.Events` interface
emits after a successful state change, at the boundary where a database-backed
service would commit. Rejected requests emit nothing. The tests adapt that domain
interface with a lambda:

```java
var inventory = new Inventory(
    (sku, event, payload) -> recording.emit(event, sku, payload));
```

Use the same domain boundary for your production telemetry/outbox adapter. Adapt
spec file paths and names in `PrsObserver`/`PrsExtension`, and set `PRS_BIN` to your
built/installed executable. No copied subprocess runner or framework extension is
needed. `prs-test-core` is available without Jupiter for other frameworks.

The supported recorder captures timestamped JSON snapshots, serializes concurrent
emissions, enforces a bounded CLI timeout, and fails closed for missing/malformed
reports and tool errors. Strict mode defaults to requiring a relevant event for
every selected spec. It cannot infer arbitrary missing events, so retain normal
application assertions. Concurrent services must emit in actual commit order and
join workers before recording teardown.

Failed recordings retain event JSONL, reports, console output and P replay logs
under `target/prs`. Structured errors include source labels and artifact paths;
body/setup/teardown failures preserve observation errors as suppressed exceptions.
Successful recording files are removed. The expected-negative test retains one
failure directory; `mvn clean` removes generated outputs.

Coverage is optional: the explicit examples do not collect it, while automatic and
generated tests each own a destination. Both delegate union and source-fingerprint
validation to core `prs coverage merge-observe`, preserving failed observations.
The reports describe observed monitor states/handlers, not full P model paths or
proof of correctness. To combine the two compatible suite reports:

```sh
"$PRS_BIN" coverage merge-observe target/coverage/model.json target/coverage/generated.json \
  --json-report target/coverage/suite.json
```

CI uploads coverage, diagnostics, Surefire reports and jqwik replay state. See the
[helper documentation](../../java/README.md) for explicit collector lifetimes,
multiple extensions, parallel tests and separate JVM/fork destinations.
