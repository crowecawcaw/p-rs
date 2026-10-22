# Certified change bundle service

`Service` stores an append-only event chain under `change-bundles:event:*` and a
single mutable root under `change-bundles:root`. A committed event names its
previous event and contains one logical state delta. The root names the current
chain tip. Replaying that chain reconstructs branch heads, bundle records, and
publish-request results. The initial-head event is created lazily, so construction
does no I/O. Bundle records contain their current revision, complete update map,
digest, sorted approvals, CI result, and publication flag. The original submit
digest is retained separately from the current revision digest.

Each operation reads a root version and reconstructs that exact committed view.
It computes a delta, writes the corresponding immutable event with CAS-if-absent,
then CASes the root from the observed version to the new event. A failed root CAS
means another operation committed first; the operation retries from the new view.
The event key is a SHA-256 of its parent and canonical delta, so contenders proposing
the same event can share the immutable record. Orphan events from a crash before the
root CAS are ignored because no committed root points to them. Old chain nodes are
never modified or deleted.

The root CAS is the linearization point for every state-changing operation. In
particular, a successful publish delta includes the updated heads, the bundle's
published marker, and its request outcome in one event reached by one root CAS.
Rejected publishes also append their memoized outcome through the same root pointer;
retries first consult that durable memo. Submit retries return the retained original
digest even after revisions. Read operations linearize at their initial root read,
then replay only nodes reachable from that immutable tip, yielding a consistent
snapshot. Rejected/stale operations and duplicate idempotent approvals can linearize
at the snapshot read that justified their result.

The chain uses one record per committed logical mutation, keeping individual
records small and allowing history to grow beyond the store's 16 KiB value limit.
Replay cost grows linearly with committed events; this trades throughput for a
simple single-key atomic visibility point. There is no process-local cache or
synchronization. Store errors and cancellation propagate, so after a lost response
a fresh call reconstructs state from durable records. `read_heads` and `get_status`
return detached data rather than aliases into replayed state.

## P model

`bundle_model.p` models two publishers that both observe generation zero, durably
stage candidate event data, and then contend on the one root-CAS transition. The
store accepts at most one parent-generation commit; it updates two branch heads,
the publication marker, the request memo, and generation together. Assertions check
that multi-branch heads never split, a published marker always has a memo, and a
root commit cannot refer to unstaged event data. Staged data left by a crash is
unreachable, matching the event-chain recovery rule.

Checked with `/opt/snappea check bundle_model.p -n 1000 --seed 5 -j 1` (passed).
This abstraction informed the implementation's immutable event staging followed
by a single shared root CAS and grouping publish heads and idempotency outcome in
the same delta. It is deliberately small: it omits bundle revisions, approvals,
CI, rejected outcomes' precedence, event-key hashing, store size limits, and the
Python runtime's response-loss timing. It checks the atomic coordination shape,
not the implementation's full correctness.

## Validation

`python -m unittest -v test_public` passes all four public tests. An additional
concurrent 64-branch exercise completed with both publish request IDs returning
`published`, all heads advancing together, and 190 store steps. A submit retry after
a simulated crash immediately after its first durable write returned the original
revision-1 response.
