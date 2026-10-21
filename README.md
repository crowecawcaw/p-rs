# Snap Pea

A Rust implementation of the [P](https://github.com/p-org/P) language toolchain:
parser, type checker and model checker, with no .NET, Java or code generation. It
interprets P models directly instead of compiling them to C#, so there is no compile
step.

Checking the Raft tutorial (120 schedules) takes 0.4 s with snappea and 38 s with P 3.1.0,
including 4 s to compile; warmed up, snappea checks 290 schedules per second and P 4
([benchmarks](docs/development.md#benchmarks)).

There are two command line tools:

* `snappea`, with its own options, parallel workers and time tracking.
* `snappea-compat`, a drop-in replacement for the upstream `p` tool. It reads `.pproj`
  files, accepts the same options, and writes the same console output, traces and
  output folders.

Try it without installing anything in the [P Playground](https://crowecawcaw.github.io/snappea/playground.html),
or start with the [overview](https://crowecawcaw.github.io/snappea/docs/overview.html) of what P is and how it
differs from unit tests.

## Status

Early, but checks real models. snappea passes upstream's compiler regression tests
(424), checker suites (249) and tutorials, and reproduces upstream's golden traces and
PObserve outputs. See [docs/development.md](docs/development.md#validation) for how this
is tested.

## Install

```sh
cargo install snappea   # snappea and snappea-compat, built with a Rust toolchain
pip install pysnappea     # snappea, prebuilt, with the Python helper
```

Each [GitHub release](https://github.com/crowecawcaw/snappea/releases) also has a prebuilt
`snappea` for Linux (x86-64, ARM), macOS (Intel, Apple silicon) and Windows, with a
`SHA256SUMS` file. To build the latest `main` instead:
`cargo install --git https://github.com/crowecawcaw/snappea snappea`.

The application helpers are one package per language, all named `snappea`:

| Language | Install |
| --- | --- |
| Python | `pip install pysnappea` |
| Rust | `cargo add snappea` |
| Go | `go get github.com/crowecawcaw/snappea` |
| Java | Maven `io.github.crowecawcaw:snappea`, plus a `natives-<os>-<arch>` jar ([snippet](https://crowecawcaw.github.io/snappea/docs/java.html#install)) |

## Usage

Run `snappea` in a folder with a `.pproj` file:

```sh
snappea tests                                   # list the test cases
snappea check                                   # check every test case, 10000 schedules each
snappea check tcMyTest -n 1000                  # one test case, 1000 schedules
snappea check 'tcMy*' --time 5m                 # test cases matching a glob, for 5 minutes each
snappea check model.p tcMyTest                  # .p files instead of a .pproj
snappea check tcMyTest --strategy pct:10        # another search strategy
snappea check tcMyTest --exhaustive             # every scheduling and data choice
snappea replay PCheckerOutput/tcMyTest/bug.schedule
```

`snappea check` type-checks the sources itself and reports errors as `p compile` does.
It prints one line per test case and, for a bug, its report and the `snappea replay`
command that reproduces it:

```
$ snappea check -n 300
Checking TwoPhaseCommit: 10 test cases
ok    tcSingleClientNoFailure (300 schedules, 8 workers, 0.06s)
FAIL  tcMultipleClientsWithFailure (bug found after 23 schedules, 8 workers, 0.02s)
      Deadlock detected. Client(6) is waiting to receive an event, but no other controlled tasks are enabled.
      Replay: snappea replay PCheckerOutput/tcMultipleClientsWithFailure/bug.schedule
...
5 of 10 test cases failed in 0.98s, peak memory 18 MB. Output: PCheckerOutput/
```

Each test case's files go to `PCheckerOutput/<test>/`, replacing the previous run's:
`bug.txt` (the buggy schedule's log), `bug.schedule`, `bug.trace.json`, `coverage.txt`,
`summary.txt` and, for scenario monitors, `scenario_coverage.json`. `-v` adds
upstream's progress and statistics. It exits with 0 when no bug is found, 1 when one
is, and 2 on any other error. `snappea <command> --help` lists the options.

`snappea check` explores schedules on one worker thread per CPU core (`-j N` sets the
number). The schedule count is split between the workers, each with its own seed; the
first bug stops them all, and its `bug.schedule` file replays it as usual. The `random`,
`pct`, `fairpct`, `prob` and `pos` strategies use workers; the feedback strategies and
`rl` (which learn from earlier schedules), `--exhaustive` and `--verbose` run one
worker. With more than one worker, a given `--seed` explores different schedules than
upstream, and which bug is reported can vary from run to run; `-j 1` explores exactly
upstream's schedules.

`--exhaustive` explores every scheduling choice and every `$` / `choose` value by
depth-first search, skipping states it has already seen. Its report says whether the
search finished or hit the `--max-steps` bound. Time tracking is not supported in this
mode yet.

Time tracking (`--latency <file>[:<profile>]`, `--timed`) checks timeouts and latency
budgets; see [design/time](design/time/RetryBudget/README.md).

`snappea observe` replaces [PObserve](https://github.com/p-org/P/tree/master/Src/PObserve):
it checks logs from a running system against a project's spec monitors, without jars.

```sh
snappea observe --spec BankBalanceCorrect logs/
```

See [docs/observe.md](docs/observe.md) for log formats and custom parsers, and
[docs/library.md](docs/library.md) for using the crates from Rust.
[Testing and production](docs/production.md) shows how the same `emit` calls feed
both test checks and `snappea observe` on production logs.

To use snappea in place of upstream P, in scripts written for `p`, run `snappea-compat` (or
alias it to `p`). It takes upstream's options and writes upstream's output
([Compatibility](#compatibility)):

```sh
snappea-compat check -tc tcMyTest -s 1000
```

From Python, the `snappea` package ([`python/`](python)) checks specs against events your
code emits during its ordinary pytest tests, and installs the `snappea` binary. See
the [Python guide](https://crowecawcaw.github.io/snappea/docs/python.html) and the runnable
[inventory integration examples](examples/python/README.md).

From Rust, the [`snappea` crate](crates/snappea/README.md) checks in process:
[`snappea::test`](crates/snappea/src/test/README.md) has recordings configured from
`Cargo.toml`, structured violations, events from child processes, and suite coverage.
[`snappea::emit`](crates/snappea/src/emit/README.md) emits the same events in tests and to
production logs for `snappea observe`; with only the `emit` feature, no checker is compiled. See the
[Rust guide](https://crowecawcaw.github.io/snappea/docs/rust.html) and the [Rust inventory examples](examples/rust/README.md) for explicit checks, checked
fixtures, and generated operation sequences.

From Java, the [`snappea` artifact](https://crowecawcaw.github.io/snappea/docs/java.html) provides a dependency-free emitter
with production sinks, explicit recordings, and automatic JUnit checks with fixture
lifecycle and coverage support; its `natives-<os>-<arch>` classifier jars carry the
`snappea` binary. The [Java inventory examples](examples/java/README.md) cover explicit
checks, a child process, JUnit fixtures, and jqwik sequences.

From Go, [`snappeatest`](https://crowecawcaw.github.io/snappea/docs/go.html) records application events and integrates checks
with `testing.T`, subtests, and fuzz callbacks; `snappeaemit` emits from processes a test
starts and writes production logs. The
[Go inventory examples](examples/go/README.md) show explicit checks, cleanup fixtures,
native fuzzing and child processes. `snappeatest` downloads the matching `snappea` release
binary on first use.

All helpers follow one set of names, defaults and behaviours:
[docs/language-support.md](docs/language-support.md), checked by the shared
[conformance corpus](tests/conformance/README.md).

## Goals

Make P easier for developers to incorporate into application design and development.
The tenets behind that goal are in [AGENTS.md](AGENTS.md#tenets).

## Compatibility

This section is about `snappea-compat`. `snappea` checks the same way, finds the same bugs
and writes the same trace formats, but does not imitate upstream where it doesn't have
to:

* Options: its own, shorter ones (`snappea <command> --help`).
* Console: one line per test case, the bug's report and its `snappea replay` command, and a
  summary with time and peak memory. Upstream's progress only with `-v`.
* Output folder: `PCheckerOutput/<test>/` with fixed names (`bug.txt`, `bug.schedule`,
  `bug.trace.json`, `coverage.txt`, `summary.txt`, `scenario_coverage.json`), replaced by
  each run. No `BugFinding/` folder, history folders or `.sci` file.
* `<IncludeProject>` paths in a `.pproj` are resolved against the including project
  file's folder first, then the current folder (upstream: only the current folder).
* `observe`: sources and logs are positional arguments, live input stops at the first
  error without `--fail-fast`, and there is no report folder (use `--json-report`). Exit
  codes are 0, 1 and 2, as for `check`; see [docs/observe.md](docs/observe.md).

**Exact** (drop-in):

* `check`, `merge-scenario-coverage` and `observe` options, help, console output and
  exit codes.
* Parse and type error messages.
* Bug reports, assertion and liveness messages.
* Trace files (`.txt`, `.trace.json`, `.schedule`), coverage reports, output folder
  layout and history rotation.
* `.pproj` files.
* PObserve output files (replay logs, `ParserError_N.txt`, `UnknownError_N.txt`,
  `PObserveMetrics.txt`).

**Functional** (same result, details differ):

* Findings may be reported in a different order.
* `check` reads P sources, not a compiled `.dll`: a `.pproj` or `.p` file as its path,
  `--pproj`, `--pfiles` and `--projname` (upstream: options of `p compile`), or else the
  `.pproj` or `.p` files in the current folder.
* Source discovery is logged differently, since there is no compiled output to find.
* Files are enumerated in sorted order; elapsed time and memory figures differ.
* An action that runs forever without reaching a scheduling point is reported as a bug
  (upstream hangs).
* Nesting deeper than 128 levels is a parse error, `nesting is too deep` (upstream has
  no limit). Blocks, statements, expressions, types and each operand of an operator
  chain such as `a + b + c` count a level; the upstream tests nest at most 16 deep.
* `merge-scenario-coverage --help` prints its usage (upstream looks for a directory
  named `--help`).
* `observe` reports a null dereference with its own message.
* `observe` extensions: several names after `--spec`, `--all-specs`,
  `--exclude-specs`, `--json-report`, `--coverage`, `--strict`, `--unknown-events`,
  `--check-hot`, several `-l` values and glob patterns, and live input (`--stdin`,
  `--live`, `--sequence`, `--record`). Runs that use none of them are unchanged.
* Foreign functions and log parsers are written in Rust, not C# or Java.

**Not supported yet**:

* PVerifier mode.
* `observe` event filters (`--filters`), socket mode and `SERIALIZEDEVENTS` input.
* `--memout` is accepted and ignored.
* `--graph` and `--graph-bug` are accepted but, as upstream, write nothing.

**Out of scope**:

* `p compile` and its output (`PGenerated/`, `.dll`, jars). There is no `compile`
  command; `OutputDir` and `Target` in `.pproj` files are ignored.
* .NET and Java stack frames in exception reports, traces and `UnknownError_N.txt`.
* Foreign code in C# or Java, and PObserve jars (`--jars`).

## Contributing

See [docs/development.md](docs/development.md) for the crate layout, tests and
benchmarks, and [AGENTS.md](AGENTS.md) for the commands to run before sending a change.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT), at your option.

Parts of snappea, including the tests vendored in `tests/upstream`, are derived from
[P](https://github.com/p-org/P) and remain under
[P's MIT license](tests/upstream/LICENSE.txt) (Copyright (c) 2015 P Developers).

Unless you explicitly state otherwise, any contribution you submit for inclusion in
this project, as defined in the Apache-2.0 license, is dual licensed as above, without
any additional terms or conditions.
