# Certified change bundles design

## Durable representation

The service stores an immutable, content-addressed event chain. `cb:root` is the only mutable coordination record: its store version is the global commit sequence and its value is the ID of the current event. Each event contains its predecessor ID and one operation. A deterministic SHA-256 over canonical JSON names the event record. The genesis event carries the initial heads. Reads follow the root's chain back to genesis and replay it into a private in-memory snapshot; no process cache is used.

A mutation reads the root and reconstructs that snapshot, decides its result, writes its event if absent, then CASes `cb:root` from the version it read to the event ID. The successful root CAS is the operation's linearization point. A lost CAS restarts from the new root. The event is immutable and may be left orphaned if the process dies before the root CAS; it is not visible to readers. A crash after the root CAS is safe because a retry reads the committed operation. Genesis initialization follows the same immutable-record-then-root pattern.

Publish checks the durable request memo first, then evaluates revision, published state, readiness, and expected branch heads in the specified order. Its one event contains either the rejection memo or the successful outcome together with all head changes and the published flag. Thus those facts become visible at the same root CAS. `read_heads` and `get_status` pin one root ID and replay only that chain, giving a consistent snapshot. Returned updates are copied through JSON encoding so callers cannot mutate service state.

## Failure and concurrency behavior

Concurrent operations race on the root CAS and retry against the winner's complete state. This serializes revisions against approvals/CI and makes publishing mutually exclusive with changes to the same bundle. Content-addressed orphan records can be reused if a retry builds the same event. Store responses lost after a successful event write or root CAS do not compromise durability. Publish request outcomes, including failures such as `not_ready` and `conflict`, are recorded in the event history, so a reused request ID always returns its original outcome.

## P model

`Bundle.p` models two approved, CI-passing bundles competing to advance a shared head from the same base. The server handles each publish as the equivalent of a root-CAS transaction, memoizing every request. `CommitChain` checks each observed successful commit extends the prior visible head and that no second competing bundle commits. I ran `/opt/snappea check Bundle.p -n 200 --seed 7`; all 200 explored schedules passed. This informed the Python design's single root CAS around the outcome memo, published flag, and full head update.

The model abstracts away the event-chain storage protocol, root version CAS retries, caller crashes, and branches beyond one scalar head. It checks the atomic publication decision at the logical level; it does not establish correctness of the Python implementation or model storage recovery.

## Limits

Every operation replays the committed event chain, so work grows linearly with the history. This keeps records small and the protocol simple for the stated hundreds-of-bundles workload, at the cost of read latency. As with any content-addressed record layout, it assumes SHA-256 collisions do not occur. Each individual event must fit the store's 16 KiB record limit.
