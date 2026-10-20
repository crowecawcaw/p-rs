# Java helpers

Check the events your Java code emits against P specs, in tests and in production.
Java 17+. The helpers run the `prs observe` CLI; the rules every language helper
follows are in [docs/language-support.md](../docs/language-support.md).

One artifact, `io.github.crowecawcaw:p-rs`, holds both halves:

| Package | Use it for | Needs |
| --- | --- | --- |
| `io.github.crowecawcaw.prs.emit` | `PrsEmit.emit(...)` in application code; production sinks | the JDK only |
| `io.github.crowecawcaw.prs` | Recordings and checks from any test framework; the JUnit 5 extension | opentest4j and the JUnit Jupiter API, from your test framework |

`p-rs` has no runtime dependencies: JUnit and opentest4j are optional, so production
code depending on `p-rs` pulls nothing else and loads no test classes. The `prs` checker comes in a classifier
jar of the same artifact, `natives-<os>-<arch>`: `natives-linux-x86_64`,
`natives-linux-aarch64`, `natives-macos-x86_64`, `natives-macos-aarch64`,
`natives-windows-x86_64`.

## Install

Depend on `p-rs`, plus the natives jar for the platform the tests run on (test scope),
with the `prs.version` property set to a [release](https://github.com/crowecawcaw/p-rs/releases):

```xml
<dependency>
  <groupId>io.github.crowecawcaw</groupId>
  <artifactId>p-rs</artifactId>
  <version>${prs.version}</version>
</dependency>
<dependency>
  <groupId>io.github.crowecawcaw</groupId>
  <artifactId>p-rs</artifactId>
  <version>${prs.version}</version>
  <classifier>${prs.natives}</classifier>
  <scope>test</scope>
</dependency>
```

If everyone builds on one platform, write its classifier in place of `${prs.natives}`.
Otherwise set it per platform with these profiles (the
[os-maven-plugin](https://github.com/trustin/os-maven-plugin)'s names differ from ours):

```xml
<profiles>
  <profile><id>prs-linux-x86_64</id><activation><os><name>linux</name><arch>amd64</arch></os></activation><properties><prs.natives>natives-linux-x86_64</prs.natives></properties></profile>
  <profile><id>prs-linux-aarch64</id><activation><os><name>linux</name><arch>aarch64</arch></os></activation><properties><prs.natives>natives-linux-aarch64</prs.natives></properties></profile>
  <profile><id>prs-macos-x86_64</id><activation><os><family>mac</family><arch>x86_64</arch></os></activation><properties><prs.natives>natives-macos-x86_64</prs.natives></properties></profile>
  <profile><id>prs-macos-aarch64</id><activation><os><family>mac</family><arch>aarch64</arch></os></activation><properties><prs.natives>natives-macos-aarch64</prs.natives></properties></profile>
  <profile><id>prs-windows-x86_64</id><activation><os><family>windows</family><arch>amd64</arch></os></activation><properties><prs.natives>natives-windows-x86_64</prs.natives></properties></profile>
</profiles>
```

Gradle (Kotlin DSL):

```kotlin
val prsOs = System.getProperty("os.name").lowercase().let {
    if (it.startsWith("mac")) "macos" else if (it.startsWith("windows")) "windows" else "linux" }
val prsArch = if (System.getProperty("os.arch") in setOf("aarch64", "arm64")) "aarch64" else "x86_64"
dependencies {
    implementation("io.github.crowecawcaw:p-rs:$prsVersion")
    testRuntimeOnly("io.github.crowecawcaw:p-rs:$prsVersion:natives-$prsOs-$prsArch")
}
```

The checker binary is found from `.binary(...)`, then `PRS_BIN`, then the natives jar
on the classpath, then `prs` on `PATH`. The natives jar's binary is copied on first
use to `$XDG_CACHE_HOME/p-rs/<version>/` (else `~/Library/Caches/p-rs` on macOS,
`%LOCALAPPDATA%\p-rs` on Windows, `~/.cache/p-rs` elsewhere), checked against the
SHA-256 in the jar, and reused by later runs. Without any binary, checks fail with a
`SetupError` of kind `binary_missing` naming the natives dependency to add.

### From this repository

To use a checkout instead of a release (version `X.Y.Z-SNAPSHOT` in
[pom.xml](pom.xml)): from the repository root, with JDK 17+, Maven 3.9 and Rust, build
the checker and install `p-rs` with a natives jar for your platform (here macOS on
Apple silicon):

```sh
cargo build --locked -p p-rs
mkdir -p java/natives/natives-macos-aarch64
cp target/debug/prs java/natives/natives-macos-aarch64/
PRS_BIN="$PWD/target/debug/prs" mvn -f java/pom.xml -Pnatives install
```

The `natives` profile packages every `java/natives/<classifier>/prs` (or `prs.exe`) it
finds, or the folders under `-Dprs.natives=DIR`, as classifier jars. Or skip the
natives jar and set `PRS_BIN`, or put `prs` on `PATH` with `cargo install --path crates/p-rs`.

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
3. The production sink, if set: `PrsEmit.setSink(Sink.file(path))`, `Sink.stdout()`,
   `Sink.writer(out)` or `Sink.callback(line -> ...)`. Lines are the JSON
   `prs observe -l` reads, written on a background thread; `emit` never throws or
   waits, and drops (full queue, I/O error, bad payload) are counted in
   `PrsEmit.dropped()`. `PrsEmit.flush(timeout)` waits for the queue;
   `PrsEmit.close()` (or `setSink(null)`) writes it and turns the sink off. See
   [Testing and production](../docs/production.md).
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
