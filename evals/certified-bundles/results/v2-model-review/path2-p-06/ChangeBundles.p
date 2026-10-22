// Abstracts the shared commit pointer: each request either observes a conflict
// or makes the whole branch vector visible at one transition.
event ePublish: (left: string, right: string);
event eHeads: (left: string, right: string);

type tPublish = (left: string, right: string);

spec AtomicBundleHeads observes eHeads {
  start state Consistent {
    on eHeads do (p: (left: string, right: string)) {
      assert (p.left == "s0" && p.right == "t0") ||
             (p.left == "s1" && p.right == "t1") ||
             (p.left == "s2" && p.right == "t2"),
             "a bundle exposed a partially advanced branch vector";
    }
  }
}

machine BundleStore {
  var left: string;
  var right: string;

  start state Init {
    entry {
      left = "s0";
      right = "t0";
      goto Serving;
    }
  }

  state Serving {
    on ePublish do (p: tPublish) {
      // The expected bases are s0/t0. The two assignments represent one
      // successful root-pointer CAS and therefore one visible state change.
      if (left == "s0" && right == "t0") {
        left = p.left;
        right = p.right;
      }
      announce eHeads, (left = left, right = right);
    }
  }
}

machine Publisher {
  var store: BundleStore;
  var left: string;
  var right: string;

  start state Init {
    entry (input: (store: BundleStore, left: string, right: string)) {
      store = input.store;
      left = input.left;
      right = input.right;
      send store, ePublish, (left = left, right = right);
      goto Done;
    }
  }
  state Done {}
}

machine Driver {
  start state Init {
    entry {
      var store: BundleStore;
      store = new BundleStore();
      new Publisher((store = store, left = "s1", right = "t1"));
      new Publisher((store = store, left = "s2", right = "t2"));
    }
  }
}

test tcAtomicBundles [main=Driver]:
  assert AtomicBundleHeads in { BundleStore, Publisher, Driver };
