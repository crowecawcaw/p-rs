# Using p-rs from Rust

The crates are not published yet; depend on them by git.

For runnable application integration, see the [Rust inventory examples](../examples/rust/README.md):
explicit checks, a shared test fixture, and generated operation sequences with proptest.

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

`check` runs on the calling thread. To use several threads, create a
`prs_checker::engine::WorkerPool`, run `Checker::explore` for each worker on its own
thread (each with its own `Checker` of the shared program) and merge the results with
`Checker::finish`; `run_workers` in `crates/prs-compat/src/check.rs` does this.

Logs are checked with `prs_checker::observe::Spec::new(&program, "MySpec")?.observe(...)`,
which takes the log entries, a `Parser` and `ObserveOptions` and returns a `Report`.

Foreign types and functions are supplied through
`prs_checker::foreign::ForeignFunctions`. `crates/prs-checker/tests/foreign.rs` is an
example; it ports the PriorityQueue tutorial's C# code. `crates/prs-checker/tests/library.rs`
runs the example above.
