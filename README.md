# p-rs

A Rust implementation of the [P](https://github.com/p-org/P) language toolchain:
parser, type checker and model checker in one `prs` binary, with no .NET, Java or
code generation.

`prs` is a drop-in replacement for the upstream `p` tool. It reads `.pproj` files,
accepts the same options, and writes the same console output, traces and output folders.
It interprets P models directly instead of compiling them to C#, so there is no
`p compile` step.

Try it without installing anything in the [P Playground](https://crowecawcaw.github.io/p-rs/).

## Status

Early, but checks real models. `prs` passes upstream's compiler regression tests
(424), checker suites (249) and tutorials, and reproduces upstream's golden traces and
PObserve outputs. See [docs/development.md](docs/development.md#validation) for how this
is tested.

## Install

There are no prebuilt binaries yet. With a Rust toolchain:

```sh
cargo install --git https://github.com/crowecawcaw/p-rs prs-cli
```

## Usage

Run `prs` in a folder with a `.pproj` file, as you would `p`:

```sh
prs check -tc tcMyTest -s 1000               # 1000 random schedules
prs check -tc tcMyTest --replay PCheckerOutput/tcMyTest/BugFinding/MyProject_0_0.schedule
prs check -pf model.p -tc tcMyTest           # .p files instead of a .pproj
prs check --mode pex -tc tcMyTest            # exhaustive search
```

`prs check` type-checks the sources itself and reports errors as `p compile` does.
Bugs are written to `PCheckerOutput/` as upstream writes them.

PEx mode (`--mode pex`) explores every scheduling choice and every `$` / `choose`
value by depth-first search, skipping states it has already seen. Its report says
whether the search finished or hit the `--max-steps` bound. Time tracking is not
supported in this mode yet.

`prs observe` replaces [PObserve](https://github.com/p-org/P/tree/master/Src/PObserve):
it checks logs from a running system against a project's spec monitors, without jars.

```sh
prs observe --spec BankBalanceCorrect -l logs/
```

See [docs/observe.md](docs/observe.md) for log formats and custom parsers, and
[docs/library.md](docs/library.md) for using the crates from Rust.

## Goals

1. Simplify or eliminate the toolchain for working with P models.
2. Stay compatible with the official P tools: exact, drop-in compatibility where
   feasible; functional compatibility where expedient; anything else is listed as out of
   scope. See [Compatibility](#compatibility).

## Compatibility

**Exact** (drop-in):

* `prs check`, `prs merge-scenario-coverage` and `prs observe` options, help, console
  output and exit codes.
* Parse and type error messages.
* Bug reports, assertion and liveness messages.
* Trace files (`.txt`, `.trace.json`, `.schedule`), coverage reports, output folder
  layout and history rotation.
* `.pproj` files.
* PObserve output files (replay logs, `ParserError_N.txt`, `UnknownError_N.txt`,
  `PObserveMetrics.txt`).

**Functional** (same result, details differ):

* Findings may be reported in a different order.
* Source discovery is logged differently, since there is no compiled output to find.
* Files are enumerated in sorted order; elapsed time and memory figures differ.
* An action that runs forever without reaching a scheduling point is reported as a bug
  (upstream hangs).
* `prs observe` reports a null dereference with its own message.
* Foreign functions and log parsers are written in Rust, not C# or Java.

**Not supported yet**:

* PVerifier mode.
* `prs observe` event filters (`--filters`), socket mode and `SERIALIZEDEVENTS` input.
* `--graph` and `--graph-bug` are accepted but, as upstream, write nothing.

**Out of scope**:

* `p compile` and its output (`PGenerated/`, `.dll`, jars). `prs compile` prints a
  warning and exits 0; `OutputDir` and `Target` in `.pproj` files are ignored.
* .NET and Java stack frames in exception reports, traces and `UnknownError_N.txt`.
* Foreign code in C# or Java, and PObserve jars (`--jars`).

## Contributing

See [docs/development.md](docs/development.md) for the crate layout, tests and
benchmarks, and [AGENTS.md](AGENTS.md) for the commands to run before sending a change.

## License

MIT.
