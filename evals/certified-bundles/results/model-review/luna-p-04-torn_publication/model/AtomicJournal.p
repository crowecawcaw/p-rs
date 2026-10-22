type tCasReq = (client: CasClient, request: int, expected: int, value: int);
event eCas: tCasReq;
event eCasResp: (won: bool);
event eCommit: (version: int, left: int, right: int);

// Abstraction of immutable journal records plus a CAS of the shared head pointer.
// The handler is the one indivisible store.cas on that pointer. The branch pair,
// published marker, and request outcome become visible at that same step.
machine Journal {
  var version: int;
  var left: int;
  var right: int;
  var outcomes: map[int, bool];

  start state Init {
    entry {
      version = 0;
      left = 0;
      right = 0;
      outcomes = default(map[int, bool]);
      goto Serving;
    }
  }
  state Serving {
    on eCas do (req: tCasReq) {
      var won: bool;
      if (req.request in outcomes) {
        won = outcomes[req.request];
      } else {
        won = req.expected == version;
        outcomes[req.request] = won;
        if (won) {
          version = version + 1;
          left = req.value;
          right = req.value;
          announce eCommit, (version = version, left = left, right = right);
        }
      }
      send req.client, eCasResp, (won = won,);
    }
  }
}

// A P monitor checks each atomic visibility point for a whole multi-branch commit.
spec WholeCommit observes eCommit {
  var previous: int;
  start state Clean {
    on eCommit do (p: (version: int, left: int, right: int)) {
      assert p.left == p.right, "multi-branch commit became partially visible";
      assert p.version == previous + 1, "head version skipped or repeated";
      previous = p.version;
    }
  }
}

machine CasClient {
  var journal: Journal;
  var request: int;
  var value: int;
  var first: bool;
  var retried: bool;

  start state Init {
    entry (input: (journal: Journal, request: int, value: int)) {
      journal = input.journal;
      request = input.request;
      value = input.value;
      retried = false;
      send journal, eCas, (client = this, request = request, expected = 0, value = value);
      goto Waiting;
    }
  }
  state Waiting {
    on eCasResp do (resp: (won: bool)) {
      if (!retried) {
        first = resp.won;
        retried = true;
        // A retry reuses its request id and must replay the durable memo.
        send journal, eCas, (client = this, request = request, expected = 0, value = value);
      } else {
        assert first == resp.won, "retry changed its recorded outcome";
        goto Done;
      }
    }
  }
  state Done {}
}

machine CompetingPublishes {
  start state Init {
    entry {
      var journal: Journal;
      journal = new Journal();
      new CasClient((journal = journal, request = 1, value = 11));
      new CasClient((journal = journal, request = 2, value = 22));
    }
  }
}

test tcAtomicJournal [main=CompetingPublishes]:
  assert WholeCommit in { Journal, CasClient, CompetingPublishes };
