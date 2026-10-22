type PublishReq = (client: Publisher, bundle: string, request: string);
event ePublish: PublishReq;
event eResponse: (request: string, result: string);
event ePublished: (bundle: string, oldHead: string, newHead: string);
event eRejected: (request: string, result: string);

machine BundleStore {
  var head: string;
  var outcomes: map[string, string];
  var published: map[string, bool];
  var ready: map[string, bool];

  start state Init {
    entry { head = "s0"; outcomes = default(map[string, string]); published = default(map[string, bool]); ready = default(map[string, bool]); ready["a"] = true; ready["b"] = true; goto Serving; }
  }
  state Serving {
    on ePublish do (r: PublishReq) {
      var result: string;
      if (r.request in outcomes) {
        result = outcomes[r.request];
      } else if (!(r.bundle in ready) || !ready[r.bundle]) {
        result = "not_ready";
        outcomes[r.request] = result;
        announce eRejected, (request = r.request, result = result);
      } else if (r.bundle in published && published[r.bundle]) {
        result = "published";
        outcomes[r.request] = result;
      } else if (head != "s0") {
        result = "conflict";
        outcomes[r.request] = result;
        announce eRejected, (request = r.request, result = result);
      } else {
        result = "published";
        outcomes[r.request] = result;
        published[r.bundle] = true;
        head = "s1";
        announce ePublished, (bundle = r.bundle, oldHead = "s0", newHead = head);
      }
      send r.client, eResponse, (request = r.request, result = result);
    }
  }
}

machine Publisher {
  var store: BundleStore;
  var bundle: string;
  var request: string;
  var retry: bool;
  start state Init {
    entry (input: (store: BundleStore, bundle: string, request: string, retry: bool)) {
      store = input.store; bundle = input.bundle; request = input.request; retry = input.retry;
      send store, ePublish, (client = this, bundle = bundle, request = request);
      goto Waiting;
    }
  }
  state Waiting {
    on eResponse do (r: (request: string, result: string)) {
      if (retry && r.request == request) {
        send store, ePublish, (client = this, bundle = bundle, request = request);
        retry = false;
        goto Waiting;
      }
      goto Done;
    }
  }
  state Done {}
}

spec BundleInvariant observes ePublished, eRejected {
  var head: string;
  var commits: int;
  start state Initial {
    entry { head = "s0"; commits = 0; goto Active; }
  }
  state Active {
    on ePublished do (p: (bundle: string, oldHead: string, newHead: string)) {
      assert head == p.oldHead, "publish did not check the current expected parent";
      assert p.newHead != p.oldHead, "publish must advance the branch";
      commits = commits + 1;
      assert commits <= 1, "two bundles published from the same parent";
      head = p.newHead;
    }
    on eRejected do (p: (request: string, result: string)) {
      assert p.result == "conflict" || p.result == "not_ready", "invalid rejection";
    }
  }
}
