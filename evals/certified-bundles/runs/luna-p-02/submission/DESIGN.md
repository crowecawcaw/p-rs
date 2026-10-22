# Certified change bundles

## Implementation

`solution.py` stores the complete logical history as a durable append-only chain. `cb:root` is the single mutable commit pointer. Every history node is immutable and content-addressed by SHA-256 over its parent pointer and compact JSON event. Operations read the root and replay its chain into a temporary state, make their decision, prepare a node, then CAS the root from the observed version to that node. Failed CAS attempts replay and decide again. Orphan nodes from interrupted attempts are unreachable and have no effect.

The root CAS is the linearization point for submit, revision, approval, CI, and publish. Publish events carry both the memoized outcome and, for successful first publication, the entire target update map. Replaying that one event applies all changed heads, marks the bundle published, and records the request outcome together. Thus no reader can see only a subset of a publish. `read_heads` and `get_status` replay one captured root snapshot. Durable event history also makes a restart independent of process memory.

A submit event keeps its original digest separately from later revisions, so submit retries continue returning the original revision-1 response. Publish request IDs have their own replayed outcome map. Revision events clear approvals and CI. Accepted failing CI records `False`; publish checks stale revision, published state, approvals/CI, and expected heads in the contract's order.

## P model

`bundle_model.p` abstracts two already-certified bundles that both name parent head 0 and then race to publish different next heads. The shared store machine serializes compare-and-swap transitions and asserts that at most one succeeds. I ran `/opt/snappea check bundle_model.p --exhaustive`; snappea explored all 53 schedules and reported no bug. That result motivated keeping the real algorithm's single root CAS as the atomic decision and visibility point for branch updates.

The P model abstracts away durable node creation, log replay, request memoization, revisions, CI and crash points. It checks the concurrent stale-parent race only; it does not prove the Python implementation or the store protocol. Public interface tests were run with `python -m unittest -v test_public` and all four passed.

## Limits

History replay cost grows with the number of committed events, and the append-only history is not compacted. Events are bounded in size by the contract's update limit of 64 branches in expected use; as with any store layout, a single oversized event is rejected by the store. Correctness relies only on the documented store read/CAS behavior and SHA-256 collision resistance. No external persistence or synchronization is used.
