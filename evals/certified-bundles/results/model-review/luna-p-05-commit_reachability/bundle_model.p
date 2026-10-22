event eStage: (client: Publisher, request: int, seen: int);
event eCommit: (client: Publisher, request: int, seen: int);
event eDone: (request: int, outcome: bool);
event eSnapshot: (schema: int, service: int, pub0: bool, pub1: bool, memo0: bool, memo1: bool);

spec AtomicPublication observes eSnapshot {
  start state Check {
    on eSnapshot do (p: (schema: int, service: int, pub0: bool, pub1: bool, memo0: bool, memo1: bool)) {
      assert p.schema == p.service, "multi-branch update was partially visible";
      assert !p.pub0 || p.memo0, "published state has no durable request memo";
      assert !p.pub1 || p.memo1, "published state has no durable request memo";
    }
  }
}

machine BundleStore {
  var generation: int;
  var headSchema: int;
  var headService: int;
  var published: map[int, bool];
  var memo: map[int, bool];
  var staged: map[int, bool];

  start state Init {
    entry {
      generation = 0;
      headSchema = 0;
      headService = 0;
      published[0] = false;
      published[1] = false;
      memo[0] = false;
      memo[1] = false;
      staged[0] = false;
      staged[1] = false;
      goto Serving;
    }
  }

  state Serving {
    on eStage do (r: (client: Publisher, request: int, seen: int)) {
      // Durable immutable event data can exist before the root pointer changes.
      // A crash here leaves an orphan that is never part of the visible chain.
      staged[r.request] = true;
      send r.client, eCommit, (client = r.client, request = r.request, seen = r.seen);
    }
    on eCommit do (r: (client: Publisher, request: int, seen: int)) {
      var result: bool;
      assert staged[r.request], "root CAS cannot reference an unwritten event";
      if (memo[r.request]) {
        result = published[r.request];
      } else {
        result = false;
        if (r.seen == generation && headSchema == 0 && headService == 0) {
          // This is the one atomic root CAS: both heads, published marker, memo,
          // and generation change at the same visibility point.
          headSchema = 1;
          headService = 1;
          published[r.request] = true;
          result = true;
          generation = generation + 1;
        }
        memo[r.request] = true;
      }
      assert headSchema == headService, "bundle heads became partially visible";
      assert published[r.request] == result || !result,
             "memoized response disagrees with committed publication";
      announce eSnapshot, (schema = headSchema, service = headService,
                           pub0 = published[0], pub1 = published[1],
                           memo0 = memo[0], memo1 = memo[1]);
      send r.client, eDone, (request = r.request, outcome = result);
    }
  }
}

machine Publisher {
  var store: BundleStore;
  var request: int;
  var seen: int;

  start state Init {
    entry (input: (store: BundleStore, request: int)) {
      store = input.store;
      request = input.request;
      // Both callers can read generation zero before either reaches the store CAS.
      seen = 0;
      send store, eStage, (client = this, request = request, seen = seen);
      goto Waiting;
    }
  }
  state Waiting {
    on eCommit do (r: (client: Publisher, request: int, seen: int)) {
      // Store messages are not returned to the publisher; the store's eDone is
      // observed only to make termination explicit.
      goto Done;
    }
  }
  state Done {}
}

machine CompetingPublishers {
  start state Init {
    entry {
      var store: BundleStore;
      store = new BundleStore();
      new Publisher((store = store, request = 0));
      new Publisher((store = store, request = 1));
    }
  }
}

test tcAtomicBundlePublish [main=CompetingPublishers]:
  assert AtomicPublication in { BundleStore, Publisher, CompetingPublishers };
