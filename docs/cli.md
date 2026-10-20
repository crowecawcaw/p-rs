# Command line

Run `prs` in a folder with a `.pproj` file:

```sh
prs tests                                   # list the test cases
prs check                                   # check every test case, 10000 schedules each
prs check tcMyTest -n 1000                  # one test case, 1000 schedules
prs check 'tcMy*' --time 5m                 # test cases matching a glob, for 5 minutes each
prs check model.p tcMyTest                  # .p files instead of a .pproj
prs check tcMyTest --strategy pct:10        # another search strategy
prs check tcMyTest --exhaustive             # every scheduling and data choice
prs replay PCheckerOutput/tcMyTest/BugFinding/MyProject_0_0.schedule
```

`prs check` type-checks the sources itself and reports errors as `p compile` does.
Bugs are written to `PCheckerOutput/` as upstream writes them. It exits with 0 when no
bug is found, 1 when one is, and 2 on any other error. `prs <command> --help` lists the
options.

## Parallel workers

`prs check` explores schedules on one worker thread per CPU core (`-j N` sets the
number). The schedule count is split between the workers, each with its own seed; the
first bug stops them all, and its `.schedule` file replays it as usual. The `random`,
`pct`, `fairpct`, `prob` and `pos` strategies use workers; the feedback strategies and
`rl` (which learn from earlier schedules), `--exhaustive` and `--verbose` run one
worker. With more than one worker, a given `--seed` explores different schedules than
upstream, and which bug is reported can vary from run to run; `-j 1` explores exactly
upstream's schedules.

## Exhaustive search

`--exhaustive` explores every scheduling choice and every `$` / `choose` value by
depth-first search, skipping states it has already seen. Its report says whether the
search finished or hit the `--max-steps` bound. Time tracking is not supported in this
mode yet.

## Time tracking

Time tracking (`--latency <file>[:<profile>]`, `--timed`) checks timeouts and latency
budgets; see [design/time](../design/time/RetryBudget/README.md).

## Checking logs

`prs observe` replaces [PObserve](https://github.com/p-org/P/tree/master/Src/PObserve):
it checks logs from a running system against a project's spec monitors, without jars.

```sh
prs observe --spec BankBalanceCorrect -l logs/
```

See [observe.md](observe.md) for log formats and custom parsers, and
[production.md](production.md) for how the same `emit` calls feed both test checks and
`prs observe` on production logs.

## Upstream compatible CLI

To use prs in place of upstream P, in scripts written for `p`, run `prs-compat` (or
alias it to `p`):

```sh
prs-compat check -tc tcMyTest -s 1000
```

See [compatibility.md](compatibility.md) for how it differs from upstream.
