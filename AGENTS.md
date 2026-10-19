# AGENTS.md

```sh
cargo build --workspace
cargo test --workspace
cargo fmt --all
cargo clippy --workspace --all-targets
```

## Goal

Make P easier for developers to incorporate into application design and development.

## Tenets

1. **The audience is implementors, not theorists.** Documentation, examples, tools and
   interfaces should focus on clarity, ergonomics and practicality.
2. **`prs-compat` is a drop-in for upstream P, and thin.** The `prs-compat` CLI should be
   as compatible with upstream P as possible, while keeping its own code minimal so the
   main functionality lives in the core library. Outside `prs-compat`, the library is
   shaped by the other tenets, not by upstream P.
3. **Agentic development is the primary use case.** P is especially well suited to it.
   Help developers understand what P is and how to use it strategically, and help agents
   incorporate P effectively into development for design and implementation validation.

For compatibility details, see [Compatibility](README.md#compatibility) in the README.
