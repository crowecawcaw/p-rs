# Compatibility

This page is about `prs-compat`, the drop-in replacement for the upstream `p` tool.
`prs` checks the same way and writes the same files, with its own options.

## Exact (drop-in)

* `check`, `merge-scenario-coverage` and `observe` options, help, console output and
  exit codes.
* Parse and type error messages.
* Bug reports, assertion and liveness messages.
* Trace files (`.txt`, `.trace.json`, `.schedule`), coverage reports, output folder
  layout and history rotation.
* `.pproj` files.
* PObserve output files (replay logs, `ParserError_N.txt`, `UnknownError_N.txt`,
  `PObserveMetrics.txt`).

## Functional (same result, details differ)

* Findings may be reported in a different order.
* `check` reads P sources, not a compiled `.dll`: a `.pproj` or `.p` file as its path,
  `--pproj`, `--pfiles` and `--projname` (upstream: options of `p compile`), or else the
  `.pproj` or `.p` files in the current folder.
* Source discovery is logged differently, since there is no compiled output to find.
* Files are enumerated in sorted order; elapsed time and memory figures differ.
* An action that runs forever without reaching a scheduling point is reported as a bug
  (upstream hangs).
* Nesting deeper than 128 levels is a parse error, `nesting is too deep` (upstream has
  no limit). Blocks, statements, expressions, types and each operand of an operator
  chain such as `a + b + c` count a level; the upstream tests nest at most 16 deep.
* `merge-scenario-coverage --help` prints its usage (upstream looks for a directory
  named `--help`).
* `observe` reports a null dereference with its own message.
* `observe` extensions: several names after `--spec`, `--all-specs`,
  `--exclude-specs`, `--json-report`, `--coverage`, `--strict`, `--unknown-events`,
  `--check-hot`, several `-l` values and glob patterns, and live input (`--stdin`,
  `--live`, `--sequence`, `--record`). Runs that use none of them are unchanged.
* Foreign functions and log parsers are written in Rust, not C# or Java.

## Not supported yet

* PVerifier mode.
* `observe` event filters (`--filters`), socket mode and `SERIALIZEDEVENTS` input.
* `--memout` is accepted and ignored.
* `--graph` and `--graph-bug` are accepted but, as upstream, write nothing.

## Out of scope

* `p compile` and its output (`PGenerated/`, `.dll`, jars). There is no `compile`
  command; `OutputDir` and `Target` in `.pproj` files are ignored.
* .NET and Java stack frames in exception reports, traces and `UnknownError_N.txt`.
* Foreign code in C# or Java, and PObserve jars (`--jars`).

## How parity is measured

prs passes upstream's compiler regression tests (424), checker suites (249) and
tutorials, and reproduces upstream's golden traces and PObserve outputs. See
[development.md](development.md#validation) for how this is tested.
