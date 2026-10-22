# Certified change bundle service

`Service` stores one logical database behind a single CAS root pointer. The root
contains a content hash for an immutable 16-way trie. Trie leaves hold individual
branch heads, bundle records, and publish request outcomes. Nodes are addressed by
the SHA-256 of their canonical JSON encoding and are written with CAS at version
zero. A transaction reads one root version, builds any changed trie paths as
immutable nodes, then CASes the root to the new hash. A loser retries from the
new root. This gives a compact root record while preserving separate records for
an unbounded history; it also avoids a multi-key transaction requirement.

The successful root CAS is the linearization point for every changing operation.
For publish, that same root CAS exposes all advanced branch heads, the published
flag, and the request outcome together. Thus a crash after node writes but before
the root CAS leaves unreachable immutable nodes, and a crash after the root CAS
leaves the full result durable even if the caller never receives its response.
Retries reread the durable root. Reads first capture one root, then traverse only
immutable nodes reachable from it, so `read_heads` returns a consistent snapshot.
No-op rejects and already recorded publish retries linearize at the root snapshot
read; durable publish rejections are committed as outcome leaves before returning.

Bundle records retain the original submission digest separately from the current
digest so a retried submit returns its original response after revisions. Revision,
approval, CI, readiness, parent checks, and rejection order follow the contract.
Approvals are stored as sorted unique authorized reviewers. A revision resets both
approval and CI evidence. CI digest covers the full update map. Branches absent from
the trie use the constructor's identical initial head map.

## P model

`ChangeBundle.p` abstracts the transaction boundary as a single atomic store event.
Two publisher machines race to publish ready bundles based on the same parent; one
advances the head and the other conflicts. One publisher retries the same request
ID, exercising the durable outcome memo. `BundleInvariant` checks that every
published transition had the current parent and that at most one bundle advances
from that parent. Checked with `snappea check ChangeBundle.pproj -n 1000`; all
schedules passed. The public Python suite also passed with
`python -m unittest -v test_public`.

The P model abstracts away trie writes, CAS retries, crash points, CI/approval
mutation races, and arbitrary branch maps. It checks the serialization rules at
the transaction boundary; it does not establish the Python implementation's
correctness or storage size/call bounds. The Python implementation relies on
content-addressed immutable nodes and the root CAS for those details.
