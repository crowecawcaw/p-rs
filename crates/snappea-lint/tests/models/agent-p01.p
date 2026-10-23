event eInit: (x: int, y: int);
event ePublish: (bundle: int, expectedX: int, expectedY: int, newX: int, newY: int);
event eResult: (request: int, bundle: int, result: int);
event eSnapshot: (x: int, y: int);

type tPublish = (client: Client, request: int, bundle: int, expectedX: int, expectedY: int, newX: int, newY: int);
event ePublishReq: tPublish;
event ePublishResp: (bundle: int, result: int);
event eReadReq: (client: Client);
event eReadResp: (x: int, y: int);

spec AtomicBundleCorrect observes eInit, ePublish, eResult, eSnapshot {
  var x: int;
  var y: int;
  var initialized: bool;
  var published: set[int];
  var results: map[int, int];
  start state Empty {
    on eInit do (p: (x: int, y: int)) {
      assert !initialized, "initialized twice";
      x = p.x; y = p.y; initialized = true;
      goto Ready;
    }
    on ePublish do { assert false, "publish before initialization"; }
    on eResult do { assert false, "result before initialization"; }
    on eSnapshot do { assert false, "snapshot before initialization"; }
  }
  state Ready {
    on eInit do { assert false, "initialized twice"; }
    on ePublish do (p: (bundle: int, expectedX: int, expectedY: int, newX: int, newY: int)) {
      assert p.bundle in published == false, "bundle published twice";
      assert x == p.expectedX && y == p.expectedY, "publish event has stale parent";
      x = p.newX; y = p.newY;
      published += (p.bundle);
      assert x == p.newX && y == p.newY, "partial bundle update";
    }
    on eResult do (p: (request: int, bundle: int, result: int)) {
      assert p.result != 1 || p.bundle in published, "published result lacks committed bundle";
      if (p.request in results) {
        assert results[p.request] == p.result, "request ID returned different outcomes";
      } else { results[p.request] = p.result; }
    }
    on eSnapshot do (p: (x: int, y: int)) {
      assert p.x == x && p.y == y, "snapshot differs from atomic head state";
    }
  }
}

machine Coordinator {
  var x: int;
  var y: int;
  var outcomes: map[int, int];
  start state Init {
    entry { x = 0; y = 0; announce eInit, (x = x, y = y); goto Serving; }
  }
  state Serving {
    on ePublishReq do (p: tPublish) {
      var result: int;
      if (p.request in outcomes) { result = outcomes[p.request]; }
      else if (x == p.expectedX && y == p.expectedY) {
        x = p.newX; y = p.newY; result = 1;
        announce ePublish, (bundle = p.bundle, expectedX = p.expectedX, expectedY = p.expectedY, newX = x, newY = y);
        outcomes[p.request] = result;
      } else { result = 2; outcomes[p.request] = result; }
      announce eResult, (request = p.request, bundle = p.bundle, result = result);
      send p.client, ePublishResp, (bundle = p.bundle, result = result);
    }
    on eReadReq do (p: (client: Client)) {
      announce eSnapshot, (x = x, y = y);
      send p.client, eReadResp, (x = x, y = y);
    }
  }
}

machine Client {
  var coordinator: Coordinator;
  var id: int;
  var result: int;
  start state Init {
    entry (p: (coordinator: Coordinator, id: int)) {
      coordinator = p.coordinator; id = p.id;
      if (id == 1) {
        send coordinator, ePublishReq, (client = this, request = id, bundle = id, expectedX = 0, expectedY = 0, newX = 1, newY = 1);
      } else {
        send coordinator, ePublishReq, (client = this, request = id, bundle = id, expectedX = 0, expectedY = 0, newX = 2, newY = 2);
      }
      goto Waiting;
    }
  }
  state Waiting {
    on ePublishResp do (p: (bundle: int, result: int)) {
      assert p.bundle == id, "response ID mismatch";
      result = p.result;
      send coordinator, eReadReq, (client = this,);
      goto Reading;
    }
  }
  state Reading {
    on eReadResp do (p: (x: int, y: int)) {
      assert p.x == p.y, "reader observed half a bundle";
      goto Done;
    }
  }
  state Done {}
}

machine Main {
  start state Init {
    entry {
      var c: Coordinator;
      c = new Coordinator();
      new Client((coordinator = c, id = 1));
      new Client((coordinator = c, id = 2));
    }
  }
}
