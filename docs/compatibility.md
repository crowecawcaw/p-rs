# Compatibility with upstream P

This reference is about `snappea-compat`. `snappea` checks the same way, finds the same bugs
and writes the same trace formats, but does not imitate upstream where it doesn't have
to:

* Options: its own, shorter ones (`snappea <command> --help`).
* Console: one line per test case, the bug's report and its `snappea replay` command, and a
  summary with time and peak memory. Upstream's progress only with `-v`.
* Output folder: `PCheckerOutput/<test>/` with fixed names (`bug.txt`, `bug.schedule`,
  `bug.trace.json`, `scenario_coverage.json`), replaced by each run. No `BugFinding/`
  folder, history folders, `.sci`, coverage or summary file: coverage and the summary go
  to the console (or the JSON result).
* `<IncludeProject>` paths in a `.pproj` are resolved against the including project
  file's folder first, then the current folder (upstream: only the current folder).
* `observe`: sources and logs are positional arguments, live input stops at the first
  error without `--fail-fast`, and there is no report folder (use `--json-report`). Exit
  codes are 0, 1 and 2, as for `check`; see [snappea observe](observe.md).

**Exact** (drop-in):

* `check`, `merge-scenario-coverage` and `observe` options, help, console output and
  exit codes.
* Parse and type error messages.
* Bug reports, assertion and liveness messages.
* Trace files (`.txt`, `.trace.json`, `.schedule`), coverage reports, output folder
  layout and history rotation.
* `.pproj` files.
* PObserve output files (replay logs, `ParserError_N.txt`, `UnknownError_N.txt`,
  `PObserveMetrics.txt`).

**Functional** (same result, details differ):

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
  `--exclude-specs`, `--json-report`, `--strict`, `--unknown-events`,
  `--check-hot`, several `-l` values and glob patterns, and live input (`--stdin`,
  `--live`, `--sequence`, `--record`). Runs that use none of them are unchanged.
* Foreign functions and log parsers are written in Rust, not C# or Java.

**Not supported yet**:

* PVerifier mode.
* `observe` event filters (`--filters`), socket mode and `SERIALIZEDEVENTS` input.
* `--memout` is accepted and ignored.
* `--graph` and `--graph-bug` are accepted but, as upstream, write nothing.

**Out of scope**:

* `p compile` and its output (`PGenerated/`, `.dll`, jars). There is no `compile`
  command; `OutputDir` and `Target` in `.pproj` files are ignored.
* .NET and Java stack frames in exception reports, traces and `UnknownError_N.txt`.
* Foreign code in C# or Java, and PObserve jars (`--jars`).
