# Certified change bundles

## P/Snappea model

The model treats a service call as one atomic transition. It abstracts away the
CAS log, since the log is the implementation mechanism for making these same
transitions atomic. The finite-state model state is:

```text
ModelState = {
  heads: Branch -> Commit,
  bundles: BundleId -> (revision, updates, approvals, ci, published),
  submitResults: RequestId -> OriginalSubmitResponse,
  publishResults: RequestId -> Outcome
}
```

The P transition rules are `Submit`, `Revise`, `Approve`, `RecordCI`, `Publish`,
and `Read`. `Submit` inserts once; `Revise` increments revision and clears
approvals and CI; evidence transitions are revision and digest guarded; `Publish`
checks memoized request, revision, published state, readiness, and expected heads
in that order, then changes every affected head in the same transition as setting
`published` and memoizing the outcome. `Read` returns a projection of one
`ModelState`. The safety assertions are:

```text
Published(b) ==> every target head equals b.updates[target].new
Published(b) ==> b has two distinct authorized approvals and passing CI
Heads change only in a successful Publish transition
A publish request ID has one immutable outcome
A bundle revision change clears approvals and CI
```

This is the abstract P model boundary used for reasoning: every externally
visible operation corresponds to one transition over `ModelState`; the concrete
commit pointer described below implements that transition. It intentionally
leaves storage crash schedules out of the model and addresses those in the
recovery argument.

## Durable representation

The store has one small root key whose value is a monotonically increasing log
sequence. Each sequence number has an immutable log record in a separate key.
The initial state consists of the constructor's fixed initial heads and empty
bundle and request maps. Replaying records through the root sequence reconstructs
the logical state. Records are deltas: a submit stores its bundle and original
response; revise replaces the revision data; approval and CI records carry their
single change; publish carries its outcome and, on success, the whole target-head
map. This keeps history out of any one 16 KiB store value.

A writer reads the root and reconstructs that snapshot, computes its transition,
then CAS-creates the next immutable log slot and CAS-advances the root. The root
CAS is the linearization point. `read_heads` and `get_status` first read the root
and replay only that fixed prefix, so they see a single consistent snapshot even
if later writers commit while the read is in progress. Returned structures are
fresh copies.

A next-slot record can be durable while its writer has not yet advanced the
root. Any writer that finds such a slot helps advance the root before recomputing
its own operation. The record contains a complete valid transition based on the
current sequence, so helping it is safe; a stopped operation may be included as
an incomplete operation in a linearizable history. Once the root advances, its
record is visible. If a helper and original writer race to advance it, only one
root CAS succeeds. If a caller loses a response after root advancement, retrying
submit or publish finds the durable request result. A publish outcome and all
head changes share the same record and root-CAS visibility point.

No-op decisions (stale revision, unauthorized reviewer, duplicate approval,
wrong CI digest, or already-recorded result) are made against a consistent root
snapshot. They can be linearized at that snapshot read. Every first publish
request outcome is committed, including rejections; retries consult the request
map before looking at current bundle state. Submit and publish results have
separate maps and therefore separate ID namespaces.

## Limits

Replay is linear in the number of committed operations, trading read cost for a
simple append-only history that can grow beyond one store record. Calls use only
public store read/CAS operations; there is no shared process state. This assumes
individual deltas fit the store's documented 16 KiB value limit and the given
contract that initial heads and arguments are consistent across service
instances.
