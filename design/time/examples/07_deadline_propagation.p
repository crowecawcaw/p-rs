/*****************************************************************************
 * 7. Deadline propagation (gRPC style) and "no certainly-wasted work".
 *
 * Shows: deadlines are plain `duration` values carried in payloads; branching
 * on an uncertain time comparison (`now() > r.deadline`) explores both
 * outcomes; `best(...)` states "certainly", where a plain assert states
 * "worst case".
 *
 * Chain: Frontend -> Api -> Db. The client's deadline is 100ms. Each hop
 * checks the deadline on arrival and fails fast if it has passed. The property
 * (spec NoWastedWork): the Db never starts work for a request whose original
 * deadline has *certainly* passed.
 *
 * Bug: Api sets a fresh deadline (now() + 500ms) for its call to Db instead of
 * forwarding the caller's. On Api's slow path the request reaches the Db at
 * [116ms, 164ms], certainly past the 100ms deadline, and the Db works on it
 * anyway. Fix: forward `r.deadline` (the commented-out line); the Db's own
 * check is then certainly true and it fails fast.
 *
 * Why `best`: we want to flag work that is wasted on every realization. Work
 * that merely *might* be late is the normal cost of deadlines; a plain
 * (worst-case) `assert now() <= deadline` would flag every request whose
 * interval straddles the deadline.
 *****************************************************************************/

type tReq    = (id: int, caller: machine, deadline: duration);
type tResult = (id: int, ok: bool);

event eFrontendReq : tReq    latency 1ms..3ms;
event eApiReq      : tReq    latency 1ms..3ms;
event eDbReq       : tReq    latency 1ms..3ms;
event eResult      : tResult latency 1ms..3ms;
event eDbWork      : (id: int);                 // announced to specs only

machine Db {
  start state Serving {
    on eDbReq do (r: tReq) {
      if (now() > r.deadline) {
        send r.caller, eResult, (id = r.id, ok = false);   // DEADLINE_EXCEEDED
        return;
      }
      announce eDbWork, (id = r.id,);
      elapse 5ms..20ms;
      send r.caller, eResult, (id = r.id, ok = true);
    }
  }
}

machine Api {
  var db: Db;
  var pending: map[int, machine];
  start state Init { entry (d: Db) { db = d; goto Serving; } }
  state Serving {
    on eApiReq do (r: tReq) {
      if (now() > r.deadline) {
        send r.caller, eResult, (id = r.id, ok = false);
        return;
      }
      if ($) {
        elapse 5ms..20ms;                        // fast path
      } else {
        elapse 110ms..150ms;                     // slow path: cold auth cache
      }
      pending[r.id] = r.caller;
      send db, eDbReq, (id = r.id, caller = this, deadline = now() + 500ms);   // BUG
      // send db, eDbReq, (id = r.id, caller = this, deadline = r.deadline);  // fix
    }
    on eResult do (res: tResult) {
      send pending[res.id], eResult, res;
      pending -= (res.id);
    }
  }
}

machine Frontend {
  var api: Api;
  var pending: map[int, machine];
  start state Init { entry (a: Api) { api = a; goto Serving; } }
  state Serving {
    on eFrontendReq do (r: tReq) {
      elapse 1ms..5ms;
      pending[r.id] = r.caller;
      send api, eApiReq, (id = r.id, caller = this, deadline = r.deadline);
    }
    on eResult do (res: tResult) {
      send pending[res.id], eResult, res;
      pending -= (res.id);
    }
  }
}

spec NoWastedWork observes eFrontendReq, eDbWork {
  var deadlineOf: map[int, duration];
  start state S {
    on eFrontendReq do (r: tReq) { deadlineOf[r.id] = r.deadline; }
    on eDbWork do (w: (id: int)) {
      assert best(now()) <= deadlineOf[w.id],
        format("Db started request {0} at least {1} after its deadline",
               w.id, best(now()) - deadlineOf[w.id]);
    }
  }
}

machine Client {
  start state Init {
    entry (fe: Frontend) {
      send fe, eFrontendReq, (id = 1, caller = this, deadline = now() + 100ms);
    }
    ignore eResult;
  }
}

machine Main {
  start state Init { entry { new Client(new Frontend(new Api(new Db()))); } }
}

test tcDeadlines [main=Main]: assert NoWastedWork in { Main, Client, Frontend, Api, Db };
