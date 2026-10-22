# Certified change bundles

`Service` keeps the authoritative state in a persistent, content-addressed treap. The
store's versioned `certified-bundles/root-v1` record points to the current treap root.
Treap entries hold branch heads, current bundle records, submit-response memos, and
publish-request outcomes. Bundle values include revision, updates, digest, approvals,
CI result, and published state. Large values are stored as immutable linked chunks so
one store record remains below the 16 KiB limit. A node or chunk is written once under
its content hash; retries verify an existing record instead of changing it.

Every mutation first reads one root version and computes its decision from that
snapshot. It writes any new immutable treap nodes, then uses one CAS on the root record
to expose the new tree. On a failed CAS, it discards that candidate and reevaluates
against the newer root. This serializes writers but makes the root CAS the single
linearization point for each state change. In particular, successful publication
updates all target heads, the bundle's published flag, and the publish-request memo in
the same new tree. A reader captures one root and traverses only immutable nodes below
it, so `read_heads` sees a consistent snapshot.

The decision order in `publish` follows the contract: remembered request result,
revision check, already-published check, readiness, expected-head check, then commit.
Rejected results are written into the same map under the publish request ID, making
retries stable. Submit IDs and publish IDs use separate tagged keys. Submit also stores
the original response, so revising a bundle cannot change a retried submit response.
Approvals are a sorted set in the public status view; revisions clear approvals and CI.

A crash before the root CAS can leave unreachable immutable nodes or chunks, but they
are invisible and a retry can reuse them. A crash after the root CAS leaves the whole
transaction durable; the next instance reads it even if the original caller did not
receive a response. CAS contention causes a retry from the current root. Under
quiescence, a writer's eventual successful CAS completes its operation. There is no
garbage collection for unreachable immutable records, and the single root limits write
throughput; neither affects correctness.

`BundleModel.pproj` models two competing publishers that each update both branches
from the same base. The coordinator's publish handler abstracts the root CAS as one
atomic transition, while `AtomicHeads` checks that every visible successful transition
keeps both heads equal. Snappea exhaustive exploration checked all 53 schedules for
`tcCompetingPublishes` with no invariant violation. The model captures the commit
boundary and all-or-nothing head update; it abstracts persistent-node construction,
revision/approval/CI checks, and crashes before or after the root CAS. Those failure
cases are handled by the immutable-write/root-CAS recovery protocol described above.
