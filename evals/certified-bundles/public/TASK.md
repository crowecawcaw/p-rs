# Certified change bundles

Implement `solution.py`, exporting `Service` with the async interface in
`contract.py`. Use Python's standard library. This is a coordination problem, not
file merging: commit IDs are opaque strings, and branch names identify repositories.

Several independent Service instances share one durable key/value store. They
publish bundles that update multiple branches atomically. Calls overlap, processes
crash, and responses can be lost. There is no shared process memory across clients.
Your implementation must be linearizable, durable, and make progress after failures
stop. Do not modify the supplied contract, runtime, or tests.

## State and rules

All branches and their initial heads are supplied at construction. Constructors do
no async work; initialize durable state lazily. Initial heads are identical across
instances and restarts. Branches are never added/deleted. An update map is nonempty:
`{'schema': {'expected': 's0', 'new': 's1'}, ...}`. Branch names are known; all
strings are nonempty. Inputs/outputs must not alias mutable durable state.

- `submit(request_id, updates)` creates bundle ID `request_id`, at revision 1.
  Return `{'bundle': request_id, 'revision': 1, 'digest': digest(updates)}`. The
  function `digest` is provided in contract.py. Retrying submit with the same ID
  and original arguments returns its original response, even after later revisions.
- `revise(bundle, expected_revision, updates)` returns `stale` if the expected
  revision differs from the current one; otherwise `published` if already published;
  otherwise replace updates, increment revision, clear approvals/CI, return `updated`.
- `approve(bundle, revision, reviewer)` accepts only reviewers `alice`, `bob`, `carol`.
  Return False for other reviewers, stale revision, or already published bundle.
  Otherwise record the reviewer in a set and return True (duplicates also True).
  Two distinct authorized approvals are required to publish.
- `record_ci(bundle, revision, candidate_digest, passed)` returns False for a stale
  revision, wrong digest, or published bundle. Otherwise replace the current CI result
  with `bool(passed)` and return True. An accepted failing result invalidates prior
  success. CI applies to the entire update map INCLUDING its expected parent IDs.
- `publish(bundle, revision, request_id)` evaluates, in this order:
  1. If this publish request ID has a recorded outcome, return that outcome.
  2. Revision differs from current: `stale`.
  3. Bundle already published: `published`, WITHOUT applying updates again.
  4. Fewer than two authorized distinct approvals or no passing CI: `not_ready`.
  5. Any current target head differs from its expected parent: `conflict`.
  6. Otherwise atomically advance ALL target heads, mark bundle published, return
     `published`.
  Every publish outcome, including rejections, is durable and idempotent by request
  ID. The result memo and any head updates must have ONE atomic visibility point.
  Retries of a publish request use the same bundle/revision. Different calls use
  fresh request IDs. Submit and publish IDs have separate namespaces.
- `read_heads(branches)` returns the requested heads from one consistent snapshot.
- `get_status(bundle)` returns exactly:
  `{'revision': int, 'updates': dict, 'digest': str, 'approvals': sorted_list,
  'ci': bool, 'published': bool}`. New/revised bundles have ci=False.

Calls use existing bundle IDs (submit completed beforehand). Stale revisions,
unauthorized reviewers, wrong CI digests, and branch conflicts ARE valid inputs.
You need not handle malformed types or missing bundles. Do not weaken the rules
by rejecting all operations. With no failures/contention, valid work must finish.

## Durable store: your only synchronization and persistence primitive

`await store.read(key)` returns `(version, value)`, absent keys `(0, None)`.
`await store.cas(key, expected_version, value)` atomically compares version and, on
success, increments it and stores a deep copy of the JSON value. Returns bool.
No deletion, no multi-key transaction. Keys are arbitrary strings. Values are
JSON-serializable, and each encoded value is at most 16,384 UTF-8 bytes (compact
JSON, ensure_ascii=True). Oversized writes raise ValueError. The entire history
will exceed one record. Store methods yield before and after their atomic step;
a caller can lose the response AFTER a write has durably committed.

You may choose any record layout, including immutable records and shared pointers.
The store is linearizable and durable. Use only its public read/cas methods: do not
inspect its attributes, use files/databases, locks, wall time, random IDs, globals,
threads, sleeps, background tasks, or other I/O in Service. Local transient state
is allowed; do not rely on caches remaining coherent between Service instances.
All concurrency must run through awaited store operations. Do not catch BaseException
or suppress cancellation/crash exceptions. On process death, in-flight calls disappear,
volatile state is discarded, and fresh Service instances use the same durable store.

Operations should finish within 5,000 store calls in quiescence. Tests use up to
64 branches and hundreds of bundles. No throughput requirement: a shared commit
pointer is allowed. Correctness is the focus, not scalability.

## Deliverables and feedback

Write solution.py and DESIGN.md explaining the algorithm, failure recovery,
linearization points, and limitations. Run `python -m unittest -v test_public`.
Public tests provide interface examples. Hidden tests exercise the stated semantics,
interleavings, crash/retry, growth, and progress. Hidden results are unavailable
until your submission is frozen. You are not required to predict a specific layout.
