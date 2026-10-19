# Java inventory integration

These are three ways to check a real application's events with P, using the same
small inventory service. The P file contains event declarations and a monitor;
there is no model of the application and no P test driver.

Requires JDK 17+, Maven 3.9.9, and the repository's Rust toolchain to build `prs`.
From the repository root:

```sh
cargo build --locked -p prs-cli
export PRS_BIN="$PWD/target/debug/prs"
cd examples/java
mvn --batch-mode --no-transfer-progress test
```

PowerShell users can set `$env:PRS_BIN` to the absolute `target/debug/prs.exe` path.
Java dependencies and Maven plugins are pinned in `pom.xml`; CI installs Maven
3.9.9 and JDK 17, builds `prs` from this checkout, and runs the same command.

- `ExplicitInventoryTest`: open a `Recording` before constructing fixtures and
  close it with try-with-resources. Selected tests opt in. The negative control
  simulates an overselling event and requires the exact P failure,
  `spec_violation` error kind, CLI bug exit (1), and report exit (2).
- `AutomaticInventoryTest`: register `InventoryExtension` with `@ExtendWith`.
  It starts recording before `@BeforeEach` and checks after `@AfterEach`, resolving
  an `Inventory.Events` sink for dependency injection. Ordinary tests need no
  explicit checker calls. A fixture reservation is created in `@BeforeEach` and
  released in `@AfterEach`, so both lifecycle phases contribute observed events.
  Both tests can reuse the same orders because each has a fresh application and
  monitor. Product keys isolate stock within a test.
- `GeneratedInventoryTest`: jqwik generates up to 60 sequences of 1–40 reserve
  and release operations. A small reference ledger checks returned stock and
  rejected requests while P independently checks the event history. Invalid
  operations are generated deliberately, without discard-heavy preconditions.
  Each property invocation, including every shrink candidate, creates fresh app,
  ledger, log and monitor state. Do not use the Jupiter extension for jqwik:
  Jupiter lifecycle callbacks do not surround jqwik property invocations.

Run a scenario alone with `mvn test -Dtest=GeneratedInventoryTest` (substitute the
other class names). jqwik prints its seed and smallest failing operation list,
keeps failure replay state in `.jqwik-database`, and can replay a seed by adding
`seed = "<reported seed>"` to `@Property`. Keep the random default in regular CI
for exploration; the fixed attempt and sequence bounds limit its cost.

## Adapting this setup

`Inventory` has no testing dependencies. Inject an event sink and emit after a
successful state change, at the boundary where a database-backed service would
commit. Rejected operations emit nothing. Use stable product IDs as partition
keys and event payloads that describe the committed change. In a real service,
connect the same interface to your telemetry/outbox adapter; here the test sink
captures JSON Lines directly.

`Recording` is a small test adapter to the public `prs observe` CLI, **not a Java
SDK**. It buffers an invocation's events, assigns increasing logical timestamps,
and checks the log once at teardown. It uses argument arrays, a 30-second process
timeout, and unique output directories. Empty recordings fail; success requires
one named spec and all emitted events and product keys verified. Command,
serialization, parser and spec failures propagate to the test. If a body assertion
and teardown both fail, try-with-resources preserves the observation failure as a
suppressed exception; Jupiter reports callback failures alongside test failures.
When copying the adapter into your project, change its `specs/inventory.p` path
and `InventoryCorrect` spec name to your own, and set `PRS_BIN` to the absolute
path of the `prs` executable your local setup and CI build or install.

Failed observations retain JSONL, console output, JSON report and P replay logs
under `target/prs/recording-*`; assertions include that path and checker output.
Successful recordings are deleted. The expected negative control retains one
failure directory on a successful run. `mvn clean` removes the generated files.
CI uploads these diagnostics on failure along with Surefire reports and jqwik's
replay database.

This synchronous in-memory service uses a single thread per invocation. For a
concurrent service, synchronize recording and record the actual commit order;
for child processes, merge their structured logs with a documented ordering rule.
Keep resource cleanup inside the recording's lifetime so teardown events are
observed, and keep normal application assertions: the monitor only sees emitted
events and cannot detect missing instrumentation by itself.

The JUnit extension enables `observe --coverage` for each isolated recording and
merges coverage in a root-scoped `CloseableResource`. At suite end it prints a
summary and writes `target/coverage/model.json`; Maven displays the summary and CI
uploads the report. Explicit and jqwik recordings stay outside this suite report.
Completed observations include the executed prefix of a failing check. Tool failures
that produce no valid report still fail the test and do not contribute coverage.
