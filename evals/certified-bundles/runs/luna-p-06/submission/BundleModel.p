event ePublished: (left: string, right: string);
event eRead: (client: Publisher);
event eSnapshot: (version: int, left: string, right: string, done: bool, outcome: string);
event eCas: (client: Publisher, base: int, request: string);
event eCasResp: (ok: bool, outcome: string);

type tSnapshot = (version: int, left: string, right: string, done: bool, outcome: string);

machine StoreModel {
  var version: int;
  var left: string;
  var right: string;
  var done: bool;
  var outcome: string;
  var readers: map[Publisher, tSnapshot];

  start state Init {
    entry {
      version = 0;
      left = "l0";
      right = "r0";
      done = false;
      outcome = "";
      goto Serving;
    }
  }

  state Serving {
    on eRead do (p: (client: Publisher)) {
      send p.client, eSnapshot, (version = version, left = left, right = right,
        done = done, outcome = outcome);
    }
    on eCas do (p: (client: Publisher, base: int, request: string)) {
      if (p.base != version) {
        send p.client, eCasResp, (ok = false, outcome = "");
      } else if (done) {
        send p.client, eCasResp, (ok = true, outcome = outcome);
      } else {
        // The model bundle needs two approvals and passing CI. Both expected
        // parents are checked before this single root CAS transition.
        assert left == "l0" && right == "r0", "candidate must validate every parent";
        left = "l1";
        right = "r1";
        done = true;
        outcome = "published";
        version = version + 1;
        assert done && left == "l1" && right == "r1",
          "publish outcome and all target heads share one visibility point";
        announce ePublished, (left = left, right = right);
        send p.client, eCasResp, (ok = true, outcome = outcome);
      }
    }
  }
}

machine Publisher {
  var store: StoreModel;
  var base: int;
  var left: string;
  var right: string;
  var done: bool;
  var prior: string;
  var request: string;

  start state Init {
    entry (input: (store: StoreModel, request: string)) {
      store = input.store;
      request = input.request;
      send store, eRead, (client = this,);
      goto Read;
    }
  }
  state Read {
    on eSnapshot do (s: tSnapshot) {
      base = s.version;
      left = s.left;
      right = s.right;
      done = s.done;
      prior = s.outcome;
      if (done) {
        goto Finished;
      }
      send store, eCas, (client = this, base = base, request = request);
      goto Commit;
    }
  }
  state Commit {
    on eCasResp do (r: (ok: bool, outcome: string)) {
      if (r.ok) {
        goto Finished;
      }
      send store, eRead, (client = this,);
      goto Read;
    }
  }
  state Finished {}
}

spec AtomicBundle observes ePublished {
  start state Ready {
    on ePublished do (p: (left: string, right: string)) {
      assert p.left == "l1" && p.right == "r1",
        "a publish must expose both heads together";
    }
  }
}

test tcTwoPublishers [main=TwoPublishers]:
  assert AtomicBundle in { StoreModel, Publisher, TwoPublishers };
