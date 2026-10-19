# Java test integration

`prs-test-core` records events from application code and checks them with the public
`prs observe` CLI. `prs-junit5` adds a Jupiter extension that observes each test,
including its fixtures. Both require Java 17+. Core has no JUnit dependency; neither
module contains inventory logic, a P compiler, or a bundled native binary.

## Install from this repository

Use JDK 17+, Maven 3.9.9, and the repository's Rust toolchain. From the repository root:

```sh
cargo build --locked -p prs-cli
export PRS_BIN="$PWD/target/debug/prs"
mvn --batch-mode --no-transfer-progress -f java/pom.xml install
mvn --batch-mode --no-transfer-progress -f examples/java/pom.xml test
```

The reactor tests both modules against the CLI and installs their JARs and parent
POM into your local Maven repository. These snapshot coordinates are available from
source; this change does not publish them to Maven Central. In another project,
install this reactor first and add a test dependency:

```xml
<dependency>
  <groupId>io.github.crowecawcaw</groupId>
  <artifactId>prs-junit5</artifactId>
  <version>0.1.0-SNAPSHOT</version>
  <scope>test</scope>
</dependency>
```

Use `prs-test-core` instead for another test framework or a standalone harness.
Jupiter consumers still need their normal Jupiter engine/test runner. The example
POM shows Jupiter and jqwik configuration with pinned dependencies and plugins.

For a source installation of the CLI outside this checkout:

```sh
cargo install --git https://github.com/crowecawcaw/p-rs prs-cli
```

Use a CLI revision containing strict observation and observed-coverage merging;
building helper and CLI from the same checkout is the tested combination. Set
`PRS_BIN` to its absolute executable path, or put `prs` on `PATH`. The helpers never
download executables or silently drop unsupported CLI options. On PowerShell use
`$env:PRS_BIN = "C:\path\to\prs.exe"`.

## Explicit checks

```java
import io.github.crowecawcaw.prs.PrsObserver;
import java.nio.file.Path;
import java.time.Duration;

var observer = PrsObserver.builder()
    .specFiles(Path.of("specs/service.p"))
    .specs("ServiceCorrect")
    .strict(true)                         // default
    .checkHot(false)                      // independent; default false
    .timeout(Duration.ofSeconds(30))      // per CLI command
    .artifacts(Path.of("target/prs"))
    .build();

try (var events = observer.recording("reserve then release")) {
    // Construct fixtures here and inject an event sink into your service.
    events.emit("eReserved", "product-17", java.util.Map.of("order", "order-42", "units", 2));
    // Exercise the real service and keep normal application assertions.
    // Complete application/fixture cleanup before closing the recording.
}
```

Adapt the spec files and names to your project. Relative spec/artifact paths are
resolved against the JVM working directory when the immutable configuration is
built. `.binary("/absolute/path/to/prs")` overrides `PRS_BIN`; a bare executable
name uses `PATH`. It is one executable, never a shell command with arguments.

Emit after a successful commit, at the same domain boundary in production and
tests. A rejected operation emits no change event. Use stable resource IDs as
partition keys, and inject the `Recording` through your application's own sink
interface, as shown in the [inventory examples](../examples/java/README.md).

`emit(event, key, payload)` accepts JSON-compatible values: strings, booleans,
finite standard Java numbers, `BigInteger`/`BigDecimal`, lists, maps with string
keys, and null. Nested values follow the same rules; custom POJOs, arrays, cycles,
and nesting beyond 256 levels are rejected. Match the payload to the declared P
event type. `emit(event, key)` emits a payload of null for payload-free events.

Each emission validates and snapshots JSON before appending a complete line under
one recording lock. Default timestamps are actual epoch milliseconds sampled in
that lock. `emitAt(Instant, event, key, payload)` preserves a supplied timestamp,
including sub-millisecond precision. Supply application timestamps when checking
latency assertions; do not substitute synthetic operation counters for real time.
The batch CLI sorts timestamps; callers remain responsible for recording the
actual commit order and for handling clock changes appropriately.

A recording may receive events from several threads, but thread safety does not
establish your application's commit order. Emit at the commit boundary and join
workers before close. Mutating a payload after `emit` returns is safe; concurrent
mutation during emission requires caller synchronization. Each recording has its
own unique directory and fresh monitor execution. Never share one recording
between independently generated cases or shrink attempts.

Closing checks the recorded log once. Further close calls are no-ops; emissions
after close fail. Serialization/write errors are sticky: catching an immediate
emission exception does not let close succeed. A valid prefix can still contribute
coverage, tagged incomplete with the recording error. Explicit try-with-resources
preserves a body exception and adds close failures as suppressed exceptions.

## Automatic Jupiter checks

```java
import io.github.crowecawcaw.prs.PrsExtension;
import io.github.crowecawcaw.prs.Recording;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.extension.RegisterExtension;

@RegisterExtension
static final PrsExtension PRS = PrsExtension.builder()
    .specFiles(Path.of("specs/service.p"))
    .specs("ServiceCorrect")
    .coverage(Path.of("target/coverage/model.json")) // optional
    .build();

@BeforeEach
void setup(Recording events) {
    // Construct and stock the real service using an injected event sink.
}
```

A fresh recording opens before `@BeforeEach` and checks after `@AfterEach` for each
Jupiter invocation, including repeated and parameterized tests. Inject it into
fixture or test methods. Constructor/`@BeforeAll` injection has no per-test scope
and is rejected. The extension holds immutable configuration, with recording state
in invocation stores, so a static extension supports parallel tests.

Test, setup, and teardown failures remain primary; observation failures are
attached as suppressed exceptions. An observation failure accompanying an aborted
test is reported as a failure so a real checker error cannot disappear as a skip.
Ordering relative to other extensions follows Jupiter's normal callback ordering;
use `@Order` when their resource lifetimes must nest inside observation.

To register multiple extensions, give each a distinct `.name("billing")` and
qualify parameters with `@PrsRecording("billing") Recording events`. Unqualified
parameters with competing resolvers produce Jupiter's ambiguity error. Recordings
and namespaces remain independent, including in parallel execution.

Jupiter lifecycle callbacks do **not** surround individual jqwik property trials.
Use explicit `observer.recording(...)` inside each property invocation. The
inventory example shows fresh application/reference state for every trial and
shrink candidate, with an `@AfterContainer` coverage flush.

## Failures and strict observation

Strict observation is enabled by default. Every selected spec must successfully
check at least one relevant event; `.strict(false)` explicitly opts out. It does
not require every emitted event or key to be relevant to every spec, and it cannot
detect arbitrary missing instrumentation. Keep state assertions and explicit
completion events where your project requires stronger guarantees. `.checkHot(true)`
separately checks unfinished hot states after a recording ends.

Failures throw `ObservationFailure`, whose `artifacts()` points to retained JSONL,
console output, the raw/annotated JSON report, and replay logs when produced.
`report()` contains an immutable `ObservationReport` when a valid report exists;
`exitCode()`, `errors()`, and each error's replay `events()` provide structured
access. `json()` retains the full report, coverage, and source metadata. Transport
or malformed-report failures can have no parsed report. Successful artifacts are
deleted after collection. An expected-negative test retains its failure artifacts.

The helper checks required report types, requested spec identities, known error
kinds, consistent nonnegative metrics, and agreement between process status, report
status, and errors. A successful report must verify every reported partition;
strict success requires a positive relevant-event count for each selected spec. The native CLI's process status differs from its PObserve report
status: a safety violation is process 1 and report 2. The helper preserves these
rather than treating report 2 as a native process-code requirement.

Processes use argument arrays and file redirection, not shell interpolation or
undrained output pipes. Timeouts and interruption trigger forced termination of
the command and known descendants with bounded reaping; interruption restores the
Java interrupt flag. Diagnostics include the console tail; complete output remains
in the artifact directory on failure. These are batch checks at close, not live
fail-fast supervision of application worker processes.

## Suite coverage

Coverage is opt-in. Without a destination/collector, recordings do not request it.
The Jupiter `.coverage(path)` option owns a root-scoped collector closed after all
its tests. Extensions sharing a normalized destination share the collector;
conflicting binary, timeout, or artifact settings fail explicitly. Different spec
sets can share a destination; core validates compilation fingerprints and declarations.

For another framework, explicitly own the coverage lifetime:

```java
try (var coverage = observer.coverage(Path.of("target/coverage/generated.json"))) {
    for (var input : generatedInputs) {
        try (var events = observer.recording("generated " + input, coverage)) {
            // Fresh real application and fixtures for this input.
        }
    }
}
```

The collector spools full reports to disk, including valid reports from failed
checks, and invokes `prs coverage merge-observe --stdin` once at close. Core owns
schema validation, fingerprint compatibility, set union, and human/JSON reporting.
Source metadata includes the test label and a unique recording ID so separate
observations remain distinguishable. Coverage measures observed monitor states and
handlers; it does not measure full model paths or establish verification success.
Failed and incomplete source reports retain their metadata in the aggregate.

Merged output replaces the destination only after a successful merge; failures
retain input reports and console diagnostics and fail the suite. Existing files
are not implicitly imported, so rerunning a suite does not mix stale sources.

A collector belongs to one JVM/launcher run. Separate Maven forks/processes must
write distinct destinations, then explicitly combine them, for example:

```sh
prs coverage merge-observe target/coverage/fork-*.json --json-report target/coverage/suite.json
```

Do not use an explicit collector and an automatic collector for the same output
path concurrently. Root resources are a Jupiter feature; an explicit caller must
close its own collector, after all recordings and their cleanup finish.
