# Certified bundle service design

## Durable layout

The service stores one complete logical state consisting of branch heads, bundle
records, and publish request outcomes. The JSON state is split into 6,000-character
fragments under immutable page keys. A small root record identifies the committed
page set by a ticket and page count. The root key is the single commit pointer, so a
reader loads one root version and then reads exactly its immutable pages. It sees a
complete old or complete new snapshot even if another writer advances the root
while those pages are being read.

A shared ticket counter allocates unique page-key prefixes through CAS. Writers
serialize a prospective full state into pages first, then CAS the root against the
version they read. A failed root CAS leaves harmless unreachable pages and causes
the caller to reload and reapply its transition. The counter, root, and pages are
all ordinary records in the supplied store; no other persistence or synchronization
is used. Fragmenting permits aggregate history to grow beyond the per-record limit.

## Linearization and recovery

A state-changing operation linearizes at its successful root CAS. That CAS commits
all of the new state at once, including all changed heads and any publish outcome.
Thus a bundle cannot be partly applied. A no-op result (such as stale revision,
duplicate approval, or an invalid CI result) linearizes at the root snapshot read
from which it was decided. `read_heads` and `get_status` likewise use one root
snapshot.

Every publish request outcome is included in the committed state, including
`stale`, `not_ready`, and `conflict`. Repeating that request ID returns the saved
outcome before evaluating the current state. Bundle submit responses are retained
with their bundle, so retrying a submit ID returns its original revision-one
response. A revised bundle resets its approvals and CI result.

If a process stops while allocating a ticket or writing pages, the old root remains
committed; incomplete or unreferenced pages are ignored. If it stops after a root
CAS committed but before the response arrived, a retry reads the committed state
and observes the operation's effect or memoized result. Tickets are never reused,
so later writes do not overwrite pages referenced by an older root. The service
retries optimistic root conflicts until it can apply the operation to the current
state, which allows progress once contention stops.

## Limits

Every update currently rewrites the full state, so write and snapshot cost grows
with retained history. This favors simple atomic recovery over throughput. The
root is small regardless of history size; individual pages remain below the store
record-size limit. The interface does not specify bundle deletion, so records and
publish outcomes are retained indefinitely.
