# Language helper requirements

Normative requirements for the language helpers that check application events against P
specs from tests: Python (`python/`), Go (`go/prstest`), Java (`java/`), Rust
(`prs::test` in `crates/p-rs`), and any future one. Every helper offers the same features, names,
defaults and failure behaviour, each written idiomatically for its language.

Reader: an agent implementing or reviewing a helper. MUST, SHOULD and MAY are as in
RFC 2119. Rationale for non-obvious choices is in [Decisions](#decisions). The current
state of each helper is in the [gap checklist](#gap-checklist).

## Layers

Each helper implements all five.

| Layer | Responsibility | Owned by |
| --- | --- | --- |
| L0 Core contract | Event line format, report JSON, exit codes, coverage JSON, compiling and checking | core (`prs`) |
| L1 Emitter | `emit` from application code; no dependencies; inert outside tests; safe in production | helper |
| L2 Recording | Start, collect, check; structured result and errors | helper |
| L3 Framework integration | Automatic per-test checking in the language's main test framework | helper |
| L4 Suite | Coverage union across tests, written once at suite end | helper, merging delegated to core |

## Uniform names

Spell every name in the language's convention (`check_hot` / `CheckHot` / `checkHot`).
Never use a synonym (`liveness`, `AllowUnobserved`, `PRS_BINARY`). A helper MAY keep an
old name as a deprecated alias for one minor release, warning when it is used.

### Environment variables

| Variable | Meaning | Precedence |
| --- | --- | --- |
| `PRS_BIN` | Checker binary | after explicit config, before bundled copy and `PATH` |
| `PRS_SPEC_FILES` | Spec files, separated like `PATH` (`os.pathsep`) | overrides the config file |
| `PRS_EVENTS_DIR` | Folder child processes write events to ([format](observe.md#events-folder-format)) | set by the recording |
| `PRS_ARTIFACTS` | Folder to keep artifacts of failed checks in | overrides the config file |
| `PRS_COVERAGE` | Suite coverage JSON path; setting it enables L4 | overrides the config file |
| `PRS_MODE` | L3 mode: `auto`, `marked` or `off` | overrides the config file |
| `PRS_DOWNLOAD` | `0` turns off downloading the checker (Go only) | |

Relative paths in environment variables resolve against the process working directory.
An explicit API argument overrides every variable ([precedence](#configuration)).

### Concepts

| Concept | Type | Default |
| --- | --- | --- |
| `spec_files` | paths or globs; a folder means every `.p` in it | required (config or env) |
| `specs` | spec names | every spec in `spec_files` |
| `exclude` | spec names | none |
| `strict` | bool | `true` when `specs` is given; `false` when `specs` defaults to all |
| `check_hot` | bool | `false` |
| `unknown_events` | `ignore` / `warn` / `error` | `error` |
| `timeout` | duration | 30 s (helpers that start a process) |
| `artifacts` | path | a temp folder, kept only when the check fails |
| `mode` | `auto` / `marked` / `off` | `auto` where the framework allows it, else `marked` |

### API names

| Name | Layer | Meaning |
| --- | --- | --- |
| `emit`, `emit_at` | L1 | Record an event now / at a given time |
| `set_sink` | L1 | Send events outside tests to a production sink, replacing the previous one; none/nil/off turns it off |
| `file`, `stdout`, `writer`, `callback` | L1 | Production sink constructors |
| `dropped`, `flush`, `close` | L1 | Events dropped so far; wait for queued events (with a timeout); write them and turn the sink off |
| `check_spec` (or the language's scoped form) | L2 | Scoped recording that checks on exit |
| `Recording` / `Recorder`, `start`, `stop`, `check` | L2 | Manual form |
| `events_dir`, `env` | L2 | The recording's `PRS_EVENTS_DIR` folder, and an environment for children |
| `add_events_dir`, `add_log` | L2 | Extra event inputs |
| `SpecViolation`, `SetupError` | L2 | Error types |
| `artifacts` | L2 | Field of both error types: the failed check's folder (events, checker output, report) |
| `expect_violation` | L2 | Negative control |

## L0 Core contract

- Helpers MUST check with the core: run `prs observe` (CLI helpers) or call
  `prs::checker::observe` (Rust, in-process). They MUST NOT parse P, check payloads,
  reorder events or interpret monitor semantics themselves.
- CLI helpers MUST pass `--json-report` and read results from the report, never from
  console output. Exit codes: 0 pass, 1 spec violation, 2 tool or input error. No report
  file means a configuration or compile error; its console output is the message.
- Spec selection: `--spec <names>` when `specs` is set, else `--all-specs` with
  `--exclude-specs`. `strict` → `--strict`; `check_hot` → `--check-hot`;
  `unknown_events` → `--unknown-events` ([#47](https://github.com/crowecawcaw/p-rs/issues/47));
  coverage → `--coverage`.
- Events go to the core in the [events folder format](observe.md#events-folder-format),
  with `-l <folder>`. Extra inputs (`add_events_dir`, `add_log`) are passed as further
  `-l` values, which the core merges into one input set
  ([several logs](observe.md#several-logs-and-glob-patterns)).
- Coverage: per-check `coverage` objects from the report; suite union with
  `prs coverage merge-observe` (or `ObserveCoverageReport::merge` in Rust). Helpers MUST
  NOT merge coverage sets themselves.
- A helper SHOULD fail with a `SetupError` naming both versions when the binary lacks an
  option the helper needs, rather than drop the option.

## L1 Emitter

- `emit(event, key, payload)` and `emit_at(time, event, key, payload)`, in that order.
  Languages with keyword arguments MAY take `key` and the time by keyword (Python:
  `emit(event, payload, key=..., at=...)`).
- Payload: a named-tuple map, a single value, or nothing, in the language's natural form
  (keyword arguments, `map[string]any`, `Map`, `json!`). Enums are sent by name; sets and
  tuples as lists.
- Zero dependencies beyond the standard library (Rust: `serde` / `serde_json`, which the
  `json!` payload form needs). No test-framework dependency.
- Outside a recording, with no `PRS_EVENTS_DIR` and no production sink, `emit` does
  nothing and costs at most a lock-free check. In production it MUST NOT throw, panic or
  block.
- In tests, emitting into a recording that was already checked MUST make the check (or
  the test) fail; it is never silently dropped.
- Record the call site as `src` (`file:line`) wherever the language provides it
  cheaply (Python frames, Go `runtime.Caller`, Java `StackWalker`, Rust `#[track_caller]`).
- Order: a per-process sequence number (`seq`) assigned under a lock. Time: wall clock
  in nanoseconds, never decreasing within a process (use the larger of now and the last
  value).
- With `PRS_EVENTS_DIR` set and no in-process recording, `emit` appends to the process's
  own file in that folder ([format](observe.md#events-folder-format)).
- Production sink (see [Testing and production](production.md)):
  - `set_sink(sink)` sends events, when no recording takes them and `PRS_EVENTS_DIR` is
    unset, to `sink` as [event lines](observe.md#events-folder-format). It replaces
    the previous sink, writing what that one had queued. With `PRS_EVENTS_DIR` set it
    does nothing: a test started the process, and its folder wins.
  - Sinks: `file(path)` appends (creating the file and its folder), `stdout()`,
    `writer(w)` (flushed, never closed by the helper) and `callback(f)` (gets each line
    without the newline). Lines from one process never interleave or split; give each
    process its own file.
  - Sinks write from a background thread through a bounded queue. A full queue, a
    failed write, a callback that throws, or a payload that cannot be encoded drops the
    event and adds one to `dropped()`; `emit` never sees it.
  - `flush(timeout)` waits for the queue and returns whether it emptied. `close()`
    flushes (bounded) and turns the sink off; call it at shutdown. Helpers SHOULD also
    flush at process exit where the language has a hook.

  | Language | Set | Sinks | Dropped / flush / close |
  | --- | --- | --- | --- |
  | Python | `prs.set_sink(target)` returns a `Sink`; `None` is off | a path, `"-"` or a text stream, a callable | `sink.dropped`, `sink.flush(timeout)`, `sink.close()` |
  | Go | `prsemit.SetSink(s)`; `nil` is off | `prsemit.File(path)` (`(*Sink, error)`), `Stdout()`, `Writer(w)`, `Callback(f)` | `prsemit.Dropped()`, `Flush(timeout)`, `Close()` |
  | Java | `PrsEmit.setSink(sink)`; `null` is off | `Sink.file(path)`, `Sink.stdout()`, `Sink.writer(out)`, `Sink.callback(f)` | `PrsEmit.dropped()`, `flush(timeout)`, `close()` |
  | Rust | `prs::emit::set_sink(emitter)`; `Emitter::off()` is off | `Emitter::file(path)?`, `stdout()`, `writer(w)`, `callback(f)` | `prs::emit::dropped()`, `flush(timeout)`, `close()` |

  Rust's sink is an `Emitter` value, which code can also take as a dependency; its
  `callback` gets each `Event` on the emitting thread, because test recordings are
  built on it.
- Unknown event names MAY be rejected at emit time in tests when the recording has
  loaded the declared names; they MUST be rejected at check time under
  `unknown_events = error`.

## L2 Recording

### Forms

- MUST offer a scoped form (context manager, try-with-resources, closure, or a fixture
  registering `t.Cleanup`) and a manual `start` / `stop` / `check` form.
- The check MUST NOT run while an exception or panic is unwinding, and MUST NOT replace
  the test's original failure. It MAY attach its result to it (suppressed exception, log).
- A recording is never shared between tests, subtests, or property-test cases.

### Errors

- `SpecViolation`: fields `spec`, `key`, `message`, `location` (P assert `file:line`),
  `events` (the key's last events, each with `event`, `payload`, `src`), `liveness`
  (true for a hot-state end under `check_hot`), `others` (further violations from the
  same check), `artifacts` (the check's kept folder). It MUST be the framework's assertion-failure type (Python
  `AssertionError`, Java `AssertionError` / `AssertionFailedError`, Go: reported through
  `t.Error`/`t.Fatal` and returned as `*SpecViolation` for `errors.As`, Rust: a panic in
  `#[test]` and an `Err` value in the explicit form), so property-test shrinkers treat it
  as an ordinary failure.
- `SetupError`: everything else, with a `kind`:

  | `kind` | Cause |
  | --- | --- |
  | `binary_missing` | No checker binary; the message says how to install one |
  | `config` | Bad options or config file, missing spec files, P compile errors, unknown spec name |
  | `parser` | An event line or payload the core cannot read (including truncated lines) |
  | `validation` | `strict` found a selected spec with no relevant event |
  | `unknown_event` | An event name no loaded P file declares, under `unknown_events = error` |
  | `out_of_order` | Live input out of order (core report kind) |
  | `timeout` | The checker exceeded `timeout` |

  Kinds that the core reports are copied from the report's `errors[].kind`.
- Assertions on errors use these fields, never message text matching.

### Negative controls

`expect_violation(spec, match)` requires the enclosed check to raise a `SpecViolation`
for `spec` whose `message` matches `match` (a regular expression, searched). It fails
if the check passes or another spec is violated. Enclosing recordings ignore `spec` for
the rest of the test, so the deliberate violation is not reported twice.

### Failure output

Match Python's format: the spec's message; the violating key's last events (up to 20)
with `src` and a marker on the failing event (or "events ended here" for liveness); the
P assert location; the artifacts path. Example in [python.md](python.md#failure-reports).

### Event inputs

- Each recording creates its own events folder and exposes it as `events_dir()` and an
  environment `env()` (the current environment plus `PRS_EVENTS_DIR`), so starting a
  child with it is one line. Language spellings: Go `r.EventsDir()`, `r.Env()`; Java
  `recording.eventsDir()`, `recording.environment()`; Rust `recording.events_dir()`,
  `recording.env()`.
- At check time the recording reads its folder together with in-process events.
- `add_events_dir(path)` adds a folder the recording did not create (container volume,
  session-scoped service). `add_log(path, parse)` adds a log whose lines `parse` turns
  into events, or a JSON-lines log when `parse` is omitted.
- Children must be stopped or joined before the check; see the
  [format](observe.md#events-folder-format).

### Binary resolution

In order: explicit config, `PRS_BIN`, the bundled or embedded copy, `PATH`; Go then
downloads a release binary. A miss is a `SetupError` of kind `binary_missing` with
install instructions. Rust checks in-process and has no binary. Distribution per
language is under [Packaging and distribution](#packaging-and-distribution).

## L3 Framework integration

- Modes: `auto` (every test), `marked` (opted-in tests), `off`. `PRS_MODE` overrides
  config. Where the framework has no global per-test hook (Go `testing`, Rust `#[test]`),
  `auto` is unavailable: the helper offers a one-line fixture, and `off` makes it inert.
- Per-test options: skip every spec, skip some specs, `check_hot`.
- The recording starts before fixtures or setup run and is checked after the test body.
  A test that emitted nothing and is not strict passes without running the checker.
- A test that already failed is not failed again by the check.
- Recordings are never shared across property-test cases. Each helper documents the
  per-case pattern for its property framework.

| Framework | Hook | Setup inside | Teardown inside | Property testing |
| --- | --- | --- | --- | --- |
| pytest (Python) | plugin; `@pytest.mark.prs(skip=..., check_hot=...)` | fixture setup: yes | fixture teardown: no (checked when the body ends) | Hypothesis: `with check_spec(...)` inside the `@given` body |
| `testing` (Go) | `prstest.Start(t, cfg)` in the test's fixture | after `Start` | `t.Cleanup` registered after `Start`: yes; before: no | `go test -fuzz`: `Start` inside the `f.Fuzz` callback with its `t` |
| JUnit 5 (Java) | `PrsExtension` (`@ExtendWith` or autodetection), `@Prs(skip=..., checkHot=...)` | `@BeforeEach`: yes | `@AfterEach`: yes | jqwik: `observer.recording(...)` per `@Property` try, coverage via `@AfterContainer` |
| `#[test]` (Rust) | `model.check(\|recording\| ...)` closure | inside the closure | inside the closure | proptest: a recording per case inside the `proptest!` body |

Agents writing tests SHOULD join workers inside the recording; the table says where
that is.

## L4 Suite

- One setting enables suite coverage: `PRS_COVERAGE` or the framework option.
- The helper owns collection and the merge call. Users write no `TestMain`, static
  mutex or JSON code where the framework offers a suite-end hook.
- With a suite-end hook (pytest session finish, JUnit `CloseableResource` /
  `LauncherSessionListener`), the helper writes the file once at suite end.
- Without one (Rust; Go without `prstest.Main`), the helper merges each check into the
  file as it completes.
- Several processes may share one `PRS_COVERAGE` path (pytest-xdist, `go test ./...`,
  Gradle forks): the helper MUST merge into an existing file under an exclusive file
  lock, never overwrite it. Deleting a stale file before a run is the user's job (CI
  starts clean).
- Output: the core `observed_monitor_suite` JSON at the path, and the `render_text`
  summary on the framework's terminal output.

## Configuration

One project file per ecosystem, same keys as the [concepts](#concepts):

| Language | File |
| --- | --- |
| Python | `[tool.prs]` in `pyproject.toml` |
| Java | `prs.properties`, or `prs.*` keys in `junit-platform.properties` |
| Go | none: `prstest.Config` and environment variables |
| Rust | `[package.metadata.prs]` in the crate's `Cargo.toml` |

- Paths in a config file resolve relative to that file, never the working directory.
  Go resolves `Config` paths relative to the package folder (`go test`'s working
  directory).
- Precedence: explicit API argument, then environment variable, then config file, then
  default. Explicit arguments include Go's `Config` fields, Java's builder calls and
  Rust's `Model` builders: a set field beats the environment, which applies only when
  it is unset.
- Unknown keys in a config file are a `SetupError` of kind `config`.

## Conformance corpus

`tests/conformance/<case>/` holds `spec.p`, `events.jsonl` (optional
`children/*.jsonl`) and `expect.json`. The schema and replay steps are in
[tests/conformance/README.md](../tests/conformance/README.md). In short:

```json
{
  "description": "...",
  "requires": ["events_dir"],
  "options": {"specs": [...], "exclude": [...], "strict": true, "check_hot": false, "unknown_events": "error"},
  "expect": {"outcome": "pass | violation | setup_error", "spec": "...", "key": "...",
             "message": "substring", "liveness": false, "kind": "validation",
             "coverage": {"Spec": {"states": {"covered": 1, "total": 2}, "handlers": {...}}}}
}
```

- Each helper MUST have a test that replays every case through its own L1/L2 API
  (`emit_at` for `events.jsonl`, children files copied into the recording's events
  folder) and compares the result with `expect.json`.
- A case whose `requires` lists a feature the helper lacks is skipped with the reason
  printed. Nothing else is skipped.
- `crates/p-rs/tests/cli_conformance.rs` replays the corpus through `prs observe`, so the
  corpus stays consistent with the core.
- A helper is conformant when the corpus passes without skips and its checklist below is
  complete. A new language starts by copying the checklist.

## Decisions

### Packaging and distribution

Every language publishes **one package named `p-rs`**, holding the emitter, the test
helper and a way to get the checker. Code uses the short name `prs`. (`prs` itself is
taken on crates.io and PyPI.) Production code depends on the same package and uses only
the emitter; unused test code is not loaded (Go, Java) or not compiled (Rust features).

| Language | Package | Code name | Checker |
| --- | --- | --- | --- |
| Python | PyPI `p-rs` | `import prs` | Native binary in platform wheels. |
| Rust | crates.io `p-rs`; features `emit` (production, no checker), `test`, `cli` (the `prs` binary) | `prs` | In-process; no binary. |
| Go | module `github.com/crowecawcaw/p-rs` | packages `prsemit`, `prstest` | `PRS_BIN`, `PATH`, then a release binary downloaded into a per-version user cache and verified against checksums in the module. The module MUST NOT contain native binaries. |
| Java | Maven Central `io.github.crowecawcaw:p-rs`; JUnit 5 optional | `io.github.crowecawcaw.prs` | Classifier jars `natives-<os>-<arch>` of the same artifact, extracted to a per-version cache on first use. |

All packages share one version and release (`release.yml`); Go tags are plain `vX.Y.Z`.
The tagged commit holds the release version everywhere; afterwards `main` keeps it in
`Cargo.toml` while Go goes back to `dev` and Java to the next `-SNAPSHOT`
([RELEASING.md](../.github/RELEASING.md)).
The checker must work offline with no per-user setup, so only Go downloads, and only
when no binary is found. A WASI build (wazero for Go, Chicory for Java) may later replace
the native binaries if it passes the conformance corpus at no more than 2× native cost.
Work is tracked in [#64](https://github.com/crowecawcaw/p-rs/issues/64).

### Go suite hook

Keep the explicit fixture (`prstest.Start(t, cfg)`): Go has no global per-test hook,
and a one-line fixture is idiomatic. For L4, add `prstest.Main(m)` for use as
`func TestMain(m *testing.M) { prstest.Main(m) }`: `TestMain` is Go's suite hook, so
this is the framework hook, not boilerplate. Without it, the helper merges per check
under a file lock (L4), so coverage still works, only slower.

### Strict default

`strict` defaults to `true` when the user names the specs, and `false` when `specs`
defaults to every spec. Strict means "every selected spec checked an event". With named
specs that catches disabled instrumentation, which is its purpose. With every spec
selected (pytest `auto` mode) most tests exercise only some specs, so strict would fail
nearly every test.

### Rust `Model`

`Model` accepts both paths and embedded sources, and several specs:

- `Model::load()` reads `[package.metadata.prs]` from `CARGO_MANIFEST_DIR` (paths relative
  to that `Cargo.toml`), honouring `PRS_SPEC_FILES`.
- `Model::from_files(paths)` and `Model::compile(sources)` (for `include_str!`) for
  explicit setup.
- `.specs([...])`, `.exclude([...])`, `.strict(..)`, `.check_hot(..)`,
  `.unknown_events(..)` as builders; `specs` defaults to every spec.
- `EventSink::emit(event, key, payload)`: the argument order changes to match L1.
  The crate is pre-1.0, so no alias.

### Smaller decisions

- `PRS_MODE` is added to the uniform environment variables: Go and Rust have no
  command-line switch for test runs, so an environment variable is the only uniform way
  to turn checking off.
- Events folder files are `<name>-<pid>-<random>.jsonl`: containers sharing a volume
  often share pid 1, and a process name makes reports readable
  ([#49](https://github.com/crowecawcaw/p-rs/issues/49) open question).
- `unknown_events` is separate from `strict` (as #47 proposes); helpers default both on
  for named specs. A per-project allow-list of undeclared names is not added: `warn`
  covers mixed sinks.
- Session-scoped services: a helper SHOULD offer a suite-level recording (checked at
  suite end) over a folder the service writes to, replacing Python's documented
  workaround.

## Gap checklist

Unchecked items are work to do. Close each with a PR; update this list in the same PR.
The common items #45, #47, #48, #49 and #53 cover are linked.

### Python

- [x] Bundled binary (wheel)
- [x] `emit` / `at=`, `src`, `seq`, `pid`
- [x] `specs` default all, `exclude`
- [x] Structured `SpecViolation`, `expect_violation`, failure output
- [x] `auto` / `marked` / `off`; fixture setup inside, teardown outside
- [x] `PRS_EVENTS_DIR`, `add_events_dir`, `add_log`
- [x] Suite coverage with no boilerplate (`--prs-coverage`)
- [x] `PRS_BIN` (#53)
- [x] `check_hot` everywhere (`check_spec`, `Recorder`, mark). (`SpecViolation.liveness` stays.)
- [x] `strict` default per [Strict default](#strict-default)
- [x] `unknown_events` option, default `error` (#47)
- [x] `SetupError` with `kind`; `artifacts` on both error types
- [x] Emitting after the check fails it (threads started in the recording)
- [x] `PRS_COVERAGE`, `PRS_ARTIFACTS`, `PRS_MODE`, `timeout`
- [x] `events_dir()` / `env()` on `Recorder`
- [x] Events folder file name `<name>-<pid>-<random>.jsonl`
- [x] Coverage file merge under a file lock (pytest-xdist)
- [x] Production sink for `emit` (#48)
- [x] Conformance runner

### Go

- [x] `Emit(event, key, payload)`, `EmitAt`
- [x] `PRS_BIN`, `strict` default true with named specs, `CheckHot`
- [x] `timeout`, artifacts kept on failure
- [x] Packaging per [Packaging and distribution](#packaging-and-distribution): module at the repo root, release binary downloaded and verified (#64)
- [x] `Specs` optional (default all), `Exclude`
- [x] `Strict` (`*bool`, set with `prstest.Bool`)
- [x] `Config` fields beat environment variables ([precedence](#configuration))
- [x] `*SpecViolation` / `*SetupError` for `errors.As` (#53)
- [x] `ExpectViolation(t, spec, match)` (a `Recorder` method)
- [x] Failure output in Python's format, with `src` from `runtime.Caller`
- [x] `EventsDir()`, `Env()`, `AddEventsDir`, `AddLog` (#49); move `prsemit` out of `python/`
- [x] `UnknownEvents`, default `error` (#47)
- [x] `prstest.Main(m)` and `PRS_COVERAGE`; per-check merge without it
- [x] `PRS_SPEC_FILES`, `PRS_ARTIFACTS`, `PRS_MODE`
- [x] Production emitter package (#48)
- [x] Conformance runner

### Java

- [x] `emit(event, key, payload)`, `emitAt`
- [x] `PRS_BIN`, `strict` default true, `checkHot`, `timeout`, `artifacts`
- [x] Suite coverage via the extension; jqwik via `@AfterContainer`
- [x] Packaging per [Packaging and distribution](#packaging-and-distribution): one `p-rs` artifact with natives classifier jars, on Maven Central (#64)
- [x] `specs` optional (default all), `exclude`
- [x] `SpecViolation extends AssertionFailedError` with structured fields; `SetupError` with `kind`
- [x] `expectViolation(spec, match)`
- [x] Failure output in Python's format, with `src` from `StackWalker`
- [x] `auto` (extension autodetection) / `marked` / `off`; `@Prs(skip=..., checkHot=...)`
- [x] `eventsDir()`, `environment()`, `addEventsDir`, `addLog` (#49)
- [x] `unknownEvents`, default `error` (#47)
- [x] Config file (`prs.properties` / `junit-platform.properties`), paths relative to it
- [x] `PRS_SPEC_FILES`, `PRS_COVERAGE`, `PRS_ARTIFACTS`, `PRS_MODE`
- [x] Production emitter package (#48)
- [x] Conformance runner

### Rust

- [x] Packaging per [Packaging and distribution](#packaging-and-distribution): one `p-rs` crate with features (#64)
- [x] In-process checking; strict default true; `check_hot`
- [x] `emit(event, key, payload)` argument order
- [x] `Model` per [Rust `Model`](#rust-model): paths, config, several specs, `exclude`
- [x] `SpecViolation` / `SetupError` with structured fields and `kind` (#53)
- [x] `expect_violation(spec, match)`
- [x] Failure output in Python's format, with `src` via `#[track_caller]`
- [x] Shared, thread-safe suite coverage; `PRS_COVERAGE` (#53)
- [x] `events_dir()`, `env()`, `add_events_dir`, `add_log` (#49), reading folders through a core helper
- [x] `unknown_events`, default `error` (#47)
- [x] Fixture pattern (`model.check(|recording| ...)`); `PRS_MODE`
- [x] Production emitter (#48): `prs::emit` (feature `emit`)
- [x] Conformance runner
