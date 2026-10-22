# Certified change bundles

## P/Snappea model

`CertifiedBundles.p` is the small state-machine model. It abstracts one bundle that
changes two branches. Approvals and CI arrive as separate messages, revisions clear
both kinds of evidence, and publish checks the revision, two approvals, passing CI,
and both expected branch heads before changing either head. The paired head writes
are one abstract transition. The model captures the core safety gate and atomicity
property; the Python implementation adds request memoization, arbitrary branch maps,
and the durable store protocol.

## Durable representation

The store has one mutable key, `ccb/root`, and immutable event records under
`ccb/event/<sha256>`. Every event includes the previous root tail and one logical
state change. Event JSON is canonicalized before hashing. The event record is written
first; then a compare-and-swap on the root publishes its key. The root key's store
version is the commit sequence. Replay from the root tail reconstructs heads, bundles,
submit replies, publish replies, and current evidence. It is intentionally a simple
append-only representation so old bundles and request outcomes stay durable without
putting the whole history in a size-limited value.

A store CAS on an immutable event key may leave an orphan if the process dies before
publishing the root pointer. Orphans are invisible and safe. Competing changes based
on one root can each write candidate events, but only one root CAS succeeds. Losers
read and replay the winning root, reevaluate their rule, and retry. If a process loses
the response after the root CAS, the event is already visible; replay gives the next
instance the committed state and the request's memoized result.

## Linearization points and idempotency

For a state-changing operation, the successful CAS of `ccb/root` is its
linearization point. It publishes both an operation result memo (where required) and
its state changes at once. Each publish event contains the outcome and, only on the
successful first publication, all target head values. Therefore no reader can observe
a memoized `published` result without the corresponding complete head update. Submit
uses a durable response record keyed by its request/bundle ID; retrying the same ID
returns the first response. Publish outcomes are similarly memoized by publish request
ID, including rejection outcomes.

A read first captures the root key and then follows immutable records from that tail.
That root read is the snapshot point. Later root commits cannot alter any event the
reader follows. Returned nested update maps are rebuilt/copied, so callers cannot
mutate stored or cached state.

Invalid/stale checks that make no state change return from the captured snapshot. A
valid duplicate approval or identical CI result also returns from that snapshot; the
linearization point is the root read. For updates that change state, the operation
retries after a failed root CAS and makes its decision against the new snapshot.

## Limits

The journal is unbounded, so replay work grows with committed operations. Each event
is small (a submit or revision carries its update map), and the tested workload of
hundreds of bundles remains within the store-call budget. Inputs whose single event
exceeds the store's 16 KiB record limit cannot be represented by this layout. The
implementation uses SHA-256 content addressing for immutable records; a hash collision
is checked and raised as a storage-integrity error. The design assumes the contract's
fixed branch set and identical initial heads across service instances.
