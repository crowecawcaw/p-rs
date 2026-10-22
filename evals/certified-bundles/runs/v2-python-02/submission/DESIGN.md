# Certified change bundle service design

## State layout

The service stores one CAS-protected root manifest. It points to an immutable
branch-head map and 128 immutable data shards. Bundle records, submit responses,
and publish outcomes are placed in shards by a stable hash of their logical key;
their key prefixes keep the namespaces separate. Immutable values are addressed
by the SHA-256 of canonical JSON. Shard references keep the root manifest small
as the number of bundles grows, and unchanged shards are reused across commits.

The constructor only copies the supplied initial heads. The first operation
lazily writes the initial head map and installs the root with CAS. If multiple
instances do this together, they can reuse the same content-addressed head value,
and only one root CAS wins; losing instances read the installed root.

## Atomicity and linearization

A mutating operation reads one root version, reads and updates its affected
immutable values, writes new content-addressed values, then CASes the root
manifest. A successful root CAS is the operation's linearization point. If the
CAS fails, another operation won, so the caller rereads current state and
reevaluates its rule. This makes revisions, approvals, CI changes, outcomes, and
head changes linearizable across instances.

Publishing stores its request outcome in the same proposed manifest as any bundle
published marker and all changed branch heads. A successful publish therefore has
a single atomic visibility point. Rejected publish outcomes are stored in the
manifest too. A repeated request ID reads that outcome before evaluating current
state. Submit similarly stores its bundle and original response together, making
retries return the original response even after later revisions.

`read_heads` and `get_status` first pin a manifest by reading the root pointer,
then read immutable values reachable from it. Their results therefore come from
one consistent snapshot. Returned structures are copied so callers cannot mutate
service state.

## Failure recovery and limits

Immutable values are written before the root CAS. A crash before the CAS leaves
unreachable values that do not affect visible state. A crash after the CAS leaves
a complete state because each referenced value was already written. If a content
write response is lost, a later operation can reuse the same immutable value; if
a root write response is lost, the durable root shows the committed state. No
in-process cache is used, so restarted instances recover from the shared store.

This design serializes commits through one root pointer. It favors simple atomic
semantics over throughput. It assumes each logical value, especially the branch
head map and an individual bundle update map, fits the store's 16 KiB record
limit. It also assumes no SHA-256 collision.
