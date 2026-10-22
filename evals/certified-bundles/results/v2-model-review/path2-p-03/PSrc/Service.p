type tAttempt = (client: Publisher, bundle: string, expected: string, proposed: string);
event ePublish: tAttempt;
event eResult: (bundle: string, outcome: string);

type tClientInput = (service: Coordinator, bundle: string, proposed: string);

// The global CAS pointer is the single commit point. A successful event changes
// the head and memoizes the bundle outcome together in this atomic handler.
machine Coordinator {
  var head: string;
  var published: set[string];

  start state Init {
    entry { head = "s0"; }
    on ePublish do (p: tAttempt) {
      if (p.bundle in published) {
        send p.client, eResult, (bundle = p.bundle, outcome = "published");
      } else if (p.expected != head) {
        send p.client, eResult, (bundle = p.bundle, outcome = "conflict");
      } else {
        head = p.proposed;
        published += (p.bundle);
        announce eCommitted, (bundle = p.bundle, expected = p.expected, newHead = p.proposed);
        send p.client, eResult, (bundle = p.bundle, outcome = "published");
      }
    }
  }
}

machine Publisher {
  var service: Coordinator;
  var bundle: string;
  var proposed: string;

  start state Init {
    entry (input: tClientInput) {
      service = input.service;
      bundle = input.bundle;
      proposed = input.proposed;
      send service, ePublish, (client = this, bundle = bundle,
                               expected = "s0", proposed = proposed);
      goto Waiting;
    }
  }
  state Waiting {
    on eResult do (r: (bundle: string, outcome: string)) { goto Done; }
  }
  state Done {}
}

machine Race {
  start state Init {
    entry {
      var service: Coordinator;
      service = new Coordinator();
      new Publisher((service = service, bundle = "bundle-a", proposed = "sA"));
      new Publisher((service = service, bundle = "bundle-b", proposed = "sB"));
    }
  }
}
