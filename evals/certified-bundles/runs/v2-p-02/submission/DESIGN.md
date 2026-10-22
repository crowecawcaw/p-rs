# Certified change bundles

## P/Snappea model

The service is modeled as one abstract coordinator whose state is the committed
log prefix plus the state obtained by folding that prefix over the initial heads.
Each operation is an atomic command in the model. A write appends one immutable
command and then attempts the shared pointer compare-and-swap. A failed compare
retries from the new prefix; thus only commands on the pointer's reachable chain
are visible. The bounded state for the model is:

```text
heads: Branch -> Commit
bundles: Bundle -> (revision, updates, approvals, ci, published, submitReply)
publishResults: PublishRequest -> Outcome
```

The P transition for `Publish` follows the contract order: an existing request
result wins; then revision, already-published, readiness, and expected-head checks
are considered. On success, every target head and `published` change in the same
transition. Its invariants are:

```text
published(bundle) => ci(bundle) and |authorizedApprovals(bundle)| >= 2
published(bundle) => each target head equals that bundle's new commit
heads change only in a successful Publish transition
publishResults never changes once a request key is present
```

The model treats pointer CAS as the only commit transition. This is the key
abstraction: event preparation and allocator changes are private until the
pointer refers to the prepared event. Distinct concurrent proposals can be
prepared, but a proposal based on an obsolete prefix cannot commit. Replaying the
same bounded schedules in P/Snappea checks that retries and interleavings preserve
the invariants, including the competing-bundle conflict case.

## Python representation

`solution.py` stores one durable pointer, one monotonic ID allocator, and immutable
event records. Every event contains its predecessor ID and one operation with its
arguments. A call reads the pointer and reconstructs the current state by replaying
its predecessor chain from the constructor's identical initial heads. It evaluates
the operation on a private copy, writes a uniquely named event, then CASes the
pointer using the version read with that snapshot. That pointer CAS is the
linearization point. If another operation won first, the CAS fails and the call
retries against the new history.

This layout keeps the durable record size independent of total history: the
pointer, allocator, and per-operation records stay small, while history is spread
across records. Submit responses and publish outcomes are events, so a retry after
an uncertain response finds the durable result in the replayed state. Publish
success stores all head changes and the published flag inside one event. A publish
rejection is also an event, making its request ID permanently idempotent. Submit
IDs and publish IDs live in separate state maps.

The allocator reserves a unique event key using CAS. A crash after reservation or
after writing an event can leave an unreachable gap or orphan record; neither is
visible and a retry allocates a new ID. A crash after the pointer CAS leaves the
operation committed, and replay recovers it. Reads take one pointer snapshot and
replay only immutable records reachable from it, so `read_heads` and `get_status`
observe a single consistent prefix. Calls with outcomes that need no state change
(e.g. duplicate approval, stale revision, or already-recorded CI value) linearize
at their pointer snapshot.

The store's CAS provides the only cross-instance synchronization. There are no
coherent caches or process-local coordination assumptions. The tradeoff is replay
cost: each operation reads the committed event chain, so this favors the stated
hundreds-of-bundles workload over very long histories. Event records are bounded
by the store's 16 KiB value limit; the tested workload's update maps fit in one
record.
