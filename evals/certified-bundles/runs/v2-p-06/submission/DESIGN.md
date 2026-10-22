# Certified change bundle service

## P/Snappea model

`CertifiedBundles.p` abstracts the publication linearization point. Each `RootWriter`
stands for a caller committing a root transition. The event payload says how many
branch heads changed and whether that same transition memoized the publish result.
`AtomicBundlePublish` checks the safety condition that a two-branch bundle changes
either zero or both heads, and that every outcome is memoized. Two writers run
concurrently, so Snappea explores their schedules. The model focuses on the
coordination boundary; it abstracts the validation rules and bundle contents that
the Python service evaluates before attempting the root transition.

## Python algorithm

The durable log contains immutable event nodes and one mutable root pointer. Nodes
record initialization, bundle submission/revision, approvals, CI results, and
publish outcomes. Each node points to its predecessor. The root selects one complete
prefix of this log. Folding that prefix produces branch heads, bundle records,
submit responses, and publish request outcomes.

Each service lazily creates a genesis event if the root is absent. A snapshot reads
the root once, follows immutable parent links, and folds the events in order. For a
state-changing API call, the service evaluates its rule against that snapshot, writes
an immutable event node under an unused key using CAS, then CASes the root from the
version it read to the new node. On root CAS failure, it retries from a fresh
snapshot. Nodes made unreachable by a failed root CAS are harmless orphans. Node
keys are selected by probing; uniqueness is enforced by each node's absent-key CAS,
not by a process-local counter.

The root CAS is the linearization point for successful mutations. Read-only calls
linearize at their root read and return values folded from that immutable prefix.
Rejected revise/approve/CI decisions linearize at their snapshot read. Publish
always appends its outcome event, so every outcome is memoized by publish request
ID; the event simultaneously applies all branch head changes if its outcome is
`published`. A retry checks the memo before examining current bundle state.
Submission events retain the original response for idempotent retries.

A process crash before the root CAS leaves at most an orphan node. A crash after
the root CAS leaves a complete durable event, which a restarted service discovers
from the root. Losing a response after commit is handled by submit and publish
request memoization. Other API retries follow their specified revision and
current-state semantics.

## Limits

This design favors simple coordination over throughput: every mutation serializes
on one root and snapshots replay the event chain, so work grows with service
history. Events are compact for ordinary inputs; the store's 16 KiB per-value
limit still applies to a single event and to the initialization record. Old log
nodes are retained because the store has no delete operation. The service relies
only on the store's public read/CAS operations and does not require shared process
memory.
