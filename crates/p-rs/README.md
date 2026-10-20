# p-rs

The [P](https://github.com/p-org/P) modeling language in Rust: a parser, type checker
and model checker with no .NET, Java or code generation, plus helpers to check the
events your application emits against P specs.

## The `prs` tool

```sh
cargo install p-rs        # installs prs and prs-compat
prs check                 # check every test case of the .pproj in this folder
prs observe --spec BankBalanceCorrect -l logs/   # check production logs
```

`prs-compat` is a drop-in replacement for upstream's `p` command.

## Library

`cargo add --dev p-rs` for tests. Production code needs only the emitter:

```toml
[dependencies]
p-rs = { version = "0.1", default-features = false, features = ["emit"] }

[dev-dependencies]
p-rs = "0.1"
```

```rust,ignore
// Application code. With only the `emit` feature, no checker is compiled in.
prs::emit::emit("eReserved", "coffee", prs::emit::json!({"order": "o1", "units": 2}));

// A test: everything emitted while the closure runs is checked against the P specs
// configured in [package.metadata.prs] of Cargo.toml.
let model = prs::test::Model::load()?;
model.check(|_recording| app.reserve("coffee", "o1", 2));
```

| Feature | Adds |
| --- | --- |
| `emit` | `prs::emit`: emit events to tests, `PRS_EVENTS_DIR` or production logs. Depends only on `serde` and `serde_json`. |
| `test` (default) | `prs::test`: in-process checks in `#[test]`s. Implies `emit`. |
| `cli` (default) | The `prs` and `prs-compat` binaries. |

`test` and `cli` also expose the checker as a library: `prs::syntax`, `prs::sema`,
`prs::checker`, `prs::project` and `prs::driver`.

## Documentation

- [The repository](https://github.com/crowecawcaw/p-rs): `prs` usage and compatibility with upstream P
- [Rust inventory examples](https://github.com/crowecawcaw/p-rs/tree/main/examples/rust)
- [Testing and production](https://github.com/crowecawcaw/p-rs/blob/main/docs/production.md)
- [Checking logs with `prs observe`](https://github.com/crowecawcaw/p-rs/blob/main/docs/observe.md)
- [The checker as a library](https://github.com/crowecawcaw/p-rs/blob/main/docs/library.md)

## License

MIT or Apache-2.0, at your option.
