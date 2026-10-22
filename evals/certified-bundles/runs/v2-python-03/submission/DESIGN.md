# Certified change bundle service design

The service stores one shared root pointer and an append-only chain of immutable,
content-addressed events. The base event contains the initial branch heads. Each
later event records one accepted operation, including the information needed to
reconstruct bundle metadata, publish outcomes, or updated heads. Event records
are small JSON values; the complete history is spread across records rather than
stored in one growing value.

Before changing the root, a caller writes its event under a key derived from the
SHA-256 hash of its canonical JSON. Event keys are installed with compare-and-set
from version zero, so an abandoned event is harmless and concurrent writers of
an identical event can share the same record. The caller then compare-and-sets
the root from the version it read to the event key. If that fails, it reloads the
new root, reevaluates the operation against the newer state, and retries. Thus a
root change makes an entire event visible atomically; readers first capture that
pointer and then replay only its immutable ancestors, obtaining one consistent
snapshot even if later commits happen during replay.

The root compare-and-set is the linearization point for successful submissions,
revisions, approvals, CI results, and publish outcomes. A publish event includes
its durable request-ID outcome and, on success, every changed branch head and the
published marker in the same root transition. Publish retries first consult the
outcome map, so all results, including rejection results, are idempotent. Reads
linearize when they capture the root pointer. Rejected operations that make no
state change linearize at the root snapshot on which their condition was checked.

Initialization is lazy. Instances independently write the same base event (given
the contract's identical initial heads) and race to install the root with CAS.
An event write whose response is lost but whose root update did not occur leaves
only an unreachable immutable record. If the root update committed before a
response was lost, a retry sees the committed state and applies the contract's
idempotency or revision rules. No in-process cache is needed, so a fresh instance
can recover solely from the durable store.

Every public operation reconstructs state by replaying the event chain. This
keeps each durable value bounded and makes the algorithm straightforward, at the
cost of read work growing linearly with history. The stated workload of hundreds
of bundles fits the store-call budget; the design does not compact old history
and is not intended for an unbounded event count. It also assumes the store's
specified durability, CAS, and per-record size guarantees.
