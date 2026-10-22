# Certified change bundles

The implementation is in `solution.py`. It keeps a single durable commit root and an
append-only journal split into one record per committed operation. The root contains
the committed sequence number, the initial branch heads, and either no pending item
or one reserved journal event. The initial branch heads fit in the root; the later
history lives in journal records, so the value-size limit does not grow with the full
history.

Every operation first replays the committed journal prefix into a temporary state.
A writer then reserves its event by CAS on the root, conditional on the sequence
number it observed. A sequence mismatch makes it re-read and re-evaluate the rule.
That conditional check makes decisions such as approval, CI acceptance, conflicts,
and publish outcomes apply to precisely the state at their serialization point.

After the reservation CAS, any caller can finish the event: it writes the event to
its numbered journal key and CASes the root to advance the committed sequence and
clear the reservation. Helpers can safely race because the reserved event is
identical for all of them. A crash before the journal write leaves the reservation
available to the next caller; a crash after the journal write lets a helper finish
the root update. An orphan journal item cannot be produced by an unreserved writer.
If a response is lost after the final root CAS, the event is already visible and a
retry derives the durable outcome from replay. Submit IDs and publish request IDs
are recorded in separate maps in the replayed state.

The final root CAS is the linearization point for each state-changing operation. For
reads and rejected operations that do not need a durable outcome, the root read that
selects their committed prefix is the linearization point. Publish always appends an
event, including rejected outcomes; for success, replay applies all target head
changes and records the request outcome as one journal event, made visible at the
same root CAS. `read_heads` replays one prefix, so all requested heads come from one
snapshot. Returned mutable data is copied away from the store and replay state.

`CertifiedBundles.p` is a bounded Snappea model of the one-bundle, one-branch
publication safety conditions. Its monitor rejects a publish without two distinct
approvals, passing CI, and the expected current head, and rejects double application.
It abstracts away the unbounded journal representation and models successful commits
as atomic observable events; the Python journal protocol provides recovery for the
interruptions abstracted out of that model.

The tradeoff is serialized writes and journal replay on reads. This favors simple
recovery and a single visibility point. It assumes each individual event and the
initial root fit the store's 16,384-byte record limit; arbitrarily large update maps
cannot be represented by one event under the supplied store interface.
