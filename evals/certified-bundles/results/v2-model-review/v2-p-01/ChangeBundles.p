// Finite P abstraction of one certified two-branch bundle. Each handler is one
// atomic service transition; the Python implementation refines it with a CAS log.
type tUpdate = (schemaExpected: string, schemaNew: string,
                serviceExpected: string, serviceNew: string, digest: string);
type tSubmit = (update: tUpdate);
type tRevise = (revision: int, update: tUpdate);
type tApproval = (revision: int, reviewer: string);
type tCI = (revision: int, digest: string, passed: bool);
type tPublish = (revision: int, request: string);
type tPublishResult = (request: string, outcome: string);

event eSubmit: tSubmit;
event eRevise: tRevise;
event eApprove: tApproval;
event eCI: tCI;
event ePublish: tPublish;
event ePublishResult: tPublishResult;

machine ChangeBundles {
  var schemaHead: string;
  var serviceHead: string;
  var schemaExpected: string;
  var schemaNew: string;
  var serviceExpected: string;
  var serviceNew: string;
  var candidateDigest: string;
  var revision: int;
  var aliceApproved: bool;
  var bobApproved: bool;
  var carolApproved: bool;
  var ciPassed: bool;
  var isPublished: bool;
  var outcomes: map[string, string];

  start state Init {
    entry {
      schemaHead = "s0";
      serviceHead = "v0";
      revision = 0;
      aliceApproved = false;
      bobApproved = false;
      carolApproved = false;
      ciPassed = false;
      isPublished = false;
      goto Serving;
    }
  }

  state Serving {
    on eSubmit do (req: tSubmit) {
      if (revision == 0) {
        revision = 1;
        Install(req.update);
      }
    }

    on eRevise do (req: tRevise) {
      if (req.revision == revision && !isPublished) {
        revision = revision + 1;
        Install(req.update);
        aliceApproved = false;
        bobApproved = false;
        carolApproved = false;
        ciPassed = false;
      }
    }

    on eApprove do (req: tApproval) {
      if (req.revision == revision && !isPublished) {
        if (req.reviewer == "alice") { aliceApproved = true; }
        if (req.reviewer == "bob") { bobApproved = true; }
        if (req.reviewer == "carol") { carolApproved = true; }
      }
    }

    on eCI do (req: tCI) {
      if (req.revision == revision && req.digest == candidateDigest && !isPublished) {
        ciPassed = req.passed;
      }
    }

    on ePublish do (req: tPublish) {
      var outcome: string;
      if (req.request in outcomes) {
        outcome = outcomes[req.request];
      } else {
        if (req.revision != revision) {
          outcome = "stale";
        } else if (isPublished) {
          outcome = "published";
        } else if (!(ciPassed && ((aliceApproved && bobApproved) ||
                                  (aliceApproved && carolApproved) ||
                                  (bobApproved && carolApproved)))) {
          outcome = "not_ready";
        } else if (schemaHead != schemaExpected || serviceHead != serviceExpected) {
          outcome = "conflict";
        } else {
          outcome = "published";
          schemaHead = schemaNew;
          serviceHead = serviceNew;
          isPublished = true;
        }
        outcomes[req.request] = outcome;
      }
      announce ePublishResult, (request = req.request, outcome = outcome);
    }
  }

  fun Install(update: tUpdate) {
    schemaExpected = update.schemaExpected;
    schemaNew = update.schemaNew;
    serviceExpected = update.serviceExpected;
    serviceNew = update.serviceNew;
    candidateDigest = update.digest;
  }
}
