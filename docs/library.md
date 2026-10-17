# Using p-rs from Rust

The crates are not published yet; depend on them by git.

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

Foreign types and functions are supplied through
`prs_checker::foreign::ForeignFunctions`. `crates/prs-checker/tests/foreign.rs` is an
example; it ports the PriorityQueue tutorial's C# code.
