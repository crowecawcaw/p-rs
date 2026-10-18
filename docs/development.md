# Development

The commands to build, test and lint are in [AGENTS.md](../AGENTS.md).

## Crates

| Crate | Purpose |
|---|---|
| `prs-syntax` | Lexer, parser and AST. |
| `prs-sema` | Name resolution and type checking; produces a typed program. |
| `prs-checker` | Interpreter, schedulers, PEx search, monitors, traces, foreign functions and log checking. |
| `prs-compat` | The upstream-compatible command line: options, `.pproj` files, console output, output folders and exit codes; also the steps of a check (sources, type checking, one test case, replay) for other front ends. Runs against a pluggable host (native or in-memory). |
| `prs-cli` | The `prs` command line, the native host, and the `prs` and `prs-compat` binaries. |
| `prs-wasm` | WebAssembly module and JavaScript wrapper. |
| `p-rs` | Placeholder that reserves the name on crates.io. |

The core crates use no file system, process or threading APIs, so they build for
`wasm32-unknown-unknown` unchanged. `prs-compat` reaches all of these through its `Host`;
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
  `cargo run --release -p prs-sema --example typecheck_corpus -- tests/upstream/RegressionTests`.
* Checker: all 249 tests of the suites upstream's harness runs find a bug exactly when
  upstream expects one, with upstream's settings (1000 schedules, 1000 steps) across
  seeds. `cargo run --release -p prs-checker --example corpus -- <suite dir>`.
  A faster version runs in `cargo test`.
* Tutorials: mirrors upstream's tutorial CI and reports the same three failing tests.
  `cargo run --release -p prs-checker --example tutorials -- tests/upstream/Tutorial`
* Traces: readable and JSON traces match upstream's golden files
  (`tests/upstream/CorrectLogs`), checked in `crates/prs-compat/tests/cli.rs`.
* `prs observe`: `scripts/compare-pobserve.py` runs upstream PObserve and `prs observe`
  on all 96 logs of upstream's regression suite, with and without
  `--assumeInputsAreSorted`. All 192 runs give the same exit code; 186 give identical
  output, and the other 6 differ only in the null-dereference message. The small logs
  and upstream's outputs are vendored in `tests/upstream/PObserve` and checked in
  `crates/prs-compat/tests/observe_upstream.rs`.

## Benchmarks

`scripts/bench.py` runs `prs-compat check` on six tutorial tests with fixed schedule
counts and seed, on one worker, so that the schedules match upstream's and history;
`run --cmd target/release/prs --workers N` measures N workers. It records check time,
schedules per second, startup time (a one-schedule check) and warm schedules per second
(after startup) and, with `--instructions` (via Valgrind), instructions executed.

```sh
cargo build --release -p prs-cli
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
