# Certified change bundles design

## Storage layout

The store has one mutable key, `certified-bundles:root`. Its value points to the
root of an immutable, persistent hash trie. Trie keys are namespaced strings for
branch heads (`h:`), bundle records (`b:`), and publish request outcomes (`p:`).
The trie follows the first 16 hexadecimal digits of SHA-256(key), then stores a
small sorted collision bucket. Each immutable node is addressed by the SHA-256
hash of its canonical JSON encoding. Nodes are written with compare-and-swap
against version zero before any pointer can reference them. A losing writer can
reuse the same content-addressed node.

The mutable root key is the serialization point for every logical change. A
writer reads a root version, reads needed records from that immutable snapshot,
creates replacement trie nodes, and CASes the root to the new trie. On CAS
failure it recomputes from the latest root. Thus updates across any number of
branches plus bundle state and publish memo are visible together. The trie keeps
individual records below the store's per-value limit and allows the total history
to grow beyond one record.

## Initialization and recovery

Initialization is lazy. The first operation builds trie entries for supplied
initial heads and CASes them into an absent root. Concurrent initializers build
the same logical state; only one root CAS succeeds. Partial trie writes from a
crash are unreachable immutable records and harmless. The next operation can
initialize again. Likewise, a crash after writing new trie nodes but before the
root CAS leaves harmless unreachable records. A crash after a successful root
CAS has committed the operation; a retry reads durable state and observes its
result. No local cache is used, so any service instance can continue after a
restart.

A lost response after a root CAS is handled by operation-level idempotency:
submit finds the existing bundle and returns its original revision-1 digest;
publish finds its durable request memo before evaluating current state. Revision,
approval, and CI calls can be safely retried according to their specified
current-state semantics.

## Linearization points

A successful transaction linearizes at its root CAS. Rejected operations that
write a publish request memo linearize at that same CAS. Operations that return
without writing linearize at their read of the root snapshot, since the result
is determined entirely from that immutable snapshot. `read_heads` and
`get_status` read one root pointer and use only immutable nodes reachable from
it, giving them a consistent snapshot.

Publish evaluates the required conditions from one snapshot. On success, all
head replacements, the published marker, and the request result memo are
included in one replacement trie and committed by one root CAS. A competing
change causes the CAS to fail and publish re-evaluates against the new state.
Different publish requests that race therefore cannot both apply the same
bundle or overwrite a branch without seeing each other's committed heads.

## Limits

This design favors a simple global serialization point over throughput. Hash
trie paths are fixed-depth, so individual lookups and updates touch a bounded
number of nodes; each new node is small. It assumes the supplied JSON records
fit the store's 16 KiB value limit and relies on SHA-256 collision resistance for
content addressing and trie path distribution. The contract provides no
multi-key transaction, so the root pointer is intentionally the sole commit
record.
