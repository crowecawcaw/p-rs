# Certified change bundles

## Abstract state machine

The service is modeled as one atomic state machine with state
`(heads, bundles, publishResults)`. A bundle has a revision, full update map,
its digest, approval set, CI bit, and published bit. `heads` maps every branch
to its current opaque commit ID. `publishResults` maps publish request IDs to
outcomes. The P/Snappea abstraction treats each API operation as one transition;
this captures the externally visible contract while hiding the storage layout.

The transition rules are:

```text
Submit(id, U): if id is absent, add (revision=1, U, digest(U), {}, false, false)
Revise(id, r, U): if r=current and not published, replace U/digest,
                   increment revision, clear approvals and CI
Approve(id, r, who): if authorized, current revision, and not published,
                      add who to the set
CI(id, r, d, pass): if current revision and digest and not published,
                     set CI := pass
Publish(id, r, q): if q has a result, return it
                   else evaluate stale, published, not-ready, conflict, success
                   and save that result; on success, update all heads and mark
                   published in this same transition
ReadHeads(B): return heads restricted to B from one state
```

The executable P source for this finite abstraction is [ChangeBundles.p](ChangeBundles.p). It models one bundle and two target branches; the implementation generalizes the maps and event history. This abstraction makes the key safety property explicit: a successful publish
changes every target head and the bundle's published flag in the same transition
that records the request outcome. No schedule can expose a partial branch update.
The model also preserves expected parent IDs in `U`, so CI certifies both the
candidate commits and their bases. Two concurrent successful publishes of one
bundle reduce to one state transition that applies heads; the next observes the
published flag. Rejections are transitions too because their request outcomes
must survive retries.

## Durable representation

`solution.py` refines that state machine with three key families in the supplied
CAS store:

- `ccb:root` is the single versioned commit pointer.
- `ccb:event:<id>` holds one immutable state delta and a pointer to the previous
  committed event.
- `ccb:blob:<id>` holds a 10,000-character chunk of JSON for an update map.
  Linked chunks keep each encoded store value under the 16,384-byte limit.
  A separate CAS allocator gives event and blob records unique IDs; gaps are
  safe and unreachable records are ignored.

To change state, a caller reads the root, replays that immutable event prefix,
checks the operation's conditions, allocates and writes its event (and any blob
chunks), then CASes the root from the observed store version to the new event ID.
A failed root CAS leaves only unreachable records; the caller rereads and
re-evaluates against the new state. The root CAS is the linearization point for
all mutations, including publish rejections. A crash before it leaves no visible
change. A crash after it leaves the complete change visible, so a retry discovers
the committed state and returns the original result. The root pointer contains
both the prior state reference and all state changed by an event, so head changes
and publish request memoization have one atomic visibility point.

Reads capture one root pointer and replay only that event prefix. Its records are
immutable, so a later root change cannot mix snapshots. The root read is the
linearization point for `read_heads`, `get_status`, and state-based no-op answers.
State is reconstructed locally on each operation, so separate Service objects
and restarts share no cache assumptions.

Submit IDs are keyed by bundle ID; replay preserves the first digest separately
from the latest revision so retried submissions always return revision 1's
response. Publish request IDs have their own global map and are checked before
all other publish conditions. Approval sets only accept the three named
reviewers. Every revision event replaces the map and clears evidence. CI events
store the boolean value, including `false`, so a failing run cancels an earlier
pass. A publish event applies the current update map only when it changes an
unpublished bundle to published.

## Progress and costs

After contention stops, each CAS retry loop eventually observes a stable root
and commits or returns from that state. The allocator is a separate short CAS
loop and does not block readers. Crashes can waste IDs and leave orphan chunks
or events but cannot block later calls. Event replay costs one read per event;
chunked maps add one read per chunk when they are replayed. This favors a simple
single linearization point and straightforward recovery over write throughput.
The immutable history grows without bound, and reconstruction work grows with
it. The design assumes the initial heads are identical as specified and that
individual request identifiers and metadata fit in one store record; update-map
payloads themselves are chunked.

## Validation

`python -m unittest -v test_public` passes all four supplied tests, covering
publish/restart, evidence invalidation on revision, idempotent rejected publishes,
and stale parent conflicts. Snappea's executable was not present in the provided
`/opt/snappea` location, so the state-machine abstraction above is documented
here rather than run as a separate checker project.
