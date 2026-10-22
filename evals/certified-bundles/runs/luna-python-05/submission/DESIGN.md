# Certified change bundle service

## Durable layout

The service stores one mutable key, `certified-bundles/root`, whose value is the
hash of the newest immutable event. Every event contains its predecessor hash and
one state transition. Event records use content-addressed keys under
`certified-bundles/event/`; they are written once and never changed. The initial
heads are stored in the first event. Replaying the immutable chain reconstructs
heads, current bundle revisions and evidence, and publish request outcomes.

This keeps each write bounded even when the history grows beyond one store value.
The store is the only persistence and synchronization mechanism. An orphan event
is harmless: it is unreachable unless the root CAS commits it. Concurrent writers
may create separate orphan events, but only one can advance a given root version.
The loser reloads state and recomputes its operation.

## Linearization and recovery

Appending an operation writes its immutable event, then compare-and-swaps the
root pointer. That successful root CAS is the operation's linearization point.
For read-only results (including rejected operations and duplicate approvals),
the root read that supplied the evaluated snapshot is the linearization point.
`read_heads` and `get_status` each read one root pointer and replay only events
reachable from it, so each response reflects one consistent snapshot.

A publish event contains its request outcome and, for successful first publication,
the entire set of new heads plus the published marker. One root CAS exposes all
of these together. Every publish outcome is kept in the log, so retrying a request
ID returns its first outcome before reevaluating current state.

If a process dies after writing an event but before advancing the root, recovery
ignores the orphan and a retry can recreate or reuse the same content-addressed
event. If it dies after the root CAS, the event and result are already visible;
retry observes the committed state. A failed CAS from concurrent activity causes
an operation to replay against the new root. No in-memory state is trusted between
calls or Service instances.

## Limits

Reads replay the event history, so work grows linearly with the number of committed
events. The design targets the specified hundreds of bundles and bounded per-event
payloads; an individual event still must fit the store's 16,384-byte value limit.
