# Using p-rs from Rust

The checker is part of the [`p-rs` crate](../crates/p-rs/README.md) (library name `prs`;
`p-rs = "0.1"` with default features): `prs::syntax`, `prs::sema`, `prs::checker`,
`prs::project` and `prs::driver`. To check application events from Rust tests, use `prs::test` instead; see
the [Rust examples](../examples/rust/README.md).

```rust
let mut sm = prs::syntax::SourceMap::new();
sm.add("model.p", source);
let analysis = prs::sema::compile(&sm, &prs::sema::Options::default())?;
let tc = analysis.program.test_cases().into_iter().find(|t| t.name == "tcMain").unwrap();
let checker = prs::checker::engine::Checker::new(&analysis.program);
let opts = prs::checker::engine::CheckerOptions { iterations: 1000, seed: 42, ..Default::default() };
let result = checker.check(&tc, &opts, &mut Vec::<String>::new())?;
println!("{} bug(s)", result.report.num_bugs);
```

`check` runs on the calling thread. To use several threads, create a
`prs::checker::engine::WorkerPool`, run `Checker::explore` for each worker on its own
thread (each with its own `Checker` of the shared program) and merge the results with
`Checker::finish`; `run_workers` in `crates/p-rs/src/driver/check.rs` does this.

Logs are checked with `prs::checker::observe::Spec::new(&program, "MySpec")?.observe(...)`,
which takes the log entries, a `Parser` and `ObserveOptions` and returns a `Report`.

Foreign types and functions are supplied through
`prs::checker::foreign::ForeignFunctions`. `crates/p-rs/tests/checker_foreign.rs` is an
example; it ports the PriorityQueue tutorial's C# code. `crates/p-rs/tests/checker_library.rs`
runs the example above.
