# Upstream P test corpus

This directory vendors the regression tests and tutorial models of the
[p-org/P](https://github.com/p-org/P) repository (MIT license, see `LICENSE.txt`),
taken from commit `260da928f67569ca7a8c0039a02d0bad7cde6096`.

The upstream harness interprets the directory layout as follows:

* `*/StaticError/<test>/` – compilation must fail (exit code 1).
* `*/Correct/<test>/` – compilation succeeds and checking 1000 schedules with at
  most 1000 steps each finds no bug (exit code 0).
* `*/DynamicError/<test>/` – checking finds a bug (exit code 1).
* Directories whose name contains `param` carry an `ExpectedParametricTests.txt`
  listing the test case names that must be reported by `--list-tests`.

Upstream only runs `Combined`, `Feature1SMLevelDecls`, `Feature2Stmts`,
`Feature3Exprs`, `Feature4DataTypes` and `Integration`; `Feature5ModuleSystem` and
`Liveness` are kept for completeness.

`CorrectLogs/bugs2/` holds upstream's golden trace files for
`Feature1SMLevelDecls/DynamicError/bug2` (`PCheckerLogGeneratorTests`): the generated
`bug2_0_0.txt` must contain the expected text and the generated `.trace.json` must
contain the expected JSON as a subset.

`PObserve/` holds the PObserve regression suite of
`Src/PObserve/PObserveRegressionTesting` (spec, run parameters, and every log under
8 KB) with the outputs of upstream PObserve on each log, with and without
`--assumeInputsAreSorted` (`expected/`, recorded by `scripts/vendor-pobserve-goldens.py`
and checked in `crates/prs-compat/tests/observe_upstream.rs`).
