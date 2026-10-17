# p-next

A pure Rust implementation of the [P](https://github.com/p-org/P) modeling language
toolchain: parser, type checker and model checker. No .NET and no code generation: P
models are checked directly by an interpreter driven by a controlled, reproducible
scheduler, with the same bug-finding semantics as the upstream checker.

## Crates

| Crate | Purpose |
|---|---|
| `pnext-syntax` | Lexer, parser and AST. |
| `pnext-sema` | Name resolution, type checking, machine and module-system rules; produces a typed program. Diagnostics match upstream's. |
| `pnext-checker` | Values, bytecode interpreter, scheduling strategies (random, pct, fairpct, probabilistic, pos, feedback, feedbackpct, feedbackpos, rl, replay), exhaustive search (PEx mode), safety and liveness monitors, deadlock detection, text/JSON traces, foreign functions, runtime monitoring of event logs (`observe`). |
| `pnext-compat` | Drop-in compatibility with the upstream `p` tool: `p compile` / `p check` / `p merge-scenario-coverage` options, `p observe` as a drop-in for PObserve, help, `.pproj` files, console output, output folders, trace files and exit codes. Runs against a pluggable host (native or in-memory). |
| `pnext-cli` | The `p` binary. |
| `pnext-wasm` | WebAssembly module plus a JavaScript wrapper to run `p` in a browser or Node.js. |

Try it in the browser: the [P Playground](https://crowecawcaw.github.io/p-rs/) (source in `web/`).

The core crates use no file system, process or threading APIs and build for
`wasm32-unknown-unknown` unchanged.

## Using the `p` tool

```sh
cargo build --release -p pnext-cli      # produces target/release/p
cd MyProject                            # a folder with a .pproj file
p compile
p check -tc tcMyTest -s 1000
p check -tc tcMyTest --replay PCheckerOutput/tcMyTest/BugFinding/MyProject_0_0.schedule
```

Options, messages, output layout (`PCheckerOutput/<test>/BugFinding/` with history
rotation) and trace files (`.txt`, `.trace.json`, `.schedule`) follow upstream. `p compile`
writes `PGenerated/PChecker/<Project>.pnext`, a self-contained snapshot of the
type-checked sources, which `p check` finds the way upstream finds its `.dll` (an
upstream-style `.dll` path is also accepted and mapped to the `.pnext` next to it).

### Exhaustive checking (PEx mode)

```sh
p compile --mode pex                    # same compiled file, written to PGenerated/PEx/
p check --mode pex -tc tcMyTest         # --sch-pex dfs for a fixed order; -s / -t to bound it
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

## Checking logs: `p observe`

`p observe` is [PObserve](https://github.com/p-org/P/tree/master/Src/PObserve): it
checks events recorded by a running system against the spec monitors of a P project.

```sh
cd MyProject                            # a folder with a .pproj file
p observe --spec BankBalanceCorrect -l logs/            # JSON lines, see below
p observe --spec BankBalanceCorrect -l trace.json -i JSON   # a P JSON trace
p observe -pf PSpec/Bank.p --spec BankBalanceCorrect -l app.log --keys 7 9
```

Options, console output, `PObserve-<date>/` output (replay logs, `ParserError_N.txt`,
`UnknownError_N.txt`, `PObserveMetrics.txt`) and exit codes (0 ok, 1 command line,
2 spec violation, 3 internal error, 4 parser errors only) follow upstream. The difference
is where the spec comes from: there is no `p compile --mode pobserve` and no `--jars`.
The P sources are type-checked in memory, from the `.pproj` in the current folder,
`-pp/--pproj <file>` or `-pf/--pfiles <files>`.

Parsers turn log entries into events. Two are built in:

* `JsonLines` (default for `-i TEXT`): one JSON object per line,
  `{"key": "7", "timestamp": 1700000000000, "event": "eWithDraw", "payload": {...}}`.
  The timestamp is epoch milliseconds or ISO-8601; payloads are matched against the
  event's payload type. `-pc '{"key": "ctx.account", "event": "type"}'` remaps the
  fields (dotted paths).
* `PTrace` (default for `-i JSON`): the `.trace.json` of `p check`, checking its sends
  and announcements.

Other log formats need a parser in Rust: implement `pnext_checker::observe::Parser` and
run `pnext_compat::observe::run_with(args, host, parsers)` (see
`crates/pnext-cli/examples/observe_regression.rs`, which ports upstream's regression
parsers).

## Using the library

```rust
let mut sm = pnext_syntax::SourceMap::new();
sm.add("model.p", source);
let analysis = pnext_sema::compile(&sm, &pnext_sema::Options::default())?;
let tc = analysis.program.test_cases().into_iter().find(|t| t.name == "tcMain").unwrap();
let checker = pnext_checker::engine::Checker::new(&analysis.program);
let opts = pnext_checker::engine::CheckerOptions { iterations: 1000, seed: 42, ..Default::default() };
let result = checker.check(&tc, &opts, &mut Vec::<String>::new())?;
println!("{} bug(s)", result.report.num_bugs);
```

Logs are checked with `pnext_checker::observe::Spec::new(&program, "MySpec")?.observe(...)`,
which takes the log entries, a `Parser` and `ObserveOptions` and returns a `Report`.

Foreign types and functions are supplied in Rust through
`pnext_checker::foreign::ForeignFunctions` (see `crates/pnext-checker/tests/foreign.rs`,
which ports the PriorityQueue tutorial's C# code).

## In the browser

```sh
rustup target add wasm32-unknown-unknown
cargo build --profile wasm --target wasm32-unknown-unknown -p pnext-wasm
node crates/pnext-wasm/js/smoke.mjs target/wasm32-unknown-unknown/wasm/pnext_wasm.wasm
```

`crates/pnext-wasm/js/pnext.mjs` exposes `run(args, files)` over an in-memory file
system; the module is about 1 MB (about 320 KB gzipped).

## Validation

* `pnext-sema`: all 424 upstream regression tests compile or fail to compile as
  upstream expects (`cargo run --release -p pnext-sema --example corpus -- tests/upstream/RegressionTests`).
* `pnext-checker`: all 249 tests of the suites upstream's harness runs find a bug exactly
  when upstream expects one, with upstream's settings (1000 schedules, 1000 steps)
  across seeds (`cargo run --release -p pnext-checker --example corpus -- <suite dir>`);
  a faster version runs in `cargo test`.
* Tutorials: `cargo run --release -p pnext-checker --example tutorials -- tests/upstream/Tutorial`
  mirrors upstream's tutorial CI and reports the same three failing test cases.
* Traces: the readable and JSON traces contain upstream's golden files
  (`tests/upstream/CorrectLogs`), checked in `crates/pnext-compat/tests/cli.rs`.
* `p observe`: `scripts/compare-pobserve.py` runs upstream PObserve and `p observe` on
  all 96 logs of upstream's PObserve regression suite, with and without
  `--assumeInputsAreSorted`. All 192 runs give the same exit code, and 186 give identical
  console output and output files. The other 6 differ only in a null-pointer message (see
  below). The small logs and upstream's outputs for them are vendored in
  `tests/upstream/PObserve` and checked in `crates/pnext-compat/tests/observe_upstream.rs`.

## Known differences from upstream

* Not implemented: the rl strategy and the PEx and PVerifier modes. Options for them are
  accepted; choosing an unimplemented strategy reports an error. `--graph` and `--graph-bug` are accepted and, as upstream,
  write nothing.
* .NET stack frames (in exception reports and in `ExceptionThrown` / `ExceptionHandled`
  trace elements) cannot be reproduced and are omitted.
* Foreign code in C# or Java cannot run; provide foreign functions in Rust.
* An action that runs forever without reaching a scheduling point is reported as a bug
  (upstream hangs).
* File enumeration order is sorted (upstream uses the OS order), and elapsed time and
  memory figures naturally differ.
* `p observe`: specs come from P sources, not jars (no `--jars`), and parsers are written
  in Rust. Event filters (`--filters`), socket mode and `SERIALIZEDEVENTS` input are not
  supported and are reported as command-line errors. Java stack frames are omitted from
  `UnknownError_N.txt`. Java's message for a null dereference names a variable of the
  generated Java code (`because "TMP_tmp2_12" is null`). That name cannot be reproduced,
  so `p observe` reports a null dereference with its own message.
