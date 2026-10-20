# Java helpers

Check the events your Java code emits against P specs, in tests and in production.
Java 17+. The helpers run the `snappea observe` CLI; the rules every language helper
follows are in [docs/language-support.md](../docs/language-support.md).

One artifact, `io.github.crowecawcaw:snappea`, holds both halves:

| Package | Use it for | Needs |
| --- | --- | --- |
| `io.github.crowecawcaw.snappea.emit` | `SnappeaEmit.emit(...)` in application code; production sinks | the JDK only |
| `io.github.crowecawcaw.snappea` | Recordings and checks from any test framework; the JUnit 5 extension | opentest4j and the JUnit Jupiter API, from your test framework |

`snappea` has no runtime dependencies: JUnit and opentest4j are optional, so production
code depending on `snappea` pulls nothing else and loads no test classes. The `snappea` checker comes in a classifier
jar of the same artifact, `natives-<os>-<arch>`: `natives-linux-x86_64`,
`natives-linux-aarch64`, `natives-macos-x86_64`, `natives-macos-aarch64`,
`natives-windows-x86_64`.

## Install

Depend on `snappea`, plus the natives jar for the platform the tests run on (test scope),
with the `snappea.version` property set to a [release](https://github.com/crowecawcaw/snappea/releases)
such as `0.1.1`:

```xml
<dependency>
  <groupId>io.github.crowecawcaw</groupId>
  <artifactId>snappea</artifactId>
  <version>${snappea.version}</version>
</dependency>
<dependency>
  <groupId>io.github.crowecawcaw</groupId>
  <artifactId>snappea</artifactId>
  <version>${snappea.version}</version>
  <classifier>${snappea.natives}</classifier>
  <scope>test</scope>
</dependency>
```

If everyone builds on one platform, write its classifier in place of `${snappea.natives}`.
Otherwise set it per platform with these profiles (the
[os-maven-plugin](https://github.com/trustin/os-maven-plugin)'s names differ from ours):

```xml
<profiles>
  <profile><id>snappea-linux-x86_64</id><activation><os><name>linux</name><arch>amd64</arch></os></activation><properties><snappea.natives>natives-linux-x86_64</snappea.natives></properties></profile>
  <profile><id>snappea-linux-aarch64</id><activation><os><name>linux</name><arch>aarch64</arch></os></activation><properties><snappea.natives>natives-linux-aarch64</snappea.natives></properties></profile>
  <profile><id>snappea-macos-x86_64</id><activation><os><family>mac</family><arch>x86_64</arch></os></activation><properties><snappea.natives>natives-macos-x86_64</snappea.natives></properties></profile>
  <profile><id>snappea-macos-aarch64</id><activation><os><family>mac</family><arch>aarch64</arch></os></activation><properties><snappea.natives>natives-macos-aarch64</snappea.natives></properties></profile>
  <profile><id>snappea-windows-x86_64</id><activation><os><family>windows</family><arch>amd64</arch></os></activation><properties><snappea.natives>natives-windows-x86_64</snappea.natives></properties></profile>
</profiles>
```

Gradle (Kotlin DSL):

```kotlin
val snappeaOs = System.getProperty("os.name").lowercase().let {
    if (it.startsWith("mac")) "macos" else if (it.startsWith("windows")) "windows" else "linux" }
val snappeaArch = if (System.getProperty("os.arch") in setOf("aarch64", "arm64")) "aarch64" else "x86_64"
dependencies {
    implementation("io.github.crowecawcaw:snappea:$snappeaVersion")
    testRuntimeOnly("io.github.crowecawcaw:snappea:$snappeaVersion:natives-$snappeaOs-$snappeaArch")
}
```

The checker binary is found from `.binary(...)`, then `SNAPPEA_BIN`, then the natives jar
on the classpath, then `snappea` on `PATH`. The natives jar's binary is copied on first
use to `$XDG_CACHE_HOME/snappea/<version>/` (else `~/Library/Caches/snappea` on macOS,
`%LOCALAPPDATA%\snappea` on Windows, `~/.cache/snappea` elsewhere), checked against the
SHA-256 in the jar, and reused by later runs. Without any binary, checks fail with a
`SetupError` of kind `binary_missing` naming the natives dependency to add.

### From this repository

To use a checkout instead of a release (version `X.Y.Z-SNAPSHOT` in
[pom.xml](pom.xml)): from the repository root, with JDK 17+, Maven 3.9 and Rust, build
the checker and install `snappea` with a natives jar for your platform (here macOS on
Apple silicon):

```sh
cargo build --locked -p snappea
mkdir -p java/natives/natives-macos-aarch64
cp target/debug/snappea java/natives/natives-macos-aarch64/
SNAPPEA_BIN="$PWD/target/debug/snappea" mvn -f java/pom.xml -Pnatives install
```

The `natives` profile packages every `java/natives/<classifier>/snappea` (or `snappea.exe`) it
finds, or the folders under `-Dsnappea.natives=DIR`, as classifier jars. Or skip the
natives jar and set `SNAPPEA_BIN`, or put `snappea` on `PATH` with `cargo install --path crates/snappea`.

## Emit events

```java
import io.github.crowecawcaw.snappea.emit.SnappeaEmit;

SnappeaEmit.emit("eReserved", sku, Map.of("order", order, "units", units)); // event, key, payload
SnappeaEmit.emit("eClosed", sku);                                            // no payload
SnappeaEmit.emitAt(Instant.ofEpochSecond(0, tick), "eTick", "clock");        // caller-chosen time
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
2. `SNAPPEA_EVENTS_DIR`, when a test started this process: one file per process,
   `<name>-<pid>-<random>.jsonl` (`<name>` is `-Dsnappea.name`, else the main class).
3. The production sink, if set: `SnappeaEmit.setSink(Sink.file(path))`, `Sink.stdout()`,
   `Sink.writer(out)` or `Sink.callback(line -> ...)`. Lines are the JSON
   `snappea observe -l` reads, written on a background thread; `emit` never throws or
   waits, and drops (full queue, I/O error, bad payload) are counted in
   `SnappeaEmit.dropped()`. `SnappeaEmit.flush(timeout)` waits for the queue;
   `SnappeaEmit.close()` (or `setSink(null)`) writes it and turns the sink off. See
   [Testing and production](../docs/production.md).
4. Nowhere: the default costs two volatile reads. Use `SnappeaEmit.enabled()` to skip
   building an expensive payload.

## Configure

Put settings in `snappea.properties` (searched from the working directory upwards, then
the classpath), or as `snappea.*` keys in `junit-platform.properties`. Relative paths
resolve against the file's folder. Unknown keys are a `SetupError` of kind `config`.

```properties
spec_files = specs/*.p          # paths, folders (every .p) or globs; comma-separated
specs = InventoryCorrect        # default: every spec in spec_files
exclude =                       # left out of the default all-specs selection
strict = true                   # default: true when specs is set, else false
check_hot = false
unknown_events = error          # ignore | warn | error
timeout = 30s
artifacts = target/snappea          # default: <tmp>/snappea; kept only for failed checks
coverage = target/coverage/monitor.json
mode = auto                     # auto | marked | off (JUnit extension)
```

Precedence: builder method, then environment variable (`SNAPPEA_SPEC_FILES` separated
like `PATH`, `SNAPPEA_BIN`, `SNAPPEA_ARTIFACTS`, `SNAPPEA_COVERAGE`, `SNAPPEA_MODE`), then the file,
then the default. The builders (`SnappeaObserver.builder()`, `SnappeaExtension.builder()`)
have a method per key: `.specFiles(...)`, `.specs(...)`, `.exclude(...)`,
`.strict(...)`, `.checkHot(...)`, `.unknownEvents(UnknownEvents.WARN)`,
`.timeout(...)`, `.artifacts(...)`, `.coverage(...)`, `.mode(...)`, `.binary(...)`.

## Explicit checks

```java
var observer = SnappeaObserver.builder().build(); // settings from snappea.properties

try (var recording = observer.recording("reserve then release")) {
    var inventory = new Inventory();          // emits through SnappeaEmit
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
@ExtendWith(SnappeaExtension.class)                 // settings from snappea.properties
class InventoryTest { ... }

@RegisterExtension                              // settings in code
static final SnappeaExtension SNAPPEA = SnappeaExtension.builder().specs("InventoryCorrect").build();
```

or every test class, with `junit.jupiter.extensions.autodetection.enabled=true` in
`junit-platform.properties`. An autodetected extension steps aside in classes that
register their own.

A recording starts before `@BeforeEach` and is checked after `@AfterEach`, so
fixtures are checked too. Inject it as a `Recording` parameter when code needs it
explicitly. A test that already failed keeps its failure; the check is attached as
suppressed.

Modes (`mode`, `SNAPPEA_MODE`): `auto` checks every test, `marked` only tests or classes
annotated `@Snappea`, `off` nothing. Per-test options:

```java
@Test @Snappea(skip = Snappea.ALL) void notChecked() { ... }
@Test @Snappea(skip = "SmallOrders") void notThisSpec() { ... }
@Test @Snappea(checkHot = true) void everyReservationShips() { ... }
```

Several extensions on one class need `.name("billing")` and
`@SnappeaRecording("billing") Recording` parameters.

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
  snappea output: target/snappea/recording-1508053395480219822
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
worker.environment().putAll(recording.environment()); // adds SNAPPEA_EVENTS_DIR
worker.start().waitFor();                             // join before the check
```

`recording.eventsDir()` is the folder itself, for containers or non-Java services
writing the [events folder format](../docs/observe.md#events-folder-format). Other
inputs:

```java
recording.addEventsDir(Path.of("volumes/events"));     // a folder this recording did not create
recording.addLog(Path.of("app-events.jsonl"));        // JSON lines snappea observe reads
recording.addLog(Path.of("server.log"), line -> ...); // returns a LogEvent, or null to skip
```

Events are ordered by timestamp per key; equal times keep this process's events
first, then the events folder, added folders and logs.

## Suite coverage

Set `coverage` (or `SNAPPEA_COVERAGE`, or `.coverage(path)`): the extension writes the
union of all checks once, when the run ends, and prints the summary. Outside JUnit,
`observer.coverage()` gives a collector for that path (inert when none is set);
pass it to `observer.recording(label, collector)` and close it at the end. Collectors
merge into an existing file under a file lock, so forks and test classes can share
one path; delete the file before a run.

## Upgrading from 0.1

- `snappea-emit`, `snappea-test-core` and `snappea-junit5` are one artifact, `snappea`; packages and
  class names are unchanged. Add the natives jar to drop `SNAPPEA_BIN`.
- `specs` is optional (default: every spec), so `strict` defaults to false without
  named specs; it stays true with them.
- Undeclared event names now fail the check (`unknownEvents`, default `ERROR`).
- Catch `SpecViolation` / `SetupError` instead of `ObservationFailure`.
- Builders throw `SetupError` (kind `config`) for bad settings.
- Artifacts default to `<tmp>/snappea`; set `artifacts` to keep them under `target/`.
- Timestamps are ISO-8601 with nanoseconds, and lines carry `src`, `seq` and `pid`.
- Coverage merges into an existing file instead of replacing it.
