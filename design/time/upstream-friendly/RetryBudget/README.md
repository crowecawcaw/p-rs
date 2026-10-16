# RetryBudget: the upstream-friendly shape of the time extension

Sample 5 of `../../examples/` rewritten with no new syntax. The extension is:

| Piece | Where | Upstream P sees it? |
|---|---|---|
| `Now()`, `NowBest()`, `Elapse(lo, hi)` | foreign functions in `PSrc/Time.p` | yes, as foreign functions (C# stubs in `PForeign/`: time stands still) |
| One-shot `Timer` | plain P in `PSrc/Timer.p` | yes, behaves like today's nondeterministic timer |
| Delivery latencies | `PTst/latency.toml` | no |
| Time-ordered scheduling | `p check --timed` | no |

`p compile` already accepts this project today (verified with the current p-next
build); only the checker behaviour below is new.

## Intended runs (hypothetical output)

```
$ p check -tc tcRetry -s 1000                       # upstream, or p-next without --latency
... Found 0 bugs.                                    # Now() is 0: budgets trivially hold

$ p check -tc tcRetry -s 1000 --latency PTst/latency.toml
<ErrorLog> Assertion Failed: resolved (ok = false) after up to 1050000us; deadline is 1000000us
  critical path: eReq@750ms -> drop -> Timer(300ms) -> eTimeout@1050ms -> eOutcome
... Found 1 bug.

$ p check -tc tcRetry -s 1000 --latency PTst/latency.toml --timed --latency-report
<ErrorLog> Assertion Failed: ... after up to 1050000us ...       # same bug: drops are real
latency report:
  PSpec/Budget.p:6  max 1050000us (schedule 17)  ok-path max 792000us (schedule 3)
```

Fix: `timeoutAfter = ms(250)` makes the give-up path 900ms and the check passes.

## What `--timed` changes here

Untimed, a Timer's `eTimeout` (stamp +300ms) can be handled before an `eResp`
stamped +40ms, so the checker also explores retries after successful replies.
They do not change the worst-case bound here, but a spec counting attempts per
request would report impossible retry counts. With `--timed` those orders are
pruned.

## Trade-offs vs. the syntax in PROPOSAL.md

* Times are `int` microseconds; `Now()` is the worst case and `NowBest()` the
  best case. No uncertain comparisons: `if (Now() > deadline)` is
  pessimistic (fails fast whenever the deadline might have passed).
* Per-send latency overrides become Timer machines or profile entries.
* `Now() - started` is a difference of worst cases. That is the right reading
  when both readings share the same history, and avoids the interval-arithmetic
  dependency problem of `[a, b] - [c, d]`.
