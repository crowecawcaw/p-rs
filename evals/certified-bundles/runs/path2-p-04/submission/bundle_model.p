// Compact P/Snappea monitor for one two-branch bundle. Events represent
// completed service operations; ePublish is emitted at the atomic commit point.
event eCreated: (revision: int);
event eRevised: (revision: int);
event eApproved: (revision: int, reviewer: string);
event eCi: (revision: int, passed: bool);
event eHeadChanged: (branch: string, head: string);
event ePublish: (revision: int, result: string);

spec BundleProtocol observes eCreated, eRevised, eApproved, eCi,
                          eHeadChanged, ePublish {
  var revision: int;
  var alice: bool;
  var bob: bool;
  var carol: bool;
  var ci: bool;
  var published: bool;
  var schemaHead: string;
  var serviceHead: string;

  start state Empty {
    on eCreated do (p: (revision: int)) {
      assert p.revision == 1, "new bundle must begin at revision one";
      revision = p.revision;
      alice = false;
      bob = false;
      carol = false;
      ci = false;
      published = false;
      schemaHead = "s0";
      serviceHead = "v0";
      goto Active;
    }
    on eRevised do { assert false, "cannot revise a missing bundle"; }
    on eApproved do { assert false, "cannot approve a missing bundle"; }
    on eCi do { assert false, "cannot certify a missing bundle"; }
    on eHeadChanged do { assert false, "head change before bundle setup"; }
    on ePublish do { assert false, "cannot publish a missing bundle"; }
  }

  state Active {
    on eCreated do { assert false, "bundle created twice"; }
    on eRevised do (p: (revision: int)) {
      assert !published, "published bundle was revised";
      assert p.revision == revision + 1, "revision must advance by one";
      revision = p.revision;
      alice = false;
      bob = false;
      carol = false;
      ci = false;
    }
    on eApproved do (p: (revision: int, reviewer: string)) {
      assert !published, "published bundle accepted approval";
      assert p.revision == revision, "stale approval accepted";
      if (p.reviewer == "alice") { alice = true; }
      if (p.reviewer == "bob") { bob = true; }
      if (p.reviewer == "carol") { carol = true; }
    }
    on eCi do (p: (revision: int, passed: bool)) {
      assert !published, "published bundle accepted CI";
      assert p.revision == revision, "stale CI accepted";
      ci = p.passed;
    }
    on eHeadChanged do (p: (branch: string, head: string)) {
      if (p.branch == "schema") { schemaHead = p.head; }
      if (p.branch == "service") { serviceHead = p.head; }
    }
    on ePublish do (p: (revision: int, result: string)) {
      var ready: bool;
      var baseMatches: bool;
      if (p.revision != revision) {
        assert p.result == "stale", "wrong stale publish result";
      } else if (published) {
        assert p.result == "published", "published bundle result changed";
      } else {
        ready = (alice && bob) || (alice && carol) || (bob && carol);
        if (!ready || !ci) {
          assert p.result == "not_ready", "publish bypassed approvals or CI";
        } else {
          baseMatches = schemaHead == "s0" && serviceHead == "v0";
          if (!baseMatches) {
            assert p.result == "conflict", "publish ignored a changed branch head";
          } else {
            assert p.result == "published", "ready bundle was not published";
            schemaHead = "s1";
            serviceHead = "v1";
            published = true;
          }
        }
      }
    }
  }
}

// A small executable trace exercises the ready, atomic two-branch publication path.
machine BundleScenario {
  start state Init {
    entry {
      announce eCreated, (revision = 1,);
      goto Approving;
    }
  }
  state Approving {
    entry {
      announce eApproved, (revision = 1, reviewer = "alice");
      announce eApproved, (revision = 1, reviewer = "bob");
      announce eCi, (revision = 1, passed = true);
      goto Publishing;
    }
  }
  state Publishing {
    entry {
      announce ePublish, (revision = 1, result = "published");
      goto Done;
    }
  }
  state Done {}
}

test tcAtomicBundlePublish [main=BundleScenario]:
  assert BundleProtocol in { BundleScenario };
