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

`snappea check` type-checks the sources and selects search strategies automatically.
Each worker retains an exhaustive frontier, fresh random exploration, a guided
corpus, and a priority scheduler (PCT). A short probe measured in controlled choices
lets small exhaustive searches finish immediately. After the probe, work credits
reserve at least 30% for random, 30% for guided, 10% for PCT, and 10% for exhaustive
search while all four arms remain enabled. Recent new event-order patterns,
monitor/scenario and activity coverage, distinct failure sites, and state-cache
convergence determine the remaining 20%. Allocation measures scheduling and data
choices rather than schedule counts or elapsed time. Each arm keeps its search
state when another arm runs.

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
`bug.txt`, `bug.schedule`, `bug.trace.json`, `coverage.txt`, `summary.txt` and, for
scenario monitors, `scenario_coverage.json`. A failure includes its report and a
`snappea replay` command. `-o DIR` selects another output folder.

The native CLI no longer accepts schedule counts, strategy selection, exhaustive
mode selection, step bounds, or feedback-tuning options. `snappea-compat` retains
upstream's CLI, defaults, step bounds, and seeded streams, including the original
feedback mutation behavior. Existing compatibility traces still replay; the native
`replay` command accepts step options needed by older traces.

Time tracking (`--latency <file>[:<profile>]`, `--timed`) checks timeouts and latency
budgets; see [design/time](../design/time/RetryBudget/README.md).
For checking application events and logs, see [snappea observe](observe.md).
