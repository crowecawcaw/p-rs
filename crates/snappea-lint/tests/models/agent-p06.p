event ePublish: (client: Publisher, bundle: int);
event eResult: (bundle: int, result: int);

machine Coordinator {
  var headA: int;
  var headB: int;
  var ready: map[int, bool];
  var published: map[int, bool];
  var outcomes: map[int, int];
  var winnerCount: int;

  start state Init {
    entry {
      headA = 0;
      headB = 0;
      ready[1] = true;
      ready[2] = true;
      published[1] = false;
      published[2] = false;
      winnerCount = 0;
      new Publisher((coordinator = this, bundle = 1));
      new Publisher((coordinator = this, bundle = 2));
      goto Serving;
    }
  }

  state Serving {
    on ePublish do (req: (client: Publisher, bundle: int)) {
      var result: int;
      if (req.bundle in outcomes) {
        result = outcomes[req.bundle];
      } else if (published[req.bundle]) {
        result = 1;
        outcomes[req.bundle] = result;
      } else if (!(ready[req.bundle])) {
        result = 2;
        outcomes[req.bundle] = result;
      } else if (headA != 0 || headB != 0) {
        result = 3;
        outcomes[req.bundle] = result;
      } else {
        // The head pair and published marker change in one atomic transition.
        headA = req.bundle;
        headB = req.bundle;
        published[req.bundle] = true;
        winnerCount = winnerCount + 1;
        result = 1;
        outcomes[req.bundle] = result;
      }
      assert (headA == 0 && headB == 0) || headA == headB,
        "a multi-branch publish became partially visible";
      assert winnerCount <= 1, "two bundles published from the same parents";
      send req.client, eResult, (bundle = req.bundle, result = result);
    }
  }
}

machine Publisher {
  var coordinator: Coordinator;
  var bundle: int;
  start state Init {
    entry (input: (coordinator: Coordinator, bundle: int)) {
      coordinator = input.coordinator;
      bundle = input.bundle;
      send coordinator, ePublish, (client = this, bundle = bundle);
      goto Waiting;
    }
  }
  state Waiting {
    on eResult do (result: (bundle: int, result: int)) {
      assert result.bundle == bundle, "response delivered to wrong caller";
      goto Done;
    }
  }
  state Done {}
}
spec BundleTrace observes ePublish, eResult {
  start state Active {
    on ePublish goto Active;
    on eResult goto Active;
  }
}

test tcAtomicPublish [main=Coordinator]:
  assert BundleTrace in { Coordinator, Publisher };

event eCommitCheck: (branches: int, changed: int);
spec AtomicCommitCheck observes eCommitCheck {
  start state Whole {
    on eCommitCheck do (p: (branches: int, changed: int)) {
      assert p.branches == p.changed, "publish exposed a partial head update";
    }
  }
}
