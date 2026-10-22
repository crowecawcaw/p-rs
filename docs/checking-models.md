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
It prints one line per test case and, for a bug, its report and the `snappea replay`
command that reproduces it:

```
$ snappea check -n 300
Checking TwoPhaseCommit: 10 test cases
ok    tcSingleClientNoFailure (300 schedules, 8 workers, 0.06s)
FAIL  tcMultipleClientsWithFailure (bug found after 23 schedules, 8 workers, 0.02s)
      Deadlock detected. Client(6) is waiting to receive an event, but no other controlled tasks are enabled.
      Replay: snappea replay PCheckerOutput/tcMultipleClientsWithFailure/bug.schedule
...
5 of 10 test cases failed in 0.98s, peak memory 18 MB. Output: PCheckerOutput/
```

Each test case's files go to `PCheckerOutput/<test>/`, replacing the previous run's:
`bug.txt` (the buggy schedule's log), `bug.schedule`, `bug.trace.json`, `coverage.txt`,
`summary.txt` and, for scenario monitors, `scenario_coverage.json`. `-v` adds
upstream's progress and statistics. It exits with 0 when no bug is found, 1 when one
is, and 2 on any other error. `snappea <command> --help` lists the options.

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

For installation and a first model, see the [command line guide](https://snappea.dev/docs/cli.html).
For checking application events and logs, see [snappea observe](observe.md).
