# Time, timeouts and latency budgets for P (design proposal, v2)

Status: draft for discussion. Nothing here is implemented. v2 incorporates an
independent review of v1 (summary of changes at the end).

## Motivation and scope

P checks the *logic* of distributed protocols but has no time: timers fire on `$`
and the scheduler ignores how long anything takes. The bugs worth catching are
those where **time interacts with control flow**:

* retry policies that outlast the caller's deadline,
* timeout layering that orphans work or causes retry storms,
* deadlines that are not propagated,
* failover paths that exceed a budget,
* hedging and quorum patterns whose benefit depends on which reply comes first,
* false bugs in today's models where a timeout fires before a reply that would
  certainly have arrived.

Summing latencies along a fixed path (examples 1-3) is supported but is not the
motivation: a spreadsheet does that. Percentiles are out of scope: ranges give
per-path bounds, not distributions. Additive non-time costs (RCU, rows, call
counts) need no feature: a spec monitor sums them today (example 10).

## Two surfaces, one semantics

**Recommended first: the upstream-compatible surface** (`upstream-friendly/`).

* `Now()`, `NowBest()`, `Elapse(lo, hi)`, `Mark()`, `Elapsed(m)`, `ElapsedBest(m)`
  are foreign functions over `int` microseconds. Upstream links C# stubs where
  time stands still, so the same `.p` files run there untimed.
* Timers are ordinary machines (`Timer` in `upstream-friendly/RetryBudget`).
* Delivery latencies and named costs live in a profile file passed with
  `p check --latency <file>`.
* `--timed` is a checker flag.

**Possible later: native syntax** (`examples/`): `duration` values and
literals, `elapse`, `send ... latency d`, latency profiles in `.p` files. Same
semantics, nicer to write, but models no longer compile upstream. Build only if
the foreign-function surface proves the idea on real models.

## Semantics

### Clocks and stamps

Every machine has a local clock.

```
send e (latency L):   stamp(e)  = clock(sender) + L
dequeue e:            clock(rcv) = max(clock(rcv), stamp(e))
Elapse / elapse d:    clock(self) = clock(self) + d
new M(...):           clock(child) = clock(creator)
announce e:           observed by specs at clock(announcer); no latency
raise e, goto:        no cost
ignored event:        dropped, no clock effect
deferred event:       clock effect when finally dequeued
receive:              like dequeue
```

Specs read `Now()` (or `now()`) as the clock of the machine whose send or
announce is being observed, plus `arrival()` = send stamp for sends, = `now()` for
announces. In a spec's start entry and in spec functions called from it the
clock is 0. Specs may not contain uncertain comparisons (see below): P forbids
nondeterminism in monitors.

### Delivery is a scheduler step

v1 enqueued a delayed send immediately. P mailboxes are FIFO, so a 300ms timeout
sent to self always sat ahead of a reply that came later: the reply-before-timeout
interleaving was never explored, which is *fewer* behaviours than today's
`Timer` machine. Fix:

* A send with nonzero latency puts the event **in flight**. Moving it into the
  receiver's mailbox is a separate, schedulable operation.
* Per (sender, receiver) link, deliveries stay FIFO, as in P.
* Self-sends with latency (timers) are exempt from link FIFO.
* Zero-latency sends behave exactly as today.

In the upstream-compatible surface this falls out for free: a `Timer` is a
separate machine, so the scheduler already chooses when its timeout lands.

Consequence: delayed sends **add** interleavings (as many as today's
nondeterministic timers). "Zero cost when unused" holds; "never changes the
explored schedules" holds only for programs whose sends all have latency 0.

### Values: ranges with correlation

A duration is a range `[best, worst]`, but plain interval arithmetic is wrong for
budgets: in `now() - t0`, both share their history, and interval subtraction
adds their widths instead of cancelling them (v1's examples 1 and 2 failed
falsely this way: a 22ms request reported as up to 38ms, and 62ms).

Representation: **affine forms**. Each evaluation of a range (an `Elapse(lo, hi)`
call, a latency from the profile, a range literal) introduces a fresh noise
symbol `e_i ∈ [0, 1]`; a duration is `c + Σ a_i e_i`. Then:

* `+`, `-`, scaling by a constant: exact; shared terms cancel.
* `max(a, b)`: exact when `a - b` has a certain sign (the common case
  `max(local, local + delay)`), otherwise over-approximated by a fresh symbol.
* `best`/`worst`: minimise/maximise over the symbols.
* A value stored in a variable keeps its symbols: reusing it means *the same*
  value (example 4 uses one stored link latency for both directions on purpose).
  A range literal is fresh at each evaluation.
* Values may be negative (a difference of correlated quantities).
* Symbols are bounded by the steps of one execution; the implementation may
  collapse old symbols into a constant range when no live value references them.

Upstream-compatible surface: `Mark()` returns a handle to the current clock;
`Elapsed(m)` / `ElapsedBest(m)` return the worst / best of `now - mark` as
`int`, computed exactly as above. `Now()` / `NowBest()` return the worst / best
of the clock itself (for deadlines carried in payloads).

### Comparisons (native surface)

A comparison of durations is *certain* or *uncertain*. One rule everywhere: an
uncertain comparison is a recorded nondeterministic choice (like `$`), in
`if`, `while`, `assert`, assignments and function arguments alike. So
`assert now() - t0 <= 100ms` fails on some schedule whenever the bound *could* be
exceeded (the worst-case reading), and `assert !(now() > d)` behaves the same way,
without three-valued logic. `==` on durations is structural (same form).

After a branch on `now() <op> d`, `now()` is narrowed in each branch (it is a
path condition, as a guard in a timed automaton). Explicit modalities:
`best(x)`, `worst(x)`, `certainly(cmp)`, `possibly(cmp)`. Specs must use those.

The upstream-compatible surface has no uncertainty: functions return `int`
bounds and the model chooses which one it compares.

### What the bounds mean (and do not)

Clock values over-approximate: FIFO and head-of-line ordering can only delay a
machine, and range bounds are the extremes. So **monotone** budget assertions
(`elapsed <= c`, where `elapsed` grows with latency) are sound for the explored
schedules: if the checker finds no violation, none of those schedules can exceed
the budget. Non-monotone checks (`best(now()) <= deadline`, `finishedAt <
gaveUpAt`) can be hidden by that over-approximation; they still find bugs, but
"no bug found" is weaker. As always in P, schedules are sampled, so "explored"
does not mean "all".

## `--timed` scheduling

Untimed, the scheduler may deliver events in an order that is impossible in time.
That over-approximates but gives false positives in timeout-heavy models (and
makes hedging unverifiable). With `--timed`:

* The candidates are **dequeuable** steps: in-flight deliveries, and machines
  that can run (their next event is neither deferred nor blocked in a `receive`
  for another event; halted machines and empty mailboxes do not count). A machine
  that can run is ranked by its own clock; a delivery by its stamp.
* A candidate `x` may be scheduled unless some other candidate `y` has
  `worst(y) < best(x)` (strictly earlier). Ties and overlaps stay
  nondeterministic.
* `Elapse` is a scheduling point in `--timed` mode, so a machine whose clock
  jumps yields to earlier work.
* **Narrowing**: when `x` is scheduled ahead of a candidate `y`, record
  `x <= y` (it happened no later). With affine forms this is a constraint on the
  symbols; in a simpler implementation, clamp `worst(x)` to `worst(y)` and
  raise `best(y)` to `best(x)`. This is sound because, in `--timed` mode,
  schedule order of candidates is arrival order (delivery is a step; see above).
  It is what makes "first reply wins" (example 6) checkable.

This turns P into a bounded discrete-event simulator. It changes exploration, so
it is opt-in.

## Profiles

Code says *where* a cost occurs; the profile says *how much*.

* Delivery latency per event, per link `(SenderType, ReceiverType)`, or per link
  and event.
* Named parameters: `Elapse(Cost("DbQuery"))` or, natively,
  `elapse cost.DbQuery; send s, e, p latency link.ToReplica;`.
* Precedence, most specific first: per-send literal > `(A, B).e` > `(A, B)` >
  profile `e` > event declaration > profile `default` > `0`.
* `default` never applies to self-sends (timers), Main's setup sends, or
  announces.
* Timeouts are protocol constants and belong in code; network numbers belong in
  the profile.
* A profile is attached per test case (`[tests]` table, or natively
  `test t [main=M, latency=P]`) and can be overridden with `--latency-profile`.

## Representation and replay

* Integer microseconds throughout; native literals take a mandatory unit
  (`us`, `ms`, `s`) and are integral (`500us`, not `0.5ms`). Overflow is a
  checker error.
* The schedule file records `--timed`, the profile name and a hash of the
  profile. Replay refuses a mismatch, since changing a number can turn a certain
  comparison uncertain (or vice versa) and shift every later choice.
* Trace lines get an `@ [best, worst]` suffix only when the program uses time,
  so traces of other programs stay byte-identical to upstream's.

## Checker output

* Failed budget assertion: the trace ends with the critical path (the chain of
  sends, deliveries and `Elapse` calls behind the `worst` bound).
* `--latency-report`: per assertion site, the maximum `worst` seen across the
  explored schedules and the schedule that produced it, even without a bug.

## Validation before building more

1. Implement clocks, `Mark`/`Elapsed`, the profile file and `--timed` behind the
   foreign-function surface (no syntax change).
2. Port the Raft and FailureDetector tutorials and one retry/timeout model.
3. Keep it if it finds bugs, or removes false positives, that untimed P cannot.
   Otherwise stop.

## Open questions

1. Is affine-form narrowing (constraints between symbols) worth it, or is
   clamping enough in practice?
2. Should time budgets also turn P's liveness monitors into bounded-response
   safety checks (`hot` state must be left within d)?
3. A cancellable timer construct would remove the generation-counter boilerplate
   in examples 5, 8 and 9. Library machine or built-in?

## Examples

`upstream-friendly/RetryBudget/` is the recommended shape and compiles with
today's `p compile`. `examples/` uses the native syntax:

| # | File | Shows |
|---|---|---|
| 1 | `01_client_server_sla.p` | event latency, `elapse`, budget in the client |
| 2 | `02_cache_profiles.p` | hit/miss paths, profiles with named costs, two test cases |
| 3 | `03_scatter_gather.p` | fan-in is a max; head-of-line merge cost |
| 4 | `04_quorum_read.p` | 2-of-3 quorum; untimed vs `--timed` |
| 5 | `05_retry_backoff.p` | timeouts and backoff as delayed self-sends |
| 6 | `06_hedged_request.p` | hedging; needs `--timed` with narrowing |
| 7 | `07_deadline_propagation.p` | deadlines in payloads, branching on time, `best`/`worst` |
| 8 | `08_timeout_ordering.p` | client / LB / server timeout layering |
| 9 | `09_failover.p` | crash, detection, takeover, redirect within budget |
| 10 | `10_capacity_costs.p` | RCU and amplification with today's monitors, `possibly` |

## Changes from v1 (review findings)

* Delayed sends are in flight; delivery is a scheduler step (v1 never explored
  reply-before-timeout).
* Affine forms instead of plain intervals (v1 gave false budget failures).
* One rule for uncertain comparisons (choice everywhere), branch narrowing,
  explicit modalities, none in specs.
* `--timed` defined over dequeuable candidates; `Elapse` is a scheduling point;
  narrowing justified by delivery-as-step.
* Soundness stated for monotone assertions only.
* `new`, `announce`, `ignore`, `defer`, `receive`, `raise` specified.
* Profile precedence fixed; per-state handler costs replaced by named costs;
  `default` excludes timers.
* Integer microseconds, unit-suffixed integral literals, replay metadata,
  byte-identical traces for untimed programs.
* Examples fixed: 2 (named costs), 4 (no clash with `best`, ignore has no clock
  effect), 7 (numbers, interval-safe check), 8 (explicit modality in the spec),
  9 (rewritten: unhandled event, crash ordering, takeover model), 10 (`possibly`
  in the spec), literals in 2, 3.
