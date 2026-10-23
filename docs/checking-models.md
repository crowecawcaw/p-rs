# Checking P models

Run `snappea` in a folder with a `.pproj` file:

```sh
snappea tests                                   # list the test cases
snappea check                                   # check every test case, 10000 schedules each
snappea check tcMyTest -n 1000                  # one test case, 1000 schedules
snappea check 'tcMy*' --time 5m                 # test cases matching a glob, for 5 minutes each
snappea check model.p tcMyTest                  # .p files instead of a .pproj
snappea check tcMyTest --strategy pct:10        # another search strategy
snappea check tcMyTest --exhaustive             # every scheduling and data choice
snappea replay PCheckerOutput/tcMyTest/bug.schedule
```

`snappea check` type-checks the sources itself and reports errors as `p compile` does.
It prints one line per test case with its coverage and, for a bug, its report and the
`snappea replay` command that reproduces it. Then it lists the code no test case ran:

```
$ snappea check -n 1000
Checking Inventory: 2 test cases
ok    tcAtomicReserve (1000 schedules, 4 workers, 0.07s)
      Coverage: 41/49 statements, 6/10 branches, 6/9 event handlers
FAIL  tcCheckThenReserve (bug found after 4 schedules, 4 workers, 0.00s)
      Assertion Failed: PSpec/inventory.p:27:7 reservation exceeds available stock
      Replay: snappea replay PCheckerOutput/tcCheckThenReserve/bug.schedule
      Coverage: 43/49 statements, 5/10 branches, 8/9 event handlers
Coverage of all test cases: 49/50 statements, 8/10 branches, 9/9 event handlers
Not covered by any test case:
  PSrc/Inventory.p:46  Inventory.Serving, on eReleaseReq: if (req.order in reservations) never false
  PSrc/Inventory.p:85  OrderClient.Checking, on eAvailableResp: if (resp.units < units) never true
1 of 2 test cases failed in 0.08s, peak memory 10 MB. Output: PCheckerOutput/
```

A test case that finds a bug writes its files to `PCheckerOutput/<test>/`, replacing
the previous run's: `bug.txt` (the buggy schedule's log), `bug.schedule` and
`bug.trace.json`. Scenario monitors also write `scenario_coverage.json` there. `-v` adds
upstream's progress and statistics. It exits with 0 when no bug is found, 1 when one
is, and 2 on any other error. `snappea <command> --help` lists the options.

On a terminal or in CI (`CI` set) the output is the text above; otherwise (agents,
scripts) it is JSON: per test case its result, schedules, bug report and `replay`
command, and coverage; then the coverage of all test cases with every uncovered entry.
`--format text|json` or `--json` chooses, as does `SNAPPEA_FORMAT=text|json`, and
`--json-report FILE` also writes the JSON to `FILE`. Every `snappea` command takes
these options.

`snappea check` explores schedules on one worker thread per CPU core (`-j N` sets the
number). The schedule count is split between the workers, each with its own seed; the
first bug stops them all, and its `bug.schedule` file replays it as usual. The `random`,
`pct`, `fairpct`, `prob` and `pos` strategies use workers; the feedback strategies and
`rl` (which learn from earlier schedules), `--exhaustive` and `--verbose` run one
worker. With more than one worker, a given `--seed` explores different schedules than
upstream, and which bug is reported can vary from run to run; `-j 1` explores exactly
upstream's schedules.

`--exhaustive` explores every scheduling choice and every `$` / `choose` value by
depth-first search, skipping states it has already seen. Its report says whether the
search finished or hit the `--max-steps` bound. Time tracking is not supported in this
mode yet.

Time tracking (`--latency <file>[:<profile>]`, `--timed`) checks timeouts and latency
budgets; see [design/time](../design/time/RetryBudget/README.md).

## Coverage

Coverage counts what the explored schedules ran, in the machines and monitors each test
case includes: statements, both outcomes of each `if`, and the `do` and `goto` event
handlers of each state. Code that starts with `assert false` must never run, so it is
not counted. A bug stops a test case early, so its coverage is partial.

The list after the test cases names, for each piece of code that never ran, the
outermost one: a handler that was never triggered rather than each of its statements.
It shows the first 20 entries; the JSON result has them all.

Code that never runs is code the model was not checked on. Usually a test case is
missing (no test sends the event, or none reaches the state), or a test driver never
chooses the value that takes the branch; sometimes the code cannot run, which is worth
knowing too. Uncovered code in a spec monitor means the property it checks was never
exercised.

For installation and a first model, see the [command line guide](https://snappea.dev/docs/cli.html).
For checking application events and logs, see [snappea observe](observe.md).
