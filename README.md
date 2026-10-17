# p-rs

A pure Rust implementation of the [P](https://github.com/p-org/P) modeling language
toolchain: parser, type checker and model checker. No .NET and no code generation: P
models are checked directly by an interpreter driven by a controlled, reproducible
scheduler, with the same bug-finding semantics as the upstream checker.

## Goals

1. Simplify or eliminate the toolchain for working with P models.
2. Stay compatible with the official P tools: exact, drop-in compatibility where
   feasible; functional compatibility where expedient; anything else is listed as out of
   scope. See [Compatibility](#compatibility).

## Crates

| Crate | Purpose |
|---|---|
| `prs-syntax` | Lexer, parser and AST. |
| `prs-sema` | Name resolution, type checking, machine and module-system rules; produces a typed program. Diagnostics match upstream's. |
| `prs-checker` | Values, bytecode interpreter, scheduling strategies (random, pct, fairpct, probabilistic, pos, feedback, feedbackpct, feedbackpos, rl, replay), exhaustive search (PEx mode), safety and liveness monitors, deadlock detection, text/JSON traces, foreign functions, runtime monitoring of event logs (`observe`). |
| `prs-compat` | Drop-in compatibility with the upstream `p` tool: `prs check` / `prs merge-scenario-coverage` options, `prs observe` as a drop-in for PObserve, help, `.pproj` files, console output, output folders, trace files and exit codes. Runs against a pluggable host (native or in-memory). |
| `prs-cli` | The `prs` binary. |
| `prs-wasm` | WebAssembly module plus a JavaScript wrapper to run `prs` in a browser or Node.js. |

Try it in the browser: the [P Playground](https://crowecawcaw.github.io/p-rs/) (source in `web/`).

The core crates use no file system, process or threading APIs and build for
`wasm32-unknown-unknown` unchanged.

## Using the `prs` tool

```sh
cargo build --release -p prs-cli      # produces target/release/prs
cd MyProject                            # a folder with a .pproj file
prs check -tc tcMyTest -s 1000
prs check -tc tcMyTest --replay PCheckerOutput/tcMyTest/BugFinding/MyProject_0_0.schedule
prs check -pf model.p -tc tcMyTest          # P files instead of a .pproj
```

There is no compile step: `prs check` reads and type-checks the P sources itself, from
the `.pproj` in the current folder, `-pp/--pproj <file>`, `-pf/--pfiles <files>` (with
`-pn/--projname` to name the output), or else the `*.p` files in the current folder.
Type errors are reported as `p compile` reports them. Options, messages, output layout
(`PCheckerOutput/<test>/BugFinding/` with history rotation) and trace files (`.txt`,
`.trace.json`, `.schedule`) follow upstream.

### Exhaustive checking (PEx mode)

```sh
prs check --mode pex -tc tcMyTest         # --sch-pex dfs for a fixed order; -s / -t to bound it
```

PEx mode explores every behaviour of the test (every scheduling choice and every `$` /
`choose` value) by depth-first search, skipping program states it has already explored.
The report says whether the search finished, so "no bug found" means there is none within
the step bound (`--max-steps`), or that it stopped early. Bugs produce the usual trace and
`.schedule` files, which `--replay` reproduces. Liveness is checked for schedules that
end; time tracking is not supported in this mode yet.

`--sch-rl` selects the Q-learning scheduler (from Coyote): random testing that favours
choices leading to program states it has seen less often. Runs are reproducible from
`--seed`.

## Checking logs: `prs observe`

`prs observe` is [PObserve](https://github.com/p-org/P/tree/master/Src/PObserve): it
checks events recorded by a running system against the spec monitors of a P project.

```sh
cd MyProject                            # a folder with a .pproj file
prs observe --spec BankBalanceCorrect -l logs/            # JSON lines, see below
prs observe --spec BankBalanceCorrect -l trace.json -i JSON   # a P JSON trace
prs observe -pf PSpec/Bank.p --spec BankBalanceCorrect -l app.log --keys 7 9
```

Options, console output, `PObserve-<date>/` output (replay logs, `ParserError_N.txt`,
`UnknownError_N.txt`, `PObserveMetrics.txt`) and exit codes (0 ok, 1 command line,
2 spec violation, 3 internal error, 4 parser errors only) follow upstream. The difference
is where the spec comes from: there are no jars (no `--jars`). The P sources are
type-checked in memory, from the `.pproj` in the current folder,
`-pp/--pproj <file>` or `-pf/--pfiles <files>`.

Parsers turn log entries into events. Two are built in:

* `JsonLines` (default for `-i TEXT`): one JSON object per line,
  `{"key": "7", "timestamp": 1700000000000, "event": "eWithDraw", "payload": {...}}`.
  The timestamp is epoch milliseconds or ISO-8601; payloads are matched against the
  event's payload type. `-pc '{"key": "ctx.account", "event": "type"}'` remaps the
  fields (dotted paths).
* `PTrace` (default for `-i JSON`): the `.trace.json` of `prs check`, checking its sends
  and announcements.

Other log formats need a parser in Rust: implement `prs_checker::observe::Parser` and
run `prs_compat::observe::run_with(args, host, parsers)` (see
`crates/prs-cli/examples/observe_regression.rs`, which ports upstream's regression
parsers).

## Using the library

```rust
let mut sm = prs_syntax::SourceMap::new();
sm.add("model.p", source);
let analysis = prs_sema::compile(&sm, &prs_sema::Options::default())?;
let tc = analysis.program.test_cases().into_iter().find(|t| t.name == "tcMain").unwrap();
let checker = prs_checker::engine::Checker::new(&analysis.program);
let opts = prs_checker::engine::CheckerOptions { iterations: 1000, seed: 42, ..Default::default() };
let result = checker.check(&tc, &opts, &mut Vec::<String>::new())?;
println!("{} bug(s)", result.report.num_bugs);
```

Logs are checked with `prs_checker::observe::Spec::new(&program, "MySpec")?.observe(...)`,
which takes the log entries, a `Parser` and `ObserveOptions` and returns a `Report`.

Foreign types and functions are supplied in Rust through
`prs_checker::foreign::ForeignFunctions` (see `crates/prs-checker/tests/foreign.rs`,
which ports the PriorityQueue tutorial's C# code).

## In the browser

```sh
rustup target add wasm32-unknown-unknown
cargo build --profile wasm --target wasm32-unknown-unknown -p prs-wasm
node crates/prs-wasm/js/smoke.mjs target/wasm32-unknown-unknown/wasm/prs_wasm.wasm
```

`crates/prs-wasm/js/prs.mjs` exposes `run(args, files)` over an in-memory file
system; the module is about 1 MB (about 320 KB gzipped).

## Validation

* `prs-sema`: all 424 upstream regression tests compile or fail to compile as
  upstream expects (`cargo run --release -p prs-sema --example corpus -- tests/upstream/RegressionTests`).
* `prs-checker`: all 249 tests of the suites upstream's harness runs find a bug exactly
  when upstream expects one, with upstream's settings (1000 schedules, 1000 steps)
  across seeds (`cargo run --release -p prs-checker --example corpus -- <suite dir>`);
  a faster version runs in `cargo test`.
* Tutorials: `cargo run --release -p prs-checker --example tutorials -- tests/upstream/Tutorial`
  mirrors upstream's tutorial CI and reports the same three failing test cases.
* Traces: the readable and JSON traces contain upstream's golden files
  (`tests/upstream/CorrectLogs`), checked in `crates/prs-compat/tests/cli.rs`.
* `prs observe`: `scripts/compare-pobserve.py` runs upstream PObserve and `prs observe` on
  all 96 logs of upstream's PObserve regression suite, with and without
  `--assumeInputsAreSorted`. All 192 runs give the same exit code, and 186 give identical
  console output and output files. The other 6 differ only in a null-pointer message (see
  below). The small logs and upstream's outputs for them are vendored in
  `tests/upstream/PObserve` and checked in `crates/prs-compat/tests/observe_upstream.rs`.

## Benchmarks

`scripts/bench.py` runs `prs check` on six tutorial tests with fixed schedule counts and
seed, and records total check time, schedules per second, compile time and (with
`--instructions`, via Valgrind) instructions executed:

```sh
cargo build --release -p prs-cli
python3 scripts/bench.py run --out bench.json
python3 scripts/bench.py report --current bench.json --history old.jsonl
```

* Every commit on main: `.github/workflows/bench.yml` appends the result to `main.jsonl`
  on the `benchmarks` branch and reports the change from the previous commit in the run
  summary. Charts: [benchmarks page](https://crowecawcaw.github.io/p-rs/bench/).
  Times are wall clock on a shared runner (about ±10% noise); the instruction counts,
  stable to about 0.2%, tell a real change from noise.
* Against upstream P: run the "Benchmark vs upstream P" workflow by hand. It installs P
  from NuGet and runs both tools on the same runner and workloads, and appends the
  comparison to `upstream.jsonl`.

## Compatibility

**Exact** (drop-in):

* `prs check`, `prs merge-scenario-coverage` and `prs observe` options, help, console
  output and exit codes.
* Parse and type error messages.
* Finding messages: bug reports, assertion and liveness messages.
* Trace files (`.txt`, `.trace.json`, `.schedule`), coverage reports, output folder
  layout and history rotation.
* `.pproj` files.
* PObserve output files (replay logs, `ParserError_N.txt`, `UnknownError_N.txt`,
  `PObserveMetrics.txt`).

**Functional** (same result, details differ):

* The order of findings may differ.
* Source discovery is logged differently (prs loads the `.pproj` itself; there is no
  compiled file to search for).
* File enumeration order is sorted (upstream uses the OS order); elapsed time and memory
  figures differ.
* An action that runs forever without reaching a scheduling point is reported as a bug
  (upstream hangs).
* `prs observe` reports a null dereference with its own message (Java's names a variable
  of the generated code, e.g. `because "TMP_tmp2_12" is null`).
* Foreign functions are written in Rust (`prs_checker::foreign`), log parsers too
  (`prs_checker::observe::Parser`).

**Not supported** (yet):

* PVerifier mode.
* `prs observe`: event filters (`--filters`), socket mode and `SERIALIZEDEVENTS` input
  (reported as command-line errors).
* `--graph` and `--graph-bug` are accepted and, as upstream, write nothing.

**Out of scope**:

* `p compile` and compiled output (`PGenerated/`, `.dll`, jars). `prs check` and
  `prs observe` read P sources directly; `OutputDir` and `Target` in `.pproj` files are
  ignored.
* .NET stack frames (in exception reports and `ExceptionThrown` / `ExceptionHandled`
  trace elements) and Java stack frames (in `UnknownError_N.txt`): prs does not use .NET
  or Java.
* Running foreign code in C# or Java, and PObserve jars (`--jars`).
