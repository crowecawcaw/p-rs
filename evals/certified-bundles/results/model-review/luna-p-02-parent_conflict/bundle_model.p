event eTry: (client: Client, expected: int, next: int);
event eDone: (ok: bool);

machine SharedStore {
  var head: int;
  var successes: int;
  start state Init {
    entry { head = 0; successes = 0; goto Serving; }
  }
  state Serving {
    on eTry do (r: (client: Client, expected: int, next: int)) {
      if (head == r.expected) {
        head = r.next;
        successes = successes + 1;
        assert successes <= 1;
        send r.client, eDone, (ok = true,);
      } else {
        send r.client, eDone, (ok = false,);
      }
    }
  }
}

machine Client {
  var store: SharedStore;
  var next: int;
  start state Init {
    entry (input: (store: SharedStore, next: int)) {
      store = input.store;
      next = input.next;
      // Both bundles were certified against the same expected parent head 0.
      send store, eTry, (client = this, expected = 0, next = next);
      goto Waiting;
    }
  }
  state Waiting {
    on eDone do (result: (ok: bool)) { goto Done; }
  }
  state Done {}
}

machine Main {
  start state Init {
    entry {
      var store: SharedStore;
      store = new SharedStore();
      new Client((store = store, next = 1));
      new Client((store = store, next = 2));
    }
  }
}
