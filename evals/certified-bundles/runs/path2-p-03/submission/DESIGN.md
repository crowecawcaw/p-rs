# Certified change bundle service

## Representation

The durable state is an immutable, content-addressed, path-compressed radix trie. It maps namespaced logical keys (`H:` branch heads, `B:` bundles, `S:` submit responses, and `P:` publish outcomes) to JSON values. Trie nodes live under hash-derived store keys. A single `ccb:root` record points to the trie root. The root's CAS version is the global commit sequence.

A writer reads the root version and value, makes its decision from that immutable snapshot, stores any new trie nodes, then CASes `ccb:root` from the version it read to the new root. If that CAS loses, it discards the candidate root and reevaluates against the winner. Thus speculative nodes are harmless unreachable data. A crash before root CAS leaves no visible effect; a crash after the CAS leaves the complete operation visible. Initialization uses the same root CAS and is safe across constructors.

The root CAS is the linearization point for every state-changing operation. It commits bundle changes, head changes, and publish request memos together. Reads first capture the root pointer and traverse only immutable nodes reachable from that pointer, so `read_heads` sees one consistent snapshot. Reads and state-dependent no-op/rejected operations linearize at their captured root read. Submit response keys preserve the original revision-1 response through later revisions. Publish outcome keys memoize every outcome by request ID, and are checked before the current bundle state.

## Semantics

Bundle state stores revision, the complete update map and its digest, sorted approvals, current CI result, and publication status. Revision replaces the update map and resets approvals and CI. Approval and CI updates are conditional on the current revision; CI also checks the digest. Publish evaluates the specified order, then either memoizes a rejection or places all new branch heads, the published flag, and the successful request memo in one root update. A later publish with another request ID sees the published flag and records `published` without advancing heads again.

The only shared synchronization is the store's compare-and-swap root record. No process-local cache is retained between calls. Per-call trie node caches only avoid duplicate reads while following the same immutable root. JSON values are copied at API boundaries and when inserted into the durable tree.

## P/Snappea model

`PSrc/Service.p` models two publishers racing to advance one shared branch from `s0`. The coordinator handles a publish attempt atomically: the first changes the head and marks its bundle, while the other sees a parent conflict. The `ChangeBundles` spec checks that each commit used the current parent and that a bundle is never applied twice. `tcAtomicPublish` explores all schedules for this race. This model focuses on the transaction boundary and competing parents; the Python implementation carries the full revision, approval, CI, and request-memo rules.

## Limits

The trie relies on SHA-256 collision resistance for content addressing and logical-key paths. Each individual JSON value still must fit the store's 16,384-byte limit; in particular, an update map must fit in its bundle value. The implementation makes no throughput claim: root CAS intentionally serializes commits to provide a single durable atomic visibility point. Orphaned immutable nodes from losing attempts are safe but are not garbage-collected.
