// Specs for the example lock server. Only events and specs: no P model is needed.

event eEnter: (client: string);
event eExit: (client: string);
event eRequest: (client: string);
event eGrant: (client: string);

// At most one client holds a lock at a time (one spec instance per lock: emit's key).
spec MutualExclusion observes eEnter, eExit {
  var holder: string;
  start state Free {
    on eEnter do (p: (client: string)) { holder = p.client; goto Held; }
    on eExit do (p: (client: string)) {
      assert false, format("{0} left a lock nobody holds", p.client);
    }
  }
  state Held {
    on eEnter do (p: (client: string)) {
      assert false, format("{0} entered while {1} is inside", p.client, holder);
    }
    on eExit do (p: (client: string)) {
      assert p.client == holder, format("{0} left, but {1} holds the lock", p.client, holder);
      goto Free;
    }
  }
}

// Every request is eventually granted (a liveness rule: checked with check_hot=True).
spec RequestsGranted observes eRequest, eGrant {
  var waiting: int;
  start cold state Idle {
    on eRequest do { waiting = waiting + 1; goto Waiting; }
    ignore eGrant;
  }
  hot state Waiting {
    on eRequest do { waiting = waiting + 1; }
    on eGrant do {
      waiting = waiting - 1;
      if (waiting == 0) { goto Idle; }
    }
  }
}
