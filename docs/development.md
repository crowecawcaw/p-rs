# Development

The commands to build, test and lint are in [AGENTS.md](../AGENTS.md).

## Crates

Everything published is one crate, `crates/p-rs` (library `prs`, on crates.io as
`p-rs`), split into modules:

| Module | Purpose |
|---|---|
| `prs::syntax` | Lexer, parser and AST. |
| `prs::sema` | Name resolution and type checking; produces a typed program. |
| `prs::checker` | Interpreter, schedulers, PEx search, monitors, traces, foreign functions and log checking. |
| `prs::host` | The environment commands run in: console, file system, clock, threads. `MemHost` is in-memory (tests, wasm); `prs::cli` has the native one. |
| `prs::project` | Finding and reading P sources: `.pproj` files, `--pfiles`, the current folder; type-error reporting. |
| `prs::driver` | The commands on a host, shared by `prs` and `prs-compat`: `check` (checking test cases, replay, output folders, traces, coverage), `observe` (checking logs: `compile`, `prepare`, `Observer::run`, `json_report`) and `scenario` (scenario coverage reports). `check::Style` picks the output: `Upstream` (upstream's folder layout, history and file names) or `Prs` (`<out>/<test>/bug.*`, replaced by each run). |
| `prs::compat` | Upstream's command line for `prs-compat` (`p check` options and help, `p merge-scenario-coverage`, PObserve's options and report folder), on `prs::driver`. Nothing else depends on it. |
| `prs::cli` | The `prs` command line and the native host (hidden from the docs; used by the binaries and `prs-wasm`). |
| `prs::test` | In-process checks for Rust tests. |
| `prs::emit` | The event emitter for application code. |

The binaries are `src/bin/prs.rs` and `src/bin/prs-compat.rs`. Features select what is
compiled: `emit` (only `prs::emit`, for production code), `test` (`prs::test`, `emit` and
the checker) and `cli` (the binaries and the checker); the default is `cli` and `test`.
Tests are in `crates/p-rs/tests`, named after the module they cover.

`crates/prs-wasm` (not published) is the WebAssembly module and JavaScript wrapper.

The checker modules use no file system, process or threading APIs, so they build for
`wasm32-unknown-unknown` unchanged. `prs::driver` reaches all of these through a `Host`;
the threads for `prs check --workers` come from `Host::threads`, which only the native
host provides, so the WebAssembly module (which runs the `prs` command line) uses one
worker.

## WebAssembly

```sh
rustup target add wasm32-unknown-unknown
cargo build --profile wasm --target wasm32-unknown-unknown -p prs-wasm
node crates/prs-wasm/js/smoke.mjs target/wasm32-unknown-unknown/wasm/prs_wasm.wasm
```

`crates/prs-wasm/js/prs.mjs` exposes `run(args, files)` over an in-memory file system.
The module is about 1 MB (320 KB gzipped). The playground in `web/` is built on it; see
[web/README.md](../web/README.md).

## Validation

Upstream's test suites are vendored in `tests/upstream`.

* Type checker: all 424 upstream regression tests compile or fail to compile as
  upstream expects. Checked in `cargo test`, or
  `cargo run --release -p p-rs --example typecheck_corpus -- tests/upstream/RegressionTests`.
* Checker: all 249 tests of the suites upstream's harness runs find a bug exactly when
  upstream expects one, with upstream's settings (1000 schedules, 1000 steps) across
  seeds. `cargo run --release -p p-rs --example corpus -- <suite dir>`.
  A faster version runs in `cargo test`.
* Tutorials: mirrors upstream's tutorial CI and reports the same three failing tests.
  `cargo run --release -p p-rs --example tutorials -- tests/upstream/Tutorial`
* Traces: readable and JSON traces match upstream's golden files
  (`tests/upstream/CorrectLogs`), checked in `crates/p-rs/tests/compat_cli.rs`.
* `prs-compat observe`: `scripts/compare-pobserve.py` runs upstream PObserve and
  `prs-compat observe` on all 96 logs of upstream's regression suite, with and without
  `--assumeInputsAreSorted`. All 192 runs give the same exit code; 186 give identical
  output, and the other 6 differ only in the null-dereference message. The small logs
  and upstream's outputs are vendored in `tests/upstream/PObserve` and checked in
  `crates/p-rs/tests/compat_observe_upstream.rs`.

## Benchmarks

`scripts/bench.py` runs `prs-compat check` on six tutorial tests with fixed schedule
counts and seed, on one worker, so that the schedules match upstream's and history;
`run --cmd target/release/prs --workers N` measures N workers. It records check time,
schedules per second, startup time (a one-schedule check) and warm schedules per second
(after startup) and, with `--instructions` (via Valgrind), instructions executed.

```sh
cargo build --release -p p-rs
python3 scripts/bench.py run --out bench.json
python3 scripts/bench.py report --current bench.json --history old.jsonl
```

* Every commit on main: `.github/workflows/bench.yml` appends the result to
  `main.jsonl` on the `benchmarks` branch and reports the change in the run summary.
  [Charts](https://crowecawcaw.github.io/p-rs/bench/). Wall-clock times on shared
  runners vary by about ±10%; instruction counts are stable to about 0.2% and are the
  better signal. GitHub also hands out runners with different CPUs, up to 40% apart in
  speed, so charts have one line per CPU model, and the run summary compares times
  only with the previous run on the same CPU (instructions with the previous run on
  any CPU).
* Against upstream P: run the "Benchmark vs upstream P" workflow by hand. It installs P
  from NuGet, runs both tools on the same runner and appends the comparison to
  `upstream.jsonl` on the `benchmarks` branch.
