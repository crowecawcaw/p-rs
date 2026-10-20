# Time, timeouts and latency budgets in p-rs (design)

Status: Phase 1 is implemented in `prs::checker` (`crates/p-rs/src/checker/time.rs`) and
`prs check` (`--latency <file>[:<profile>]`, `--timed`), tested in
`crates/p-rs/tests/checker_time.rs`. Time in `prs observe` is documented in
[docs/observe.md](../../docs/observe.md#time-and-latency-budgets). The worked example is
[RetryBudget](RetryBudget/README.md).

**No language change.** Models use plain P (foreign functions and ordinary machines).
`examples/` sketches a possible later native syntax over the same semantics; those files
do not parse with `prs`.

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

A model opts in by declaring any of these global foreign functions. Times are
`int` microseconds.

```p
fun Now(): int;                // worst case of this machine's clock
fun NowBest(): int;            // best case
fun Elapse(lo: int, hi: int);  // local work taking lo..hi us
fun ElapseCost(name: string);  // local work priced by the profile's "cost.<name>"
fun Mark(): int;               // stopwatch handle for the current clock
fun Elapsed(m: int): int;      // worst case of (clock - mark m)
fun ElapsedBest(m: int): int;  // best case
```

* p-rs implements these natively and does not consult a user's Rust
  `ForeignFunctions` for them. Only global declarations count (a machine-owned
  `fun Now()` is an ordinary foreign function). If a declaration uses one of these
  names with a different signature, that is an error that names the expected
  declaration.
* Declare only the functions you use.
* `Elapse` needs `0 <= lo <= hi <= 2^40`; other arguments are a checker
  error (an exception in the model).
* `Mark` handles count up from 0 within each schedule. They are deterministic
  and appear in payload logs. An unknown handle is an error.

Timers are ordinary machines (`RetryBudget/PSrc/Timer.p`). Each
armed timer is a fresh machine that calls `Elapse(d, d)` and then sends the
timeout. Because the timer is a separate machine, the scheduler already decides
whether a reply or the timeout is handled first.

## Semantics

### Clocks

Time tracking is active for a test when the program declares any function above,
or when `--latency` or `--timed` is given. Otherwise nothing is computed, and
logs and traces are byte-identical to today's.

| Operation | Effect |
|---|---|
| machine created by `new` | child clock = creator clock (Main starts at 0) |
| `Elapse(lo, hi)` | clock += a fresh draw of `lo..hi` |
| `ElapseCost(n)` | clock += a fresh draw of the profile's `cost.n`; 0 without a profile; error if the profile has no such entry |
| `send` | event stamp = sender clock + delivery latency (a fresh draw, see Profiles) |
| an event is dequeued, or taken by a `receive` | clock = max(clock, stamp) |
| ignored event dropped | no effect |
| monitors | read the clock of the machine whose send or announce they observe; 0 in a spec's start entry |

Clocks only grow. Ranges are at most 2^40 us. With the step bounds, that keeps
the arithmetic far from overflow (saturating arithmetic is only a backstop).

### Values: affine forms

A plain interval is wrong for budgets: in `Elapsed(m)` the mark and the clock
share history, and interval subtraction would add their widths instead of
cancelling them. Each time value is therefore an affine form:

    v = c + Σ a_i·ε_i,   ε_i ∈ [0, 1], one fresh symbol per draw

* Stored clocks and stamps have `a_i ≥ 0`. Differences (inside `Elapsed`) can
  have negative coefficients.
* `best(v) = c + Σ min(0, a_i)` and `worst(v) = c + Σ max(0, a_i)`.
* A range `lo..hi` drawn once is `lo + (hi - lo)·ε_new`.
* `+` and `-` are exact: shared symbols cancel.
* `max(a, b)` is exact when the sign of `a - b` is certain. Otherwise it is sound
  but not exact, and it is one of these three forms:
  * `b + ε_new·worst(a - b)`, which keeps `b`'s correlation;
  * `a + ε_new·worst(b - a)`, which keeps `a`'s correlation;
  * a fresh `[max(best a, best b), max(worst a, worst b)]`, which keeps none.

  The form adding the least new (uncorrelated) width wins, then the one with the
  smaller worst case, then the order above. In a join, `b` is the incoming stamp,
  so a reply that descends from the client's own request keeps cancelling
  against the client's marks.
* A form with more than 32 symbols collapses its smallest symbols into one fresh
  symbol. Its own range is kept; correlation through those symbols is lost, which
  only widens later differences. Whether 32 and "smallest first" are right is to
  be measured on Raft in Phase 2.

### Delivery order (default mode)

Delivery follows P: an event enters the receiver's queue when it is sent, and
queue order is send order. Latency changes stamps, not order. Consequence: an
event sent earlier with a long latency is handled before one sent later with a
short latency, unless `--timed` is on.

### `--timed` scheduling

Untimed, the scheduler may run steps in orders that are impossible in time, for
example a timer's 300 ms timeout handled before a reply stamped 20 ms. With
`--timed`:

1. **Events are picked at the receiver's scheduling point.**
   * When a machine finishes a handler and has events to take, it yields first.
   * A `receive` whose events are already queued yields first.
   * A blocked `receive` is enabled by a matching send, but it takes nothing
     until it runs.
   * When the machine resumes, the event comes from a candidate set that
     contains the first eligible event of each sender (per-sender FIFO). Eligible
     means not deferred or ignored, or for a `receive`, listed in its cases.
     Candidates that certainly arrive after another candidate
     (`best(c) > min worst`) are dropped. If more than one remains, which one is
     taken is a nondeterministic choice, recorded in the schedule like any
     other. Ignored events queued before it are dropped, as today.
   * Per-sender FIFO means an event's effective arrival is at least that of
     the same sender's earlier events to this receiver, whatever its own stamp.
2. **Ready times.** Before each scheduling decision, every enabled operation
   gets one:

   | Operation | Ready time |
   |---|---|
   | test task | 0 |
   | machine about to take an event (handler loop or `receive`) | `[max(best clock, min best c), max(worst clock, min worst c)]` over the candidates `c` |
   | machine whose state has a default handler | clock |
   | anything else (start, create, send, paused mid-handler) | clock |

   Operation `x` is removed from the candidates when some other enabled `y` has
   `worst(ready y) < best(ready x)`. Ties and overlaps stay nondeterministic. The
   earliest operation is never removed, so no deadlocks are introduced.
3. **Default handlers.** The default handler runs at the machine's clock.
   Candidates that certainly arrive after the clock (`best(c) > worst(clock)`)
   are not there yet. If some candidate certainly arrived before the clock
   (`worst(c) < best(clock)`), an event is taken. Otherwise the default handler
   is one more option in the nondeterministic choice.

Notes:

* `Elapse` is not a scheduling point; its effect counts at the machine's next
  one.
* Specs observe sends and announces in schedule order, not time order. A spec
  that is sensitive to the order of observations from different machines can
  see an order that is impossible in time. See open question 1.
* The filter changes which choices the strategies see. Seeds therefore do not
  reproduce untimed runs, and POS racing only sees the unfiltered operations.
* Timed traces have extra scheduling points (the yields in step 1), so a
  schedule recorded with `--timed` only replays timed. The schedule file says so
  (see Replay).
* **Zeno loops.** A machine that loops with no cost (a default handler, or
  zero-latency self-sends) keeps the lowest ready time. Every operation that is
  certainly later then starves, and the run ends at the step bound, which can
  show up as a liveness report. Give such loops a cost (`Elapse`, or a profile
  entry for the self-sent event). A "time did not advance" diagnostic is
  planned for Phase 2.

Deferred to Phase 2: **narrowing**. When `x` is chosen ahead of an overlapping
`y`, record `x ≤ y`. Without it, "first reply wins" patterns (hedging) get
pessimistic bounds.

### What results mean

Clock values over-approximate:

* queueing only delays;
* ranges are kept as ranges;
* an inexact max or a collapse only widens.

So for a **monotone** budget assertion, such as `assert Elapsed(m) <= budget`
(or `Now() <= deadline`):

* **A pass is sound only under `--timed`.** Untimed, queue order is send order,
  so orders that are possible in time (a later send arriving first) are never
  explored, and an untimed pass is informative only. Under `--timed`, if the
  assertion holds on every explored schedule, it holds for those schedules
  under the stated latencies. This needs the model not to branch on time
  values: `if (Now() > deadline)` always reads the worst case,
  so the branch where the deadline has not yet passed is never explored.
* **A failure is possible, not certain.** The schedule can exceed the budget
  when every draw is at its worst. It can also be spurious, in any of these
  ways:
  * an inexact max or a collapse;
  * untimed mode exploring an order that is impossible in time (run with
    `--timed`);
  * an overlap that `--timed` resolved without narrowing.

  When every value involved is a point (as in RetryBudget's give-up path), the
  failure is exact.

As always, schedules are sampled. Checks in the other direction (for example
`ElapsedBest(m) > x`) can be masked. An inexact max also inflates `Now()`,
which makes `Now()`-based deadline checks pessimistic.

## Profiles

A small TOML subset, passed with `prs check --latency <file>`:

```toml
# comments, [sections], key = "string" only

[profile.same-az]
default        = "500us..2ms"   # any send without a more specific entry
eReq           = "1ms..2ms"     # by event name
"(Client, LB)" = "1ms..5ms"     # by (sender, receiver) machine declaration names
"(LB, Db).eGet" = "2ms..4ms"    # by link and event
"cost.DbQuery" = "2ms..8ms"     # priced by ElapseCost("DbQuery")

[profile.cross-region]
extends        = "same-az"
"(LB, Db)"     = "60ms..90ms"

[tests]
tcRetry = "same-az"             # profile per test case
```

* **Values.** `"lo..hi"` or a single value, each an integer with unit `us`, `ms`
  or `s`, at most 2^40 us.
* **Precedence**, most specific first: `(A, B).e`, `(A, B)`, `e`, `default`,
  0. `extends` copies the parent's entries first.
* **Self-sends.** A self-send is a machine instance sending to itself; two
  `Server` instances still use `(Server, Server)`. A self-send is priced only by
  its event-name entry: `default` and link entries do not apply. That keeps
  internal `raise`-like sends free unless priced explicitly.
* **Names.** Profile names must not be empty or `-` and must not contain `;`,
  `=` or `"`. Machine names are declaration names, not interface names. An
  unknown machine or event name is an error, and so are unknown profiles,
  duplicate keys and duplicate `[tests]` entries. The keys `default` and
  `extends` are reserved, so an event with either name can only be priced
  through `(A, B).e` keys. Cost names cannot be checked statically;
  `ElapseCost` with a name the active profile does not price is a runtime error.
* **Choosing a profile.** `--latency <file>:<name>` overrides `[tests]`. With
  `--latency`, a test that has neither is an error, so all-zero latencies never
  happen silently. Without `--latency`, there is no profile: latencies and costs
  are 0.

## Output

* When time tracking is active, a `<TimeLog>` line follows each dequeue or
  receive, giving the machine's clock in microseconds (`[best, worst]`, or one
  number when exact). The JSON trace has no time entries in Phase 1.
* Assertion messages are the model's own (it formats `Elapsed(m)`).
* Phase 2: the critical path on failure, and `--latency-report` (the maximum
  `Elapsed` per call site over explored schedules).

## Replay

When time tracking is active, the `.schedule` file gets these lines:

    --prs-time:timed=<0|1>;profile=<name or ->
    --prs-latency:<key>=<lo>us..<hi>us      (one per resolved profile entry)

`prs replay` takes the mode and profile from these lines, so it has no time
options. A replay therefore needs no latency file and cannot silently run a
different mode. The profile's names and the time-function signatures are still
checked against the program. A schedule without these lines was recorded
without time options, so its replay ignores them too (untimed, no profile).
Builds that do not know the lines skip them, as they skip any unknown `--`
line, and diverge.

## Phases

**Phase 1 (done):**
* clocks and affine forms;
* the seven foreign functions, with signature checks;
* profiles via `--latency <file>[:<profile>]`;
* `--timed`, with pick-at-resume, a recorded choice among overlapping
  candidates, and ready-time filtering;
* `<TimeLog>` lines;
* self-contained replay headers;
* tests: the RetryBudget project, arrival order under `--timed` (handler and
  `receive`, certain and overlapping), default handler against overlapping
  events, timeout-vs-reply filtering, stopwatches and cancellation, the max
  rule, collapse, profile parsing and errors, and replay without a latency file.

**Phase 2:**
* narrowing;
* a "time did not advance" diagnostic;
* the critical path and `--latency-report`;
* measuring the collapse policy;
* porting the Raft and FailureDetector tutorials, then deciding whether to keep
  going;
* **done:** time in `prs observe` (issues #1 and #2): checking budgets against
  production trace timestamps as exact epoch microseconds, with per-key stopwatches.

**Later, only if Phase 2 pays off:** native syntax (`examples/`).

## Open questions

1. Should `Elapse` (and `announce`) become scheduling points under `--timed`?
   That would give finer interleavings and time-ordered spec observations, at
   the cost of more steps.
2. Is 32 symbols, folded smallest first, the right collapse policy?
3. Should a profile's link keys also accept interface names when they are
   unambiguous?
