# Certified change bundles

`CertifiedBundles.p` is the Snappea model for the atomic branch-head check at publish time. It starts two already-certified bundles from the same base and explores both delivery orders. `BundleService` checks the current head and advances it in one handler transition; `PublishSafety` asserts that only one competing bundle can advance from `s0`. The model intentionally abstracts away approvals, CI and revisions: those determine readiness, while the property being checked here is the atomic compare-and-advance across competing publications. The Python service enforces the complete contract.

## Durable representation

The store has one mutable key, `ccb:root`, whose value names the current immutable event node. A deterministic immutable base node contains the initial heads. Every successful state change is represented by an immutable node containing its parent node and one event: submit, revise, approve, CI result, or publish outcome. The event chain is replayed from the base to reconstruct bundle status, branch heads and publish-request outcomes. Large event payloads are split into immutable content-addressed chunks so that each stored value stays below the store's 16 KiB limit.

All writers use the same sequence:

1. Read the root and replay the immutable chain to get one candidate state.
2. Decide the operation's result against that state.
3. Persist its immutable event node and any chunks.
4. Compare-and-swap the root from the version read in step 1 to the new node.

A failed root CAS means another operation committed first, so the writer reloads and reevaluates. A node or chunk left behind by a crash is unreachable and harmless. A crash after the root CAS is recovered by replaying that now-reachable event. Deterministic content hashes make retries reuse immutable records; the root CAS is the only visibility gate.

The root CAS is the linearization point for every state change. In particular, a successful publish event carries the request outcome and all new branch heads in the same event, so readers cannot observe a memoized result without the corresponding heads or vice versa. Rejected publish results are also events and therefore stay idempotent by publish request ID. Submit IDs live in the bundle namespace and retain the original submission digest through later revisions.

Read-only answers and operations that need no state change (such as a duplicate approval) linearize at the root read that supplied their snapshot. `read_heads` reads one root and replays only that root's immutable ancestry, giving a consistent multi-branch snapshot. Each service instance keeps only its constructor inputs and does no I/O until the first method call; no process-local cache participates in correctness.

## Tradeoffs

The append-only history can grow without bound, so reads and writes replay more records over time. This favors a simple atomic protocol over throughput and avoids a multi-key transaction. The layout keeps history out of any one store value and chunks unusually large event payloads. A P check covers the competing publication invariant; durability, retries and crash recovery are justified by the immutable-node/root-CAS argument and exercised by the Python runtime tests.
