# p-rs

A Rust implementation of the [P](https://github.com/p-org/P) language toolchain:
parser, type checker and model checker, with no .NET, Java or code generation. It
interprets P models directly instead of compiling them to C#, so there is no compile
step.

There are two command line tools:

* `prs`, with its own options, parallel workers and time tracking.
* `prs-compat`, a drop-in replacement for the upstream `p` tool. It reads `.pproj`
  files, accepts the same options, and writes the same console output, traces and
  output folders.

Try it without installing anything in the [P Playground](https://crowecawcaw.github.io/p-rs/playground.html),
or start with the [overview](https://crowecawcaw.github.io/p-rs/docs/overview.html) of what P is and how it
differs from unit tests.

## Install

```sh
cargo install p-rs   # prs and prs-compat, built with a Rust toolchain
pip install p-rs     # prs, prebuilt, with the Python helper
```

Each [GitHub release](https://github.com/crowecawcaw/p-rs/releases) also has a prebuilt
`prs` for Linux (x86-64, ARM), macOS (Intel, Apple silicon) and Windows, with a
`SHA256SUMS` file. To build the latest `main` instead:
`cargo install --git https://github.com/crowecawcaw/p-rs p-rs`.

## Usage

Run `prs` in a folder with a `.pproj` file:

```sh
prs check                                   # check every test case
prs check tcMyTest -n 1000                  # one test case, 1000 schedules
prs replay PCheckerOutput/tcMyTest/BugFinding/MyProject_0_0.schedule
prs observe --spec BankBalanceCorrect -l logs/   # check production logs
```

`prs <command> --help` lists the options. [docs/cli.md](docs/cli.md) covers search
strategies, parallel workers, exhaustive search, time tracking and `prs-compat`.

### Application helpers

The helpers check specs against events your code emits during its ordinary tests, and
write the same events to production logs for `prs observe`. Each is a package named
`p-rs`:

| Language | Install | Docs |
| --- | --- | --- |
| Python | `pip install p-rs` | [guide](docs/python.md), [examples](examples/python/README.md) |
| Rust | `cargo add p-rs` | [crate](crates/p-rs/README.md), [examples](examples/rust/README.md) |
| Go | `go get github.com/crowecawcaw/p-rs` | [guide](go/README.md), [examples](examples/go/README.md) |
| Java | Maven `io.github.crowecawcaw:p-rs` ([snippet](java/README.md#install)) | [guide](java/README.md), [examples](examples/java/README.md) |

All helpers share one set of names, defaults and behaviours
([docs/language-support.md](docs/language-support.md)).

## How p-rs compares to P

p-rs is focused on easy integration of P into real applications.

* **Faster, no toolchain.** There is no .NET, Java or compile step. Checking the Raft
  tutorial (120 schedules) takes 0.4 s with prs and 38 s with P 3.1.0
  ([benchmarks](docs/development.md#benchmarks)).
* **Drop-in compatible.** `prs-compat` accepts upstream's options and writes the same
  output, with a few [intentional differences](docs/compatibility.md): it reads P
  sources rather than a compiled `.dll`, foreign functions are written in Rust, and
  `p compile` and PVerifier are not supported.
* **Parity is measured** against upstream's compiler regression tests (424), checker
  suites (249), tutorials, golden traces and PObserve outputs
  ([validation](docs/development.md#validation)).
* **Beyond upstream.** Test helpers for Python, Rust, Go and Java; time tracking for
  timeouts and latency budgets; parallel workers; exhaustive search; and `prs observe`
  for logs without jars.

## Contributing

Issues and feature requests are welcome. Pull requests are not accepted.

## License

[Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option. Parts derived from
P, including `tests/upstream`, remain under [P's MIT license](tests/upstream/LICENSE.txt).
