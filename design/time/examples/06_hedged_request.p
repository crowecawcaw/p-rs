/*****************************************************************************
 * 6. Hedged requests: if replica A has not answered within 20ms, also ask
 *    replica B, and take whichever reply comes first.
 *
 * A has a long tail (5..200ms round trip), B is 5..30ms. Budget: 60ms.
 * The ideal bound is min(A, 20 + B) = min(200, 50) = 50ms.
 *
 * This sample needs --timed, including its narrowing rule: when the scheduler
 * delivers e ahead of a pending delivery f, it records e <= f (e happened no
 * later than f). Without --timed the scheduler may deliver A's
 * slow reply first, the reported bound is 200ms, and the check fails:
 * hedging cannot be verified by an untimed checker.
 *
 *   untimed:  bug (pessimistic, 200ms)
 *   --timed:  A first before the hedge  -> clamped to <= 20ms
 *             hedge fires @20, then A or B first -> clamped to <= 50ms. OK.
 *****************************************************************************/

type tGet   = (client: Client, id: int);
type tReply = (from: int, id: int);

event eGet   : tGet;
event eReply : tReply;
event eHedge : int;

machine Replica {
  var id: int;
  var work: duration;
  start state Init {
    entry (cfg: (id: int, work: duration)) { id = cfg.id; work = cfg.work; goto Serving; }
  }
  state Serving {
    on eGet do (g: tGet) {
      elapse work;
      send g.client, eReply, (from = id, id = g.id);
    }
  }
}

machine Client {
  var a: Replica;
  var b: Replica;
  var t0: duration;

  start state Init {
    entry {
      a = new Replica((id = 0, work = 5ms..200ms));
      b = new Replica((id = 1, work = 5ms..30ms));
      t0 = now();
      send a, eGet, (client = this, id = 1);
      send this, eHedge, 1 latency 20ms;
      goto Waiting;
    }
  }

  state Waiting {
    on eHedge do { send b, eGet, (client = this, id = 1); }
    on eReply do (r: tReply) {
      assert now() - t0 <= 60ms,
        format("hedged get answered by replica {0} after up to {1}", r.from, now() - t0);
      goto Done;
    }
  }

  state Done { ignore eReply, eHedge; }
}

machine Main {
  start state Init { entry { new Client(); } }
}

test tcHedge [main=Main]: { Main, Client, Replica };
// snappea check tcHedge          -> bug (pessimistic)
// snappea check tcHedge --timed  -> no bug
