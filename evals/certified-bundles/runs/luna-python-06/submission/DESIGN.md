# Certified change bundle service

## Durable layout

The service uses one durable key, `cb:root`, as the shared commit pointer. Its
value names an immutable log record. Each log record contains the previous record
key and one event; the record key is the SHA-256 hash of its canonical JSON. The
initial record stores the initial branch heads. Later events represent submissions,
revisions, approvals, CI results, and publish outcomes. The current state is
reconstructed by replaying the chain from the captured root back to the initial
record.

Log records are written with compare-and-swap at version zero before the root is
changed. Losing an operation after a record write leaves an unreachable record,
which is harmless. Concurrent writers can reuse an identical record for the same
parent and event. They then compare-and-swap `cb:root` at the version they read. A
failed root CAS means another operation committed first, so the caller loads the
new state and reevaluates its rules. This makes each successful root CAS the
linearization point for a state-changing operation.

## Operation semantics and recovery

The commit pointer makes a publish outcome and all of its head changes visible as
one operation. Rejected publish outcomes are also logged, keyed by the request ID,
so retries return the original result. Submissions keep their original response in
the bundle record, preserving the revision-one reply after later revisions. Every
process restart rebuilds state from durable records; local instance state does not
carry correctness responsibilities.

Reads capture the root once and replay only that immutable chain, so a concurrent
commit cannot mix branch heads from different moments. A lost root-CAS reply is
safe: a retry observes the committed event and either returns its idempotent
result or reevaluates against the new state. Uncommitted records do not affect
reads.

## Limitations

Replay work grows with the number of committed operations, and immutable abandoned
records are not collected. The implementation is intended for the stated hundreds
of bundles and modest operation history; it favors a compact correctness argument
over write throughput. Each event is stored in one JSON value, so a single update
map larger than the store's 16,384-byte record limit cannot be represented by this
layout.
