# Certified change bundles

## Model

[bundle_model.p](bundle_model.p) is a P/Snappea monitor for a bundle that updates `schema` and `service`. It checks revision invalidation, two distinct approvals, CI replacement, base-head conflicts, publication eligibility, and the joint head transition at the `ePublish` event. The event is the service's publish linearization point. The model abstracts the store to completed operation events and focuses on the bundle protocol; the Python service's durable request memoization is an implementation refinement beyond this single-bundle monitor.

## Durable representation

The store contains one small mutable root record and immutable content-addressed objects. The root points to three persistent treaps: branch heads, bundle records, and publish-request outcomes. Each bundle record points to a fourth treap holding its branch update map, so a large update map does not have to fit in a bundle node. Treap nodes include a key, value, deterministic SHA-256 priority, and child object hashes. Each JSON object is encoded canonically and addressed by its SHA-256 digest. This keeps collection sizes from accumulating into one record; an update rewrites only the treap paths it touches. Unreachable objects from abandoned attempts are harmless garbage because object records are immutable.

Each operation reads the root version and its immutable snapshot, builds any changed objects, then uses CAS on the root version. A failed CAS discards that proposal and retries from the new root. The root CAS is the linearization point for every accepted mutation. For publication, the new heads, published marker, and request outcome all appear through the same root replacement, so no reader can observe a partial multi-branch advance. A rejected publish records its outcome in the same way, making every request ID idempotent, including rejected requests.

## Recovery and snapshots

If a process dies while writing immutable objects, those objects are either safely reusable by their content hash or unreachable. If it dies after the root CAS commits, the state is already durable; retrying an operation observes the committed bundle or publish memo. A lost response after an immutable-object write cannot change visible state. A lost response after root CAS is handled by the operation's caller retrying with the same request ID; submit returns the stored original digest and publish returns its stored outcome.

`read_heads` loads one root pointer and reads only immutable nodes reachable from that snapshot, so all requested heads come from one consistent state even if a writer commits concurrently. `get_status` similarly reads one bundle version. Returned nested update maps are copied through JSON serialization so callers cannot mutate stored state.

## Semantics and limits

Revisions clear approvals and CI; CI is compared with the digest of the full update map, including expected parents. Approval, CI, revise, and publish all recheck the root version before committing, so concurrent calls serialize through the store. Publish evaluates request memo, revision, already-published state, readiness, then all expected heads, in the contract's order.

The implementation requires JSON-serializable inputs as specified. Persistent treaps have expected logarithmic path length under SHA-256 priorities, with no global process state or cache. Content-addressed records are never reclaimed because the store offers no delete operation. The implementation only uses the store's public read and CAS operations.

## Validation

`python -m unittest -v test_public` passes all four public tests. A separate smoke run exercised 180 bundles across 64 branches, two concurrent service instances, and a retry after a simulated crash. `snappea check bundle_model.p` passes the `tcAtomicBundlePublish` P test (10,000 schedules).
