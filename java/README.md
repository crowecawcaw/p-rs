# Java helpers

Check the events your Java code emits against P specs, in tests and in production.
Java 17+. The helpers run the `prs observe` CLI; the rules every language helper
follows are in [docs/language-support.md](../docs/language-support.md).

| Module | Use it for | Dependencies |
| --- | --- | --- |
| `prs-emit` | `PrsEmit.emit(...)` in application code; production sinks | JDK only |
| `prs-test-core` | Recordings and checks from any test framework | `prs-emit`, Jackson, opentest4j |
| `prs-junit5` | Automatic per-test checks in JUnit Jupiter | `prs-test-core` |

## Install

The modules are not on Maven Central yet. From the repository root, with JDK 17+,
Maven 3.9 and Rust:

```sh
cargo build --locked -p prs-cli
export PRS_BIN="$PWD/target/debug/prs"
mvn --batch-mode --no-transfer-progress -f java/pom.xml install
```

Then depend on `prs-emit` (compile scope) and `prs-junit5` (test scope), version
`0.1.0-SNAPSHOT`, group `io.github.crowecawcaw`. The checker binary is found from
`.binary(...)`, then `PRS_BIN`, then `prs` on `PATH`; without one, checks fail with a
`SetupError` of kind `binary_missing` that says how to install it
(`cargo install --git https://github.com/crowecawcaw/p-rs prs-cli`).

## Emit events

```java
import io.github.crowecawcaw.prs.emit.PrsEmit;

PrsEmit.emit("eReserved", sku, Map.of("order", order, "units", units)); // event, key, payload
PrsEmit.emit("eClosed", sku);                                            // no payload
PrsEmit.emitAt(Instant.ofEpochSecond(0, tick), "eTick", "clock");        // caller-chosen time
```

Emit after the change commits, at the same place production code would publish it.
The key splits events into independent streams, one spec instance each. Payloads: a
`Map` or record for a P named tuple, a single value, or none; enums go by name,
collections and arrays as lists. Each line records the call site (`src`), a
per-process `seq`, the `pid`, and a wall-clock time in nanoseconds that never goes
backwards.

Where an event goes:

1. A recording in this JVM: the one the current thread (or the thread that started it)
   opened. Nested recordings all get it.
2. `PRS_EVENTS_DIR`, when a test started this process: one file per process,
   `<name>-<pid>-<random>.jsonl` (`<name>` is `-Dprs.name`, else the main class).
3. The production sink, if set: `PrsEmit.toFile(path)`, `toStdout()` or
   `toConsumer(line -> ...)`. Lines are the JSON `prs observe -l` reads. File and stdout
   writes happen on a background thread; `emit` never throws or waits, and drops
   (full queue, I/O error, bad payload) are counted in `PrsEmit.dropped()`.
   `PrsEmit.flush(timeout)` waits for the queue.
4. Nowhere: the default costs two volatile reads. Use `PrsEmit.enabled()` to skip
   building an expensive payload.

## Configure

Put settings in `prs.properties` (searched from the working directory upwards, then
the classpath), or as `prs.*` keys in `junit-platform.properties`. Relative paths
resolve against the file's folder. Unknown keys are a `SetupError` of kind `config`.

```properties
spec_files = specs/*.p          # paths, folders (every .p) or globs; comma-separated
specs = InventoryCorrect        # default: every spec in spec_files
exclude =                       # left out of the default all-specs selection
strict = true                   # default: true when specs is set, else false
check_hot = false
unknown_events = error          # ignore | warn | error
timeout = 30s
artifacts = target/prs          # default: <tmp>/prs; kept only for failed checks
coverage = target/coverage/monitor.json
mode = auto                     # auto | marked | off (JUnit extension)
```

Precedence: builder method, then environment variable (`PRS_SPEC_FILES` separated
like `PATH`, `PRS_BIN`, `PRS_ARTIFACTS`, `PRS_COVERAGE`, `PRS_MODE`), then the file,
then the default. The builders (`PrsObserver.builder()`, `PrsExtension.builder()`)
have a method per key: `.specFiles(...)`, `.specs(...)`, `.exclude(...)`,
`.strict(...)`, `.checkHot(...)`, `.unknownEvents(UnknownEvents.WARN)`,
`.timeout(...)`, `.artifacts(...)`, `.coverage(...)`, `.mode(...)`, `.binary(...)`.

## Explicit checks

```java
var observer = PrsObserver.builder().build(); // settings from prs.properties

try (var recording = observer.recording("reserve then release")) {
    var inventory = new Inventory();          // emits through PrsEmit
    inventory.addProduct("coffee", 3);
    inventory.reserve("coffee", "order-1", 2);
}                                             // checked here
```

Closing checks once and throws `SpecViolation` or `SetupError`. If the block already
threw, the check's failure is attached as a suppressed exception. The manual form is
`observer.start(label)`, then `recording.stop()` and `recording.check()`.
`recording.emit(...)` and `emitAt(...)` write to that recording only, for code that
takes an injected sink. Join worker threads and child processes before the check: an
event a recording's thread emits after its check, with no other recording active,
throws.

A recording with no events and `strict` off passes without running the checker.

## JUnit 5

Pick one registration:

```java
@ExtendWith(PrsExtension.class)                 // settings from prs.properties
class InventoryTest { ... }

@RegisterExtension                              // settings in code
static final PrsExtension PRS = PrsExtension.builder().specs("InventoryCorrect").build();
```

or every test class, with `junit.jupiter.extensions.autodetection.enabled=true` in
`junit-platform.properties`. An autodetected extension steps aside in classes that
register their own.

A recording starts before `@BeforeEach` and is checked after `@AfterEach`, so
fixtures are checked too. Inject it as a `Recording` parameter when code needs it
explicitly. A test that already failed keeps its failure; the check is attached as
suppressed.

Modes (`mode`, `PRS_MODE`): `auto` checks every test, `marked` only tests or classes
annotated `@Prs`, `off` nothing. Per-test options:

```java
@Test @Prs(skip = Prs.ALL) void notChecked() { ... }
@Test @Prs(skip = "SmallOrders") void notThisSpec() { ... }
@Test @Prs(checkHot = true) void everyReservationShips() { ... }
```

Several extensions on one class need `.name("billing")` and
`@PrsRecording("billing") Recording` parameters.

Jupiter callbacks do not wrap jqwik tries: open `observer.recording(...)` inside each
`@Property` method, with a `CoverageCollector` closed in `@AfterContainer`
([example](../examples/java/src/test/java/example/GeneratedInventoryTest.java)).

## Failures

`SpecViolation` is an `AssertionFailedError`, so JUnit and jqwik treat it as an
ordinary failure:

```
P spec InventoryCorrect was broken: reservation exceeds available stock

  Events (key "coffee"):
      1  eStocked   units=1           example/Inventory.java:16
      2  eReserved  order=o1 units=2  example/Inventory.java:30  <- broke the spec here

  Spec: specs/inventory.p:27
  prs output: target/prs/recording-1508053395480219822
```

Fields: `spec()`, `key()`, `message()`, `location()`, `events()` (each with `event()`,
`payload()`, `src()`), `liveness()` (a hot state at the end, under `checkHot`) and
`others()` (further violations). `SetupError.kind()` is one of `binary_missing`,
`config`, `parser`, `validation` (strict found no relevant event), `unknown_event`,
`out_of_order` or `timeout`. Assert on fields, not message text. Both keep their
artifacts folder (`artifacts()`) and the core report (`report()`).

`ObservationFailure` is the deprecated common superclass of both; catch it only in
code written for 0.1.

## Negative controls

```java
var violation = Recording.expectViolation("InventoryCorrect", "exceeds available stock", () -> {
    try (var recording = observer.recording("oversell")) { ... }
});
```

Fails if the check passes or another spec is violated. Recordings enclosing the call
(such as the extension's) ignore that spec for the rest of the test.

## Events from other processes

```java
var worker = new ProcessBuilder("java", "-cp", classpath, "example.Worker");
worker.environment().putAll(recording.environment()); // adds PRS_EVENTS_DIR
worker.start().waitFor();                             // join before the check
```

`recording.eventsDir()` is the folder itself, for containers or non-Java services
writing the [events folder format](../docs/observe.md#events-folder-format). Other
inputs:

```java
recording.addEventsDir(Path.of("volumes/events"));     // a folder this recording did not create
recording.addLog(Path.of("app-events.jsonl"));        // JSON lines prs observe reads
recording.addLog(Path.of("server.log"), line -> ...); // returns a LogEvent, or null to skip
```

Events are ordered by timestamp per key; equal times keep this process's events
first, then the events folder, added folders and logs.

## Suite coverage

Set `coverage` (or `PRS_COVERAGE`, or `.coverage(path)`): the extension writes the
union of all checks once, when the run ends, and prints the summary. Outside JUnit,
`observer.coverage()` gives a collector for that path (inert when none is set);
pass it to `observer.recording(label, collector)` and close it at the end. Collectors
merge into an existing file under a file lock, so forks and test classes can share
one path; delete the file before a run.

## Upgrading from 0.1

- `specs` is optional (default: every spec), so `strict` defaults to false without
  named specs; it stays true with them.
- Undeclared event names now fail the check (`unknownEvents`, default `ERROR`).
- Catch `SpecViolation` / `SetupError` instead of `ObservationFailure`.
- Builders throw `SetupError` (kind `config`) for bad settings.
- Artifacts default to `<tmp>/prs`; set `artifacts` to keep them under `target/`.
- Timestamps are ISO-8601 with nanoseconds, and lines carry `src`, `seq` and `pid`.
- Coverage merges into an existing file instead of replacing it.
