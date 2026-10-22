# Certified change bundles

## Durable representation

`change-bundles/head` is the only mutable coordination key. Its value names an
immutable journal record, and its store version is the journal sequence. A root
record stores the supplied initial heads. Each transition record stores its parent
pointer and one event (submit, revise, approve, CI result, or publish result). Record
keys are deterministic SHA-256 names of the parent and canonical event. This lets a
retry reuse a record written just before a crash without depending on process memory
or generating a random identifier. Unreferenced records are harmless orphans.

A reader first captures the head pointer and then follows immutable parent links to
the root. Replaying those events reconstructs bundles, submit responses, publish
request outcomes, and branch heads. The walk cannot mix snapshots: later commits only
change the head key, while every record reachable from the captured pointer is
immutable. This layout keeps each store value bounded as history grows; it costs a
linear journal scan on each operation, acceptable for the specified scale.

## Synchronization, atomicity, and recovery

Every mutating method reads a snapshot, evaluates its rule against that snapshot,
writes the deterministic immutable transition record, then CASes `head` against the
captured store version. The successful head CAS is the operation's linearization
point. If it loses a race, it discards its decision and reevaluates from a new
snapshot. A false CAS may leave an orphan record but does not expose its event.

Publish records its request outcome and, on success, the complete branch-head map
change and published marker in one journal event. Therefore the head CAS exposes all
of them together. Memoized publish outcomes are checked before revision or readiness
rules, including outcomes such as stale, not_ready, and conflict. Submit responses
are likewise stored with the first submit event and replayed on retries. A process
that dies after writing an event but before moving `head` leaves an orphan; a retry
can safely reuse it. A process that dies after the head CAS has committed has already
made the entire logical operation durable, whether or not its caller received a
response.

Approvals and CI updates are serialized through the same head CAS, so a revision,
approval, CI result, and publish cannot observe partial or stale state. Reads of
heads and status use the captured immutable snapshot and return copies of mutable
values.

## P model

`model/AtomicJournal.p` models two concurrent publishers whose compare-and-swap
requests race on a shared version. A request retry uses the same request ID. The
journal handler represents the atomic CAS of the one shared pointer and updates two
branch heads plus the request outcome in that same step. The `WholeCommit` monitor
asserts that both heads match at every visible commit and that the version increments
once per commit. Each client also asserts that retrying an ID returns the same saved
result.

Checked with `/opt/snappea check model/AtomicJournal.p -n 100 --seed 5 -j 1`:
`tcAtomicJournal` passed all 100 explored schedules. This supports the choice of one
atomic head-pointer commit for the Python service. The model abstracts away the
immutable record write before CAS, Python event replay, oversized records, and crash
injection; it cannot establish those implementation details or Python correctness.

## Limits

Journal reconstruction work grows with the number of committed transitions. The
service has no compaction because safe compaction would require a separate protocol
for replacing the root without invalidating concurrent readers. As specified, the
service assumes callers provide valid maps and existing bundle IDs, and all store
values must fit the supplied store's size limit.
