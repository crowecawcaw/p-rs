# Certified change bundles design

## P/Snappea model

The model has one `BundleService` machine and durable `Bundle` records. Its abstract state is a branch-head map, a bundle map, submit-response memo table, and publish-outcome memo table. Every request is handled atomically by the service machine. A successful publish checks the revision, publication state, two distinct authorized approvals, passing CI for the current digest, and every expected parent before it changes all requested heads and marks the bundle published. A failed publish records its request outcome without changing heads. Submit IDs and publish IDs occupy separate memo tables.

A P-style model of the key transitions is:

```p
machine BundleService {
  var heads: map[string, string];
  var bundles: map[string, Bundle];
  var submits: map[string, SubmitResponse];
  var publishes: map[string, PublishResult];

  on Submit(id, updates) {
    if (!(id in submits)) {
      bundles[id] = NewBundle(updates); // revision 1, no approvals, ci false
      submits[id] = Response(id, 1, Digest(updates));
    }
    return submits[id];
  }

  on Revise(id, rev, updates) {
    b = bundles[id];
    if (rev != b.revision) return stale;
    if (b.published) return published;
    b.revision++; b.updates = updates; b.digest = Digest(updates);
    b.approvals = {}; b.ci = false;
    return updated;
  }

  on Publish(id, rev, request) {
    if (request in publishes) return publishes[request];
    b = bundles[id];
    if (rev != b.revision) result = stale;
    else if (b.published) result = published;
    else if (Count(b.approvals) < 2 || !b.ci) result = not_ready;
    else if (AnyParentMismatch(heads, b.updates)) result = conflict;
    else {
      foreach (branch in Keys(b.updates))
        heads[branch] = b.updates[branch].new;
      b.published = true;
      result = published;
    }
    publishes[request] = result;
    return result;
  }
}
```

The Python implementation refines this atomic machine with a durable append-only event chain. A single root key contains the chain tail and is the only mutable coordination point. Each event node is immutable and content-addressed by its canonical JSON payload (including its parent pointer). Writers first ensure that node exists and then CAS the root from the observed store version to the new tail. A failed root CAS means another operation won; the caller reloads and reevaluates its transition.

## Linearization and recovery

The successful CAS of the root key is the linearization point for each mutating operation. In particular, a publish event includes its memoized outcome, and successful publication is interpreted from that event in the same chain step, so readers see all branch changes and the memo together or neither. A reader captures the root once and traverses only immutable ancestors, giving it one consistent snapshot even if later operations commit while it reads.

A crash before the root CAS can leave an unreachable immutable node. It has no effect and can be reused if the same event is retried. A crash after the root CAS leaves the full event committed; a fresh service instance reconstructs its state from the root chain. Submit and publish retries first replay their memo tables, preserving their original results. Root CAS retries always reload state, so concurrent revisions, approvals, CI results, and publishes are evaluated in commit order.

## Limits

This design favors a small, auditable atomic point over read efficiency. Replaying the chain costs one store read per event for each operation, so latency grows with history, though each call remains within the stated store-call bound for the tested hundreds of bundles. Event values are subject to the store's 16 KiB value limit. The model assumes each request event (including its update map) fits that limit; larger request payloads would need chunked immutable event storage. Content-addressing uses SHA-256 with a payload comparison and deterministic collision suffix, so an actual digest collision does not cause an incorrect overwrite.
