// Small P model of the atomic branch-head check and multi-branch publish.
// Two certified bundles compete to replace the same base head. The service's
// publish handler is the atomic transaction represented by one P transition.
type tPublishReq = (bundle: string, expected: string, proposed: string, client: PublishWorker);
event ePublish: tPublishReq;
event ePublishResult: (bundle: string, result: string);
event eAdvanced: (bundle: string, oldHead: string, newHead: string);

machine BundleService {
  var head: string;
  var aPublished: bool;
  var bPublished: bool;

  start state Init {
    entry {
      head = "s0";
      aPublished = false;
      bPublished = false;
      goto Serving;
    }
  }

  state Serving {
    on ePublish do (req: tPublishReq) {
      var result: string;
      if ((req.bundle == "A" && aPublished) || (req.bundle == "B" && bPublished)) {
        result = "published";
      } else if (head != req.expected) {
        result = "conflict";
      } else {
        announce eAdvanced, (bundle = req.bundle, oldHead = head, newHead = req.proposed);
        head = req.proposed;
        if (req.bundle == "A") { aPublished = true; }
        if (req.bundle == "B") { bPublished = true; }
        result = "published";
      }
      send req.client, ePublishResult, (bundle = req.bundle, result = result);
    }
  }
}

machine PublishWorker {
  var service: BundleService;
  var bundle: string;
  var proposed: string;

  start state Init {
    entry (input: (service: BundleService, bundle: string, proposed: string)) {
      service = input.service;
      bundle = input.bundle;
      proposed = input.proposed;
      send service, ePublish, (bundle = bundle, expected = "s0", proposed = proposed, client = this);
      goto Waiting;
    }
  }

  state Waiting {
    on ePublishResult do (result: (bundle: string, result: string)) {
      assert result.bundle == bundle;
      goto Done;
    }
  }

  state Done {}
}

spec PublishSafety observes eAdvanced {
  var advanced: bool;
  start state Initial {
    on eAdvanced do (p: (bundle: string, oldHead: string, newHead: string)) {
      assert !advanced, "two bundles advanced from the same base";
      assert p.oldHead == "s0", "publish used an unexpected parent";
      advanced = true;
    }
  }
}

machine CompetingPublishes {
  start state Init {
    entry {
      var service: BundleService;
      service = new BundleService();
      new PublishWorker((service = service, bundle = "A", proposed = "a1"));
      new PublishWorker((service = service, bundle = "B", proposed = "b1"));
    }
  }
}

test tcCompetingPublishes [main=CompetingPublishes]:
  assert PublishSafety in { BundleService, PublishWorker, CompetingPublishes };
