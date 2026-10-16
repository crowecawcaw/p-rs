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
| `pnext-checker` | Values, bytecode interpreter, scheduling strategies (random, pct, fairpct, probabilistic, pos, feedback, feedbackpct, feedbackpos, replay), safety and liveness monitors, deadlock detection, text/JSON traces, foreign functions. |
| `pnext-compat` | Drop-in compatibility with the upstream `p` tool: `p compile` / `p check` / `p merge-scenario-coverage` options, help, `.pproj` files, console output, output folders, trace files and exit codes. Runs against a pluggable host (native or in-memory). |
| `pnext-cli` | The `p` binary. |
| `pnext-wasm` | WebAssembly module plus a JavaScript wrapper to run `p` in a browser or Node.js. |

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
