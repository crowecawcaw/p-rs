// Bounded P model for the commit rules: one bundle and one target branch.
// Successful state transitions are the observable commit-journal events.
event eApproval: (reviewer: int);
event eCI: (passed: bool);
event ePublish: (expected: int, proposed: int);

spec CertifiedBundleCorrect observes eApproval, eCI, ePublish {
  var head: int;
  var alice: bool;
  var bob: bool;
  var carol: bool;
  var ci: bool;
  var published: bool;

  start state Open {
    entry {
      head = 0;
      alice = false;
      bob = false;
      carol = false;
      ci = false;
      published = false;
    }
    on eApproval do (p: (reviewer: int)) {
      assert !published, "approval after publication";
      if (p.reviewer == 0) { alice = true; }
      else if (p.reviewer == 1) { bob = true; }
      else if (p.reviewer == 2) { carol = true; }
      else { assert false, "unauthorized reviewer recorded"; }
    }
    on eCI do (p: (passed: bool)) {
      assert !published, "CI after publication";
      ci = p.passed;
    }
    on ePublish do (p: (expected: int, proposed: int)) {
      assert !published, "bundle applied twice";
      assert (alice && bob) || (alice && carol) || (bob && carol),
        "published without two distinct approvals";
      assert ci, "published without passing CI";
      assert head == p.expected, "publish ignored parent conflict";
      head = p.proposed;
      published = true;
    }
  }
}

machine BundleCommit {
  start state Init {
    entry {
      announce eApproval, (reviewer = 0,);
      announce eApproval, (reviewer = 1,);
      announce eCI, (passed = true,);
      announce ePublish, (expected = 0, proposed = 1);
      goto Done;
    }
  }
  state Done {}
}

test tcCertifiedPublish [main=BundleCommit]:
  assert CertifiedBundleCorrect in { BundleCommit };
