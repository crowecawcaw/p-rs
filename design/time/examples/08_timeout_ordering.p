/*****************************************************************************
 * 8. Timeout layering: client timeout must cover the LB's, which must cover
 *    the server's work, or work is orphaned and retries pile up.
 *
 * Shows: timeouts at three layers as delayed self-sends, and a spec that
 * compares times recorded at different machines (`now()` in a spec is the
 * sender's clock at the observed send/announce).
 *
 * Property (NoOrphanedWork): the server never finishes an attempt after the
 * client has given up on it. Both orders are checked because the untimed
 * scheduler may show the spec either event first; the comparison itself is
 * on stamps, not on schedule order.
 *
 * The spec compares two ranges, so it states the modality explicitly (specs may
 * not contain uncertain comparisons). This is a non-monotone check: "no bug"
 * is weaker than for a plain budget, because inflated clocks can hide it.
 *
 * Config below: client 1s, LB 2s, server work 200ms..1.5s -> bug.
 * Fix: client 2.5s > LB 2s > server worst 1.5s + links.
 *****************************************************************************/

type tCall = (id: int, caller: machine);

event eCall      : tCall latency 1ms..5ms;
event eCallResp  : int   latency 1ms..5ms;
event eTimeout   : int;
event eGaveUp    : int;                        // announced: client abandons attempt
event eFinished  : int;                        // announced: server completes attempt

machine Server {
  start state Serving {
    on eCall do (c: tCall) {
      elapse 200ms..1500ms;
      announce eFinished, c.id;
      send c.caller, eCallResp, c.id;
    }
  }
}

machine LB {
  var server: Server;
  var callers: map[int, machine];
  start state Init { entry (s: Server) { server = s; goto Serving; } }
  state Serving {
    on eCall do (c: tCall) {
      callers[c.id] = c.caller;
      send server, eCall, (id = c.id, caller = this);
      send this, eTimeout, c.id latency 2s;
    }
    on eCallResp do (id: int) {
      if (id in callers) { send callers[id], eCallResp, id; callers -= (id); }
    }
    on eTimeout do (id: int) {
      if (id in callers) { send callers[id], eCallResp, -id; callers -= (id); }  // 504
    }
  }
}

machine Client {
  var lb: LB;
  var attempt: int;
  start state Init { entry (l: LB) { lb = l; goto Calling; } }
  state Calling {
    entry {
      attempt = attempt + 1;
      send lb, eCall, (id = attempt, caller = this);
      send this, eTimeout, attempt latency 1s;
    }
    on eCallResp do (id: int) { if (id == attempt) { goto Done; } }
    on eTimeout do (id: int) {
      if (id != attempt) { return; }
      announce eGaveUp, attempt;
      if (attempt < 2) { goto Calling; }
      goto Done;
    }
  }
  state Done { ignore eCallResp, eTimeout; }
}

spec NoOrphanedWork observes eGaveUp, eFinished {
  var gaveUpAt: map[int, duration];
  var finishedAt: map[int, duration];
  start state S {
    on eGaveUp do (id: int) {
      gaveUpAt[id] = now();
      if (id in finishedAt) { check(id); }
    }
    on eFinished do (id: int) {
      finishedAt[id] = now();
      if (id in gaveUpAt) { check(id); }
    }
  }
  fun check(id: int) {
    assert worst(finishedAt[id]) < best(gaveUpAt[id]),
      format("attempt {0}: server finished at {1} but the client gave up at {2}",
             id, finishedAt[id], gaveUpAt[id]);
  }
}

machine Main {
  start state Init { entry { new Client(new LB(new Server())); } }
}

test tcTimeouts [main=Main]: assert NoOrphanedWork in { Main, Client, LB, Server };
