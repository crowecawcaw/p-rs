# Certified change bundles design

## Durable representation and atomicity

`solution.py` stores a single small root pointer and a content-addressed persistent
16-way trie. Trie paths are the 64 hexadecimal nibbles of SHA-256 over each logical
key. Leaves hold the original key and JSON value, so path collisions remain
unambiguous. Internal nodes and leaves are immutable; their store keys are their
content hashes. A failed write to an existing node key is checked against the
stored value. Orphan nodes from a crash are harmless and may be reused.

The trie contains initial branch heads, bundle records, and publish request
outcomes in separate logical key namespaces. Every mutation reads a root version,
builds a new trie root, and CASes the shared root key. A CAS failure discards that
candidate root and recomputes from the latest state. This serializes commits while
allowing all read and preparation work to overlap. In particular, publish validates
all expected parents against the same snapshot that supplies the revision, readiness
state, and idempotency memo. It then writes every target head, the published flag,
and the request outcome into one candidate root. The successful root CAS is the
linearization point and their single atomic visibility point.

Submit linearizes at its root CAS; if the bundle key is already present, it returns
the original digest and revision. Revision, a new approval, and a changed CI result
linearize at their root CAS. A stale/invalid/rejected/no-op operation linearizes at
the root snapshot whose state caused its result. Read operations capture one root
pointer and read only nodes reachable from it, so a multi-branch head read is one
consistent snapshot. Returned mutable data is copied.

## Recovery

A crash before the root CAS leaves the previous state authoritative; any nodes
written for the abandoned candidate are invisible. A crash after the CAS leaves the
complete operation durable even if its caller did not receive a response. Replayed
submit calls find the existing bundle. Publish outcomes, including rejections, live
in the same trie as state and are keyed by publish request ID, so a retry returns its
recorded result before evaluating current state. Fresh Service instances need only
the shared store and the same initial heads.

## P model

`BundleModel.p` abstracts a ready bundle with two expected branch parents, two
concurrent publisher clients, and the shared root version/CAS protocol. The model
checks schedules where both clients read the same version, one commits both heads
and its outcome atomically, and the other loses the CAS, rereads, and observes the
published result. Assertions require parent validation before the commit transition
and require both target heads on the announced publish transition. The monitor test
passed with `/opt/snappea check BundleModel.p TwoPublishers.p --time 10s -n 100`.

The abstraction assumes the bundle is already ready, omits the Python trie encoding,
crashes and orphan nodes, and represents head validation and the root CAS as one
server transition. It checks the concurrency argument for publication; it does not
prove Python correctness, trie integrity, storage bounds, or recovery behavior.

## Limits

The root is a deliberate serialization point. Reads and writes cost up to 64 trie
node operations per key, and concurrent CAS retries redo path work. The design favors
simple atomic semantics over throughput. Values still obey the store's 16,384-byte
limit; a single oversized bundle record cannot be represented by this layout.
