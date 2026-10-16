# Time, timeouts and latency budgets in p-next (design proposal, v3)

Status: under review; Phase 1 below is what gets implemented first.

v3 makes the decision of v2 explicit: **no language change**. Models use plain P
(foreign functions and ordinary machines), so they compile and run upstream,
where time stands still. Everything new is in `pnext-checker` and the CLI.
The native syntax explored in v1/v2 (`examples/`) is kept as a possible later
surface over the same semantics.

## Motivation and scope

P checks protocol logic but has no time: timers fire on `$` and the scheduler
ignores how long anything takes. The bugs worth catching are where **time
interacts with control flow**: retry policies that outlast the caller's
deadline, timeout layering that orphans work, unpropagated deadlines, failover
paths over budget, and false bugs where a timeout fires before a reply that would
certainly have arrived first.

Not in scope: percentiles or distributions (ranges give per-path bounds), clock
skew, and additive non-time costs (RCU, rows, call counts), which a spec monitor
already sums today.

## Surface (plain P)

A model opts in by declaring any of these foreign functions (exact names and
signatures; times are `int` microseconds):

```p
fun Now(): int;                // worst case of this machine's clock
fun NowBest(): int;            // best case
fun Elapse(lo: int, hi: int);  // local work taking lo..hi us (lo <= hi, lo >= 0)
fun ElapseCost(name: string);  // local work priced by the latency profile
fun Mark(): int;               // stopwatch handle for the current clock
fun Elapsed(m: int): int;      // worst case of (clock - mark m)
fun ElapsedBest(m: int): int;  // best case
```

p-next implements them natively. A user's Rust `ForeignFunctions` is not
consulted for these names. Upstream links C# stubs that return 0 and do nothing
(`upstream-friendly/RetryBudget/PForeign/Time.cs`).

Timers are ordinary machines (`upstream-friendly/RetryBudget/PSrc/Timer.p`): a
fresh machine per armed timer calls `Elapse(d, d)` and then sends the timeout.
Because the timer is a separate machine, the scheduler already decides whether a
reply or the timeout is handled first; no special delivery semantics are needed.

## Semantics

### Clocks

Time tracking is active for a test when the program declares any function above
or `--latency` is given; otherwise nothing is computed and nothing changes.

| Operation | Effect |
|---|---|
| machine created by `new` | child clock = creator clock (Main starts at 0) |
| `Elapse(lo, hi)` | clock += a fresh range `lo..hi` |
| `ElapseCost(n)` | clock += a fresh draw of the profile's cost `n` (0 if none) |
| `send` | event stamp = sender clock + delivery latency (fresh draw, see Profiles) |
| dequeue, or event delivered to a waiting `receive` | clock = max(clock, stamp) |
| ignored event dropped | no effect |
| `raise`, `goto`, `announce` | no cost; raised events carry the current clock |
| monitors | read the clock of the machine whose send/announce they observe |

Clocks only grow. A spec's start entry runs with clock 0.

### Values: affine forms

A plain interval is wrong for budgets: in `Elapsed(m)` the mark and the clock
share history, and interval subtraction would add their widths instead of
cancelling them. Each time value is therefore an affine form

    v = c + Σ a_i·ε_i,   ε_i ∈ [0, 1], a_i ≥ 0, symbols fresh per draw

* A range `lo..hi` drawn once is `lo + (hi - lo)·ε_new`.
* `+` and `-` are exact; shared symbols cancel.
* `best(v) = c + Σ min(0, coef)`, `worst(v) = c + Σ max(0, coef)` (differences
  can have negative coefficients).
* `max(a, b)`: if `best(a - b) ≥ 0` it is `a`; if `worst(a - b) ≤ 0` it is `b`;
  otherwise a fresh form `[max(best a, best b), max(worst a, worst b)]`
  (correlation is lost, the range is exact).
* To bound cost, a form with more than 32 symbols collapses its smallest-range
  symbols into one fresh symbol. That keeps its own range but loses correlation
  with other values, which only widens later differences.

Arithmetic is exact `i64` microseconds; overflow is a checker error.

### Delivery order

Delivery follows P: an event enters the receiver's queue when it is sent, so
queue order is send order. Latency changes stamps, not order, in the default
mode. Consequence (documented limitation): a message sent earlier with a long
latency is handled before one sent later with a short latency from another
sender, unless `--timed` is on (below) or the network is modelled as a machine.

### `--timed` scheduling

Untimed, the scheduler may run steps in an order impossible in time (a timer's
300 ms send before a reply's 20 ms send). With `--timed`, before each scheduling
decision, every enabled operation gets a **ready time**:

* test task: 0;
* a machine about to dequeue: `max(clock, stamp)` of the event it would dequeue
  (or its clock when it would take a default handler);
* a machine whose blocked `receive` just got its event: `max(clock, stamp)`;
* any other machine (paused at a send/create/receive point mid-handler, or
  newly created): its clock.

Operation `x` is removed from the candidates if some other enabled `y` has
`worst(ready y) < best(ready x)`. Ties and overlaps stay nondeterministic. The
earliest operation is never removed, so this cannot create deadlocks. `Elapse`
is not a scheduling point; its effect counts at the machine's next one (the
only observer in between would be a spec via `announce`, which reads the clock
anyway).

Also with `--timed`, `send` inserts the event ahead of queued events **from
other senders** whose stamp is certainly later (`best(queued) > worst(new)`).
Per-sender FIFO is kept. Overlapping stamps keep send order.

Deferred to Phase 2: narrowing (when `x` is chosen ahead of an overlapping
`y`, record `x ≤ y`). Without it, "first reply wins" patterns (hedging) get
pessimistic bounds.

### What results mean

Clock values over-approximate: queueing only delays, ranges are taken at their
extremes, and collapses widen. So a **monotone** budget assertion such as
`assert Elapsed(m) <= budget` that fails on an explored schedule is a real
schedule exceeding the budget *under the stated latencies*, and one that never
fails holds for every explored schedule. Checks in the other direction (e.g.
`ElapsedBest(m) > x`) can be masked. As always, schedules are sampled.

## Profiles

A small TOML subset, passed with `p check --latency <file>`:

```toml
# comments, [sections], key = "string" only

[profile.same-az]
default        = "500us..2ms"   # any send without a more specific entry
eReq           = "1ms..2ms"     # by event name
"(Client, LB)" = "1ms..5ms"     # by (sender machine type, receiver machine type)
"(LB, Db).eGet" = "2ms..4ms"    # by link and event
"cost.DbQuery" = "2ms..8ms"     # priced by ElapseCost("DbQuery")

[profile.cross-region]
extends        = "same-az"
"(LB, Db)"     = "60ms..90ms"

[tests]
tcRetry = "same-az"             # profile per test case
```

* Values: `"lo..hi"` or a single value, each an integer with unit `us`, `ms`
  or `s`.
* Precedence, most specific first: `(A, B).e`, `(A, B)`, `e`, `default`, 0.
  `extends` copies the parent's entries first.
* `default` and the link entries never apply to self-sends (a machine sending
  to itself); a self-send can still be priced by its event-name entry.
* Unknown machine, event or profile names are errors (typos otherwise silently
  mean 0).
* `--latency-profile <name>` overrides `[tests]`; a test with no entry uses no
  profile (all latencies 0, costs 0).

## Output

* When time tracking is active, a `<TimeLog>` line after each dequeue gives the
  machine's clock as `[best, worst]` in microseconds. Programs without time
  produce byte-identical logs (and JSON traces) to today.
* Assertion messages are the model's own (it formats `Elapsed(m)`).
* Phase 2: critical path on failure; `--latency-report` (max `Elapsed` per call
  site over explored schedules).

## Replay

A schedule only replays under the same profile and mode. When time tracking is
active the `.schedule` file gets one header line

    --pnext-time:timed=<0|1>;profile=<name or ->;hash=<16 hex digits>

(hash of the resolved profile entries). Replay with a mismatch is an error. The
line is absent otherwise, so other schedule files are unchanged.

## Phases

**Phase 1 (implement now):** clocks and affine forms, the seven foreign
functions, profiles with `--latency` / `--latency-profile`, `--timed`
(ready-time filtering and send reordering), `<TimeLog>`, replay header, tests:
the RetryBudget project and focused unit tests (correlation cancels, max rule,
collapse, profile precedence, timed filtering, determinism of replay).

**Phase 2:** narrowing, critical path, `--latency-report`, port the Raft and
FailureDetector tutorials and decide whether to keep going.

**Later, only if Phase 2 pays off:** native syntax (`examples/`).

## Open questions

1. Should `Elapse` become a scheduling point under `--timed` (finer
   interleavings at the cost of more steps)?
2. Is 32 symbols the right collapse threshold?
3. Should unknown names in a profile be warnings rather than errors?
