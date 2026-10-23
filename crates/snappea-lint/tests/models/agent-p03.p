event eSubmitReq: (bundle: string, revision: int, digest: string);
event eSubmitDone: (bundle: string, revision: int, digest: string);
event eReviseReq: (bundle: string, oldRevision: int, revision: int);
event eReviseDone: (bundle: string, oldRevision: int, revision: int);
event eApproveReq: (bundle: string, revision: int, reviewer: string);
event eApproveDone: (bundle: string, revision: int, reviewer: string);
event eCIReq: (bundle: string, revision: int, digest: string, passed: bool);
event eCIDone: (bundle: string, revision: int, digest: string, passed: bool);
event ePublishDecisionReq: (bundle: string, revision: int, outcome: string, apply: bool);
event ePublishDecisionDone: (bundle: string, revision: int, outcome: string, apply: bool);
event eHead: (branch: string, oldHead: string, newHead: string);
event eRead: (branch: string, head: string);

spec BundleInvariant observes eSubmitDone, eReviseDone, eApproveDone, eCIDone, ePublishDecisionDone, eHead {
  var revision: int;
  var published: bool;
  var approvalCount: int;
  var seenAlice: bool;
  var seenBob: bool;
  var seenCarol: bool;
  var ci: bool;
  var pubCount: int;

  start state Empty {
    on eSubmitDone do (p: (bundle: string, revision: int, digest: string)) {
      assert p.revision == 1, "new bundle starts at revision one";
      revision = 1;
      published = false;
      approvalCount = 0;
      seenAlice = false;
      seenBob = false;
      seenCarol = false;
      ci = false;
      pubCount = 0;
      goto Active;
    }
    on eHead do { assert false, "head changed before bundle submission"; }
  }
  state Active {
    on eSubmitDone do { assert false, "bundle submitted twice"; }
    on eReviseDone do (p: (bundle: string, oldRevision: int, revision: int)) {
      assert !published, "published bundle revised";
      assert p.oldRevision == revision, "revision CAS used wrong base";
      assert p.revision == revision + 1, "revision did not increment once";
      revision = p.revision;
      approvalCount = 0;
      seenAlice = false;
      seenBob = false;
      seenCarol = false;
      ci = false;
    }
    on eApproveDone do (p: (bundle: string, revision: int, reviewer: string)) {
      assert !published && p.revision == revision, "stale approval accepted";
      if (p.reviewer == "alice" && !seenAlice) { seenAlice = true; approvalCount = approvalCount + 1; }
      if (p.reviewer == "bob" && !seenBob) { seenBob = true; approvalCount = approvalCount + 1; }
      if (p.reviewer == "carol" && !seenCarol) { seenCarol = true; approvalCount = approvalCount + 1; }
    }
    on eCIDone do (p: (bundle: string, revision: int, digest: string, passed: bool)) {
      assert !published && p.revision == revision, "stale CI accepted";
      ci = p.passed;
    }
    on ePublishDecisionDone do (p: (bundle: string, revision: int, outcome: string, apply: bool)) {
      assert p.revision == revision || p.outcome == "stale", "wrong revision outcome";
      if (p.apply) {
        assert !published && approvalCount >= 2 && ci, "published without evidence";
        published = true;
        pubCount = pubCount + 1;
      }
      assert pubCount <= 1, "bundle applied twice";
      if (p.outcome == "published" && !published) {
        assert false, "reported published before state changed";
      }
    }
    on eHead do (p: (branch: string, oldHead: string, newHead: string)) {
      assert published, "head changed outside successful publication";
    }
  }
}

machine BundleStore {
  var rev: int;
  var published: bool;
  var approvalCount: int;
  var seenAlice: bool;
  var seenBob: bool;
  var seenCarol: bool;
  var ci: bool;
  var heads: map[string, string];
  var expected: string;
  var proposed: string;
  var d: string;

  start state Init {
    entry {
      rev = 0;
      published = false;
      approvalCount = 0;
      seenAlice = false;
      seenBob = false;
      seenCarol = false;
      ci = false;
      heads["schema"] = "s0";
      expected = "s0";
      proposed = "s1";
      d = "digest-s0-s1";
      goto Ready;
    }
  }
  state Ready {
    on eSubmitReq do (p: (bundle: string, revision: int, digest: string)) {
      if (rev == 0) {
        rev = 1;
        d = p.digest;
        announce eSubmitDone, p;
      }
    }
    on eReviseReq do (p: (bundle: string, oldRevision: int, revision: int)) {
      if (!published && rev == p.oldRevision) {
        rev = p.revision;
        approvalCount = 0;
      seenAlice = false;
      seenBob = false;
      seenCarol = false;
        ci = false;
        announce eReviseDone, p;
      }
    }
    on eApproveReq do (p: (bundle: string, revision: int, reviewer: string)) {
      if (!published && p.revision == rev && (p.reviewer == "alice" || p.reviewer == "bob" || p.reviewer == "carol")) {
        if (p.reviewer == "alice" && !seenAlice) { seenAlice = true; approvalCount = approvalCount + 1; }
        if (p.reviewer == "bob" && !seenBob) { seenBob = true; approvalCount = approvalCount + 1; }
        if (p.reviewer == "carol" && !seenCarol) { seenCarol = true; approvalCount = approvalCount + 1; }
        announce eApproveDone, p;
      }
    }
    on eCIReq do (p: (bundle: string, revision: int, digest: string, passed: bool)) {
      if (!published && p.revision == rev && p.digest == d) {
        ci = p.passed;
        announce eCIDone, p;
      }
    }
    on ePublishDecisionReq do (p: (bundle: string, revision: int, outcome: string, apply: bool)) {
      if (p.revision == rev && !published && approvalCount >= 2 && ci && heads["schema"] == expected) {
        heads["schema"] = proposed;
        published = true;
      }
      announce ePublishDecisionDone, p;
      if (p.revision == rev && published && heads["schema"] == proposed) {
        announce eHead, (branch = "schema", oldHead = expected, newHead = proposed);
      }
    }
    on eRead do (p: (branch: string, head: string)) {
      announce eRead, (branch = p.branch, head = heads[p.branch]);
    }
  }
}

machine Driver {
  var store: BundleStore;
  start state Init {
    entry {
      store = new BundleStore();
      send store, eSubmitReq, (bundle = "b", revision = 1, digest = "digest-s0-s1");
      send store, eApproveReq, (bundle = "b", revision = 1, reviewer = "alice");
      send store, eApproveReq, (bundle = "b", revision = 1, reviewer = "bob");
      send store, eCIReq, (bundle = "b", revision = 1, digest = "digest-s0-s1", passed = true);
      send store, ePublishDecisionReq, (bundle = "b", revision = 1, outcome = "published", apply = true);
      send store, eRead, (branch = "schema", head = "s0");
    }
  }
}

test tcCertifiedAtomicPublish [main=Driver]:
  assert BundleInvariant in { BundleStore, Driver };
