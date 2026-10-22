event ePublish: (client: Client, bundle: int, request: int);
event eDone: (request: int, result: string);

event eCommit: (bundle: int, request: int, old_head: int, new_head: int);

machine Server {
  var head: int;
  var published: map[int, bool];
  var memo: map[int, string];
  var approvals: map[int, int];
  var ci: map[int, bool];
  start state Init {
    entry { head = 0; approvals[1] = 2; approvals[2] = 2; ci[1] = true; ci[2] = true; goto Serving; }
  }
  state Serving {
    on ePublish do (r: (client: Client, bundle: int, request: int)) {
      var result: string;
      if (r.request in memo) {
        result = memo[r.request];
      } else if (r.bundle in published && published[r.bundle]) {
        result = "published";
        memo[r.request] = result;
      } else if (!(r.bundle in approvals) || approvals[r.bundle] < 2 || !(r.bundle in ci) || !ci[r.bundle]) {
        result = "not_ready";
        memo[r.request] = result;
      } else if (head != 0) {
        result = "conflict";
        memo[r.request] = result;
      } else {
        // One root CAS visibility point applies heads, status, and request memo.
        announce eCommit, (bundle = r.bundle, request = r.request, old_head = head, new_head = r.bundle);
        head = r.bundle;
        published[r.bundle] = true;
        memo[r.request] = "published";
        result = "published";
      }
      send r.client, eDone, (request = r.request, result = result);
    }
  }
}

machine Client {
  var server: Server;
  var bundle: int;
  var request: int;
  var result: string;
  start state Init {
    entry (input: (server: Server, bundle: int, request: int)) {
      server = input.server; bundle = input.bundle; request = input.request;
      send server, ePublish, (client = this, bundle = bundle, request = request);
      goto Waiting;
    }
  }
  state Waiting {
    on eDone do (r: (request: int, result: string)) {
      result = r.result;
      goto Done;
    }
  }
  state Done {}
}

machine Main {
  var server: Server;
  start state Init {
    entry {
      server = new Server();
      new Client((server = server, bundle = 1, request = 11));
      new Client((server = server, bundle = 2, request = 22));
    }
  }
}

spec CommitChain observes eCommit {
  var head: int;
  var commits: int;
  start state Empty {
    on eCommit do (c: (bundle: int, request: int, old_head: int, new_head: int)) {
      assert c.old_head == head, "commit did not extend visible root";
      assert commits < 1, "two competing bundles both committed";
      assert c.new_head == c.bundle, "commit points at wrong bundle";
      head = c.new_head;
      commits = commits + 1;
    }
  }
}

test tcAtomicPublish [main=Main]:
  assert CommitChain in { Server, Client, Main };
