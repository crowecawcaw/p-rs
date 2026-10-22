# Certified change bundle service design

The service stores one append-only, immutable history as a linked chain of JSON
records. A single durable root key points to the current tail. Every request reads
that tail once, then follows immutable parent links back to the genesis record and
replays the events to obtain its view of heads and bundle state. The history is
split across records, so the accumulated history does not need to fit in one store
value. Genesis is created lazily from the constructor's initial heads.

A state-changing operation builds one small event from its snapshot, writes the
immutable node with compare-and-set, then compare-and-sets the root from the
version it read to that node. Losing a root CAS means another operation committed
first; the caller takes a fresh snapshot and reevaluates. Deterministic node keys
make duplicate node writes harmless. Nodes that lose a race remain unreachable
and have no effect. Since only the root makes records visible, an event and its
state transition have one atomic visibility point: the successful root CAS.

That root CAS is also the linearization point for successful submit, revision,
approval, CI, and publish calls. Read-only calls use their initial root read as
their snapshot point. Calls whose result needs no write (such as stale revision,
invalid approval, duplicate approval, or bad CI digest) linearize at the snapshot
that justified the result. Publish always appends an event, including rejected
outcomes, so its request ID and result become durable at the same root CAS as any
head changes. A retry first checks the memoized result. Submit records the initial
digest with revision one, so retrying after revisions returns the original result.

After a crash before the root CAS, any written node is an unreachable orphan and
retry proceeds from the previous root. After a crash after the root CAS, the whole
event is durable; retries discover its effect by replaying the history. A fresh
service instance needs no process-local state, locks, or caches. Revision events
replace the updates and clear approvals and CI; publication checks expected heads
and records all head changes in one event.

The layout favors simple recovery and correctness over read cost: replay work grows
with the number of committed operations. Each event is a separate store value, and
large individual update maps must still fit in the store's 16 KiB value limit after
the event envelope is added. The implementation relies on the stated durable,
linearizable store semantics and on identical initial heads across instances.
