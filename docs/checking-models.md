# Checking P models

Run `snappea` in a folder with a `.pproj` file:

```sh
snappea tests                                   # list the test cases
snappea check                                   # check every test, up to 10 seconds each
snappea check tcMyTest                          # one test case
snappea check 'tcMy*' --timeout 5m               # matching tests, up to 5 minutes each
snappea check model.p tcMyTest                  # explicit sources
snappea replay PCheckerOutput/tcMyTest/bug.schedule
```

`snappea check` type-checks and [lints](linting.md) the sources before exploring
schedules. Unsuppressed lint findings stop the check with exit status 2; run
`snappea lint` to inspect them without exploration. It selects search strategies automatically.
Each worker retains an exhaustive frontier, fresh random exploration, a guided
corpus, and a priority scheduler (PCT). A short probe measured in controlled choices
lets small exhaustive searches finish immediately. After the probe, work credits
reserve at least 30% for random, 30% for guided, 10% for PCT, and 10% for exhaustive
search while all four arms remain enabled. Recent new event-order patterns,
monitor/scenario, statement, branch and activity coverage, distinct failure sites, and state-cache
convergence determine the remaining 20%. Allocation measures scheduling and data
choices rather than schedule counts or elapsed time. Each arm keeps its search
state when another arm runs.

The guided corpus keeps schedules that add code coverage, short-circuit condition
outcomes, event-order features, scenario progress, or a distinct failure site.
It mutates productive seeds immediately, mixing small input changes with resampled
continuations. Seeds of distinct failure sites remain available when searching for
other bugs. Stalled guidance increases fresh sampling while retaining a share for
mutations. The corpus and event-feature history are bounded.

A completed exhaustive search ends the run early. State and choice-expansion caps
keep exhaustive exploration bounded; the remaining arms continue when a cap is
reached. Timed models use the sampled arms because exhaustive time tracking is not
supported. Hot cycles continue through fair sampling: closing a repeated state is
not a liveness proof. A coverage plateau changes allocation, never the stopping rule.

The default is a fixed **10-second limit per test**, with no automatic extensions.
`--timeout` overrides it; the previous spelling `--time` remains an alias. A timeout
means the search is incomplete, even if it found no bugs. Exit status is 0 when no
bug was found, 1 when a bug was found, and 2 on an error. The result distinguishes
exhaustive completion from an incomplete search.

Workers run in parallel, one per available CPU core by default; `-j N` limits resource
use. The first bug or a valid exhaustive completion stops the other workers. `--seed`
sets their starting seeds, but worker interleaving can change which bug wins. Bug
schedules record the choices and replay independently of the allocating policy.
`-v` shows progress and statistics and uses one worker.

Each test case's files go to `PCheckerOutput/<test>/`, replacing the previous run's:
`bug.txt`, `bug.schedule`, `bug.trace.json` and, for
scenario monitors, `scenario_coverage.json`. A failure includes its report and a
`snappea replay` command. `-o DIR` selects another output folder.

The native CLI no longer accepts schedule counts, strategy selection, exhaustive
mode selection, step bounds, or feedback-tuning options. `snappea-compat` retains
upstream's CLI, defaults, step bounds, and seeded streams, including the original
feedback mutation behavior. Existing compatibility traces still replay; the native
`replay` command accepts step options needed by older traces.

Time tracking (`--latency <file>[:<profile>]`, `--timed`) checks timeouts and latency
budgets; see [design/time](../design/time/RetryBudget/README.md).

`snappea check` type-checks the sources itself and reports errors as `p compile` does.
It prints one line per test case with its coverage and, for a bug, its report and the
`snappea replay` command that reproduces it. Then it lists the code no test case ran:

```
$ snappea check
Checking Inventory: 2 test cases
ok    tcAtomicReserve (1000 schedules, 4 workers, 0.07s)
      Coverage: 41/49 statements, 6/10 branches, 6/9 event handlers
FAIL  tcCheckThenReserve (bug found after 4 schedules, 4 workers, 0.00s)
      Assertion Failed: PSpec/inventory.p:27:7 reservation exceeds available stock
      Replay: snappea replay PCheckerOutput/tcCheckThenReserve/bug.schedule
      Coverage: 43/49 statements, 5/10 branches, 8/9 event handlers (partial: stopped at first bug)
Coverage of all test cases: 49/50 statements, 8/10 branches, 9/9 event handlers
Not covered by any test case:
  PSrc/Inventory.p:46  Inventory.Serving, on eReleaseReq: if (req.order in reservations) never false
  PSrc/Inventory.p:85  OrderClient.Checking, on eAvailableResp: if (resp.units < units) never true
Coverage includes partial results from test cases stopped at their first bug.
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
these options, before or after the command (for example, `snappea --json check`).
Argument errors, including unknown options and missing values, also produce a JSON
error result and write `--json-report` when requested. JSON keys retain insertion order.

## Coverage

Coverage counts what the explored schedules ran, in the machines and monitors each test
case includes: statements, both outcomes of each `if`, and the `do` and `goto` event
handlers of each state. Code that starts with `assert false` must never run, so it is
not counted. A bug stops a test case early, so its coverage is marked partial in
the console and with `coverage.partial: true` in JSON. The combined total includes
these results and is also marked partial when any test case finds a bug.

Scenario monitors appear in the console as a unified summary, including scenarios
that were never satisfied. JSON includes the scenario artifacts under `scenarios`
for the suite and for each test case with scenarios. The same artifacts are still
written to `scenario_coverage.json` for merging across runs.

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
