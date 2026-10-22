# Certified change bundle service

`solution.py` stores the service as an immutable event chain. The single key
`change-bundles:root` contains only the key of the newest event. Each event points
to its predecessor and contains one operation. The initial event holds the branch
heads. Replaying the chain from oldest to newest reconstructs bundles, approvals,
CI results, publish request outcomes, and current heads. The reconstructed state is
transient; separate `Service` instances share no cache.

An operation that changes state first reads the root and reconstructs that snapshot.
It writes an immutable event under a key derived from the event's canonical JSON,
then compare-and-swaps the root to that event. The successful root CAS is the
operation's linearization point. In particular, a successful publish event changes
all of its target heads and marks its bundle published during replay of one root
version. A failed root CAS means another operation won, so the caller reloads and
re-evaluates its rule against the new state. Read-only results linearize at the root
read that selects their snapshot.

The event key is deterministic. If a process dies after writing the event but
before publishing its key in the root, the event is an invisible orphan. A retry
can reuse that same event record and attempt the root CAS again. If the process dies
after the root CAS committed, a retry sees the durable state. Publish outcomes are
ordinary events keyed by request ID in the reconstructed state, so rejected outcomes
are just as durable and idempotent as successful ones. Submit responses are also
retained in the log so a later retry returns the original revision-one response.

One root CAS serializes competing state changes. This keeps the protocol small and
makes conflict evaluation, bundle publication, and outcome recording share one
visibility point. The tradeoff is that every operation replays the full event chain,
and the chain is never compacted. This is suitable for the specified history and
call limits, but costs grow with history and storage grows without bound. Events are
kept individually small; unusually large input values that exceed the store's
16,384-byte record limit cannot be written by this layout.

`ChangeBundles.p` models two concurrent publishers racing to advance the same pair
of branch heads. `AtomicBundleHeads` permits the initial pair or either complete
published pair and rejects a mixed pair. The store's publish handler makes the pair
visible in one P transition, representing the service's successful shared-root CAS.
It is an abstraction of the linearization point, rather than a model of durable
record encoding or every API validation rule.

Validation performed:

- `python -m unittest -v test_public` — all four public tests passed.
- `/opt/snappea check ChangeBundles.p tcAtomicBundles --exhaustive` — all 35
  schedules passed.
