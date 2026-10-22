// Abstract state machine for the certified two-branch publication gate.
// Store CAS races are represented by the atomic service transition: one successful
// root CAS commits the whole state change, while a failed CAS retries from a snapshot.
event eApprove: (revision: int, reviewer: string);
event eCI: (revision: int, digestMatches: bool, passed: bool);
event eRevise: (revision: int);
event ePublish: (revision: int);

machine CertifiedService {
  var revision: int;
  var schemaHead: string;
  var serviceHead: string;
  var alice: bool;
  var bob: bool;
  var carol: bool;
  var ci: bool;
  var published: bool;

  start state Ready {
    entry {
      revision = 1;
      schemaHead = "s0";
      serviceHead = "v0";
      alice = false;
      bob = false;
      carol = false;
      ci = false;
      published = false;
    }

    on eApprove do (r: (revision: int, reviewer: string)) {
      if (r.revision == revision && !published &&
          (r.reviewer == "alice" || r.reviewer == "bob" || r.reviewer == "carol")) {
        if (r.reviewer == "alice") { alice = true; }
        if (r.reviewer == "bob") { bob = true; }
        if (r.reviewer == "carol") { carol = true; }
      }
    }

    on eCI do (r: (revision: int, digestMatches: bool, passed: bool)) {
      if (r.revision == revision && r.digestMatches && !published) {
        ci = r.passed;
      }
    }

    on eRevise do (r: (revision: int)) {
      if (r.revision == revision && !published) {
        revision = revision + 1;
        alice = false;
        bob = false;
        carol = false;
        ci = false;
      }
    }

    on ePublish do (r: (revision: int)) {
      if (r.revision == revision && !published && ci &&
          alice && bob &&
          schemaHead == "s0" && serviceHead == "v0") {
        // The paired assignments are one abstract atomic transition.
        schemaHead = "s1";
        serviceHead = "v1";
        published = true;
        assert schemaHead == "s1" && serviceHead == "v1",
          "publication must advance every target branch";
      }
    }
  }
}

machine Scenario {
  var service: CertifiedService;
  start state Init {
    entry {
      service = new CertifiedService();
      send service, eApprove, (revision = 1, reviewer = "alice");
      send service, eApprove, (revision = 1, reviewer = "bob");
      send service, eCI, (revision = 1, digestMatches = true, passed = true);
      send service, ePublish, (revision = 1,);
    }
  }
}

test tcCertifiedAtomicPublish [main=Scenario];
