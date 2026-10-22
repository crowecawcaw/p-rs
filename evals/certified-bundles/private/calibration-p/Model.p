// Harness calibration only. NOT supplied to either experimental arm.
// Two certified candidates share parent 0. At most one may land.
event eValidate: (client: Publisher);
event eValidated: bool;
event eSplitCommit;
event eAtomicCommit;
event eLanded;

spec OneChildPerBase observes eLanded {
  var landed: int;
  start state Watching {
    on eLanded do {
      landed = landed + 1;
      assert landed <= 1, "two candidates landed against the same parent";
    }
  }
}

machine Branch {
  var head: int;
  start state Serving {
    on eValidate do (req: (client: Publisher)) {
      send req.client, eValidated, head == 0;
    }
    on eSplitCommit do {
      head = head + 1;
      announce eLanded;
    }
    on eAtomicCommit do {
      if (head == 0) {
        head = head + 1;
        announce eLanded;
      }
    }
  }
}

machine Publisher {
  var branch: Branch;
  start state Init {
    entry (input: (branch: Branch, split: bool)) {
      branch = input.branch;
      if (input.split) {
        send branch, eValidate, (client=this,);
        goto Waiting;
      }
      send branch, eAtomicCommit;
      goto Done;
    }
  }
  state Waiting {
    on eValidated do (ok: bool) {
      if (ok) { send branch, eSplitCommit; }
      goto Done;
    }
  }
  state Done {}
}

fun Setup(split: bool) {
  var b: Branch;
  b = new Branch();
  new Publisher((branch=b,split=split));
  new Publisher((branch=b,split=split));
}
machine Broken { start state Init { entry { Setup(true); } } }
machine Fixed { start state Init { entry { Setup(false); } } }
test tcBroken [main=Broken]: assert OneChildPerBase in { Branch, Publisher, Broken };
test tcFixed [main=Fixed]: assert OneChildPerBase in { Branch, Publisher, Fixed };
