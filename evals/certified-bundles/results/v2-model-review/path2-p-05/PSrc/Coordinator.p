// Abstract model of the durable root-CAS linearization point. A single handler
// atomically checks all expected heads, advances both targets, and memoizes req.
event ePublish: (client: PublishClient, request: int, newHead: int);
event eReply: (request: int, result: bool);

machine Coordinator {
  var left: int;
  var right: int;
  var outcomes: map[int, bool];

  start state Init {
    entry {
      left = 0;
      right = 0;
      goto Serving;
    }
  }

  state Serving {
    on ePublish do (req: (client: PublishClient, request: int, newHead: int)) {
      var ok: bool;
      if (req.request in outcomes) {
        ok = outcomes[req.request];
      } else {
        if (left == 0 && right == 0) {
          left = req.newHead;
          right = req.newHead;
          outcomes[req.request] = true;
          ok = true;
          announce eHeads, (left = left, right = right);
        } else {
          outcomes[req.request] = false;
          ok = false;
        }
      }
      send req.client, eReply, (request = req.request, result = ok);
    }
  }
}

machine PublishClient {
  var coordinator: Coordinator;
  var request: int;
  var proposed: int;

  start state Init {
    entry (input: (coordinator: Coordinator, request: int, proposed: int)) {
      coordinator = input.coordinator;
      request = input.request;
      proposed = input.proposed;
      send coordinator, ePublish,
        (client = this, request = request, newHead = proposed);
      goto Waiting;
    }
  }

  state Waiting {
    on eReply do (reply: (request: int, result: bool)) {
      assert reply.request == request;
      goto Done;
    }
  }
  state Done {}
}
