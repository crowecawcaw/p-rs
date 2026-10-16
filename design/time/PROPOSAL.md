# Latency budgets for P (design proposal)

Status: draft for discussion. Nothing here is implemented.

## Motivation

Distributed systems have latency ranges per hop, budgets per request, retries,
timeouts and alternative paths (cache hit/miss, failover). P checks the *logic* of
such systems but has no notion of time: timers fire on `$`, and the scheduler
interleaves without regard to how long anything takes. We want to:

* declare how long message delivery and local work take (as ranges),
* compute how long a request took along the path an execution actually followed,
* assert that this is within a budget, and get a replayable trace when it is not.

## Core idea: causal cost clocks

Every machine has a local clock. Every sent event carries a *stamp*:

```
stamp(e)        = clock(sender) at send + latency(e)
on dequeue:       clock(receiver) = max(clock(receiver), stamp(e))
elapse d:         clock(self)     = clock(self) + d
```

This is a Lamport clock with weights. It gives critical-path latency for free:
sequential work sums, parallel fan-in takes the max, retries accumulate in the
loop that performs them, and a machine busy with other work delays the next event
(head-of-line queueing on a single-threaded actor).

In the base mode the clocks only *observe*; they never change which schedules are
explored. That keeps the feature inside P's model (properties over executions) and
costs nothing for programs that do not use it.

## Values are intervals

A `duration` is an interval `[best, worst]`; a point duration has `best == worst`.

| Expression | Meaning |
|---|---|
| `5ms`, `2s`, `250us` | point literals |
| `5ms..20ms` | range literal |
| `a + b` | `[a.best + b.best, a.worst + b.worst]` |
| `a - b` | `[a.best - b.worst, a.worst - b.best]` |
| `k * a` (`k: int >= 0`) | scaled |
| `max(a, b)`, `min(a, b)` | componentwise |
| `best(a)`, `worst(a)` | point durations |
| `a <= b` etc. | see below |

**Comparisons are three-valued.** `a <= b` is *certainly true* if
`a.worst <= b.best`, *certainly false* if `a.best > b.worst`, and *uncertain*
otherwise. Uncertain comparisons resolve as follows:

* In `assert`, uncertain counts as false. An assertion over durations therefore
  states a worst-case bound: `assert now() - t0 <= 100ms` fails if the request
  *could* have taken more than 100ms on this path.
* In `if`/`while` conditions, an uncertain comparison is a nondeterministic
  choice (like `$`), recorded in the schedule and replayable. Model code that
  branches on time (`if (now() > deadline)`) thus explores both outcomes when
  both are possible. The interval is not narrowed after the branch (a known
  imprecision, see Open questions).

Intervals rather than sampling: deterministic, no extra state space, and the
reported number is the true worst case of the explored path rather than a lucky
draw.

## Syntax

### Delivery latency

```p
event eReadReq : tReadReq latency 1ms..5ms;      // default for this event type
send backup, eReplicate, r latency 40ms..60ms;   // per-send override
send this, eTimeout latency 300ms;               // a timer is a delayed self-send
```

Events without a declared latency are delivered with latency `0ms`.

### Local work

```p
elapse 2ms..8ms;
elapse sizeof(batch) * 100us;
```

### Reading clocks

* `now()` in a machine: the machine's local clock.
* `now()` in a spec handler: the *send* stamp of the observed event (the
  sender's clock at the moment of the send, before delivery latency).
* `arrival()` in a spec handler: the delivery stamp of the observed event
  (`now() + latency`), i.e. the earliest time the receiver can process it.

### Latency profiles

Deployment numbers belong to tests, not to protocol code:

```p
latency profile SameAZ {
  default            0.5ms..2ms;   // any event without a more specific entry
  eReadReq           1ms..3ms;     // per event type
  (Client, LB)       1ms..5ms;     // per (sender machine type, receiver machine type)
  (LB, Storage).eRead 2ms..4ms;    // per link and event type
  Storage.Serving    2ms..8ms;     // charged on entry to every handler in that state
}
latency profile CrossRegion extends SameAZ {
  (LB, Storage)      60ms..90ms;
}

test tcAZ     [main=Main, latency=SameAZ]:     assert ReadSLA in (union Sys, { Main });
test tcRegion [main=Main, latency=CrossRegion]: assert ReadSLA in (union Sys, { Main });
```

Most specific wins: per-send override > `(A, B).e` > `(A, B)` > `e` > `default`
> event declaration > `0ms`. Handler costs from the profile are added to any
`elapse` in the handler.

### Budgets

Budgets are ordinary assertions, usually in `spec` monitors so they reuse traces,
replay and coverage:

```p
spec ReadSLA observes eReadReq, eReadResp {
  var started: map[int, duration];
  start state S {
    on eReadReq  do (r: tReadReq)  { started[r.id] = now(); }
    on eReadResp do (r: tReadResp) {
      assert arrival() - started[r.id] <= 100ms,
        format("read {0} took up to {1}", r.id, arrival() - started[r.id]);
    }
  }
}
```

A spec sees the *arrival* stamp, not when the client actually dequeues the
response; for budgets that must include the client's own queueing, assert in the
client with `now()`.

## Checker output

* Traces print the clock with each step: `<Dequeue> 'eReadResp' ... @ [42ms, 118ms]`.
* On a failed duration assertion, the trace ends with the critical path: the chain
  of sends/elapses that produced the `worst` bound.
* `--latency-report` prints, per assertion site, the maximum `worst` value seen
  across all schedules and the schedule that produced it, even when no bug is found.

## Phase 2: timed scheduling (`--timed`)

In the base mode the scheduler may deliver events in an order that is impossible
in time (e.g. a 300ms timeout before a reply that arrives within 20ms). That is
sound for safety (it over-approximates) but produces false positives in
timeout-heavy models. With `--timed`, the scheduler only dequeues an event `e`
when no other enabled event `f` satisfies `f.worst < e.best`. Overlapping
intervals stay nondeterministic. Dequeuing `e` ahead of a pending `f` also
*narrows* `e`: its time is clamped to at most `f.worst` (it happened no later than
`f`). Without narrowing, "first reply wins" patterns such as hedging (example 6)
would keep the slow reply's full interval. This turns timers into real timers and
makes P a bounded discrete-event simulator. It changes exploration and so is
opt-in.

## Non-time costs

Additive costs (RCU, rows read, RPC count, retry amplification) sum across all
branches rather than taking the max, need no causal propagation, and are already
expressible as a spec monitor summing payload fields (see example 10). No language
feature is proposed for them now. The stamp is internally a small record, so a
later `resource rcu : int join sum;` could ride the same propagation.

## Out of scope

Percentiles and distributions (intervals give per-path bounds, not p99), wall-clock
skew between machines, and changing the default (untimed) scheduling.

## Open questions

1. Should `if` on an uncertain comparison narrow the interval in each branch?
   Precise but needs per-variable refinement.
2. Should `0ms` really be the default latency, or should using durations at all
   require every event to have one?
3. Does the per-state handler cost in profiles pull its weight, or is `elapse`
   enough?

## Examples

See `examples/`:

| # | File | Shows |
|---|---|---|
| 1 | `01_client_server_sla.p` | event latency, `elapse`, assert in client with `now()` |
| 2 | `02_cache_profiles.p` | cache hit/miss paths, profiles, two test cases |
| 3 | `03_scatter_gather.p` | fan-out/fan-in = max |
| 4 | `04_quorum_read.p` | 2-of-3 quorum, k-th fastest reply |
| 5 | `05_retry_backoff.p` | timeouts as delayed self-sends, backoff, `--timed` |
| 6 | `06_hedged_request.p` | hedging, first reply wins |
| 7 | `07_deadline_propagation.p` | deadlines in payloads, branching on uncertain time |
| 8 | `08_timeout_ordering.p` | client < LB < server timeouts as a spec |
| 9 | `09_failover.p` | leader failure, election and redirect within budget |
| 10 | `10_capacity_costs.p` | RCU and retry amplification with today's monitors |
