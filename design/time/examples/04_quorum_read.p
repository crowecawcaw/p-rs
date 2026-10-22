/*****************************************************************************
 * 4. Quorum read: ask 3 replicas, answer after 2 replies.
 *
 * Shows: the coordinator's latency is the time of the 2nd reply it *handles*.
 * In the base (untimed) mode the scheduler may hand it the slow replica's
 * reply first, so the bound is pessimistic: worst = the two slowest replicas.
 * Under `snappea check --timed`, a reply whose interval is entirely after another
 * pending reply's cannot be dequeued first, so the bound tightens to the 2nd
 * fastest replica when the intervals separate.
 *
 * Replica 2 sits in another region (40..60ms). Budget 30ms:
 *   untimed: fails (may wait on replica 2).
 *   --timed: passes, since replicas 0 and 1 are both certainly earlier.
 * The third reply arrives after the response has been sent; it is ignored in
 * Idle, and ignored events have no clock effect.
 * Each replica's `link` is one stored value used for both directions, so the
 * request and reply latencies of a replica are the same value (correlated),
 * not two independent draws from the range.
 *****************************************************************************/

type tRead     = (client: machine, key: int);
type tReplicaQ = (coord: Coordinator, key: int);
type tReplicaR = (replica: int, version: int);

event eRead    : tRead;
event eReplicaQ: tReplicaQ;
event eReplicaR: tReplicaR;
event eReadResp: int;

machine Replica {
  var id: int;
  var link: duration;
  start state Init {
    entry (cfg: (id: int, link: duration)) { id = cfg.id; link = cfg.link; goto Serving; }
  }
  state Serving {
    on eReplicaQ do (q: tReplicaQ) {
      elapse 1ms..3ms;
      send q.coord, eReplicaR, (replica = id, version = choose(3)) latency link;
    }
  }
}

machine Coordinator {
  var replicas: seq[(r: Replica, link: duration)];
  var replies: int;
  var newest: int;
  var client: machine;

  start state Init {
    entry {
      replicas += (0, (r = new Replica((id = 0, link = 1ms..4ms)),   link = 1ms..4ms));
      replicas += (1, (r = new Replica((id = 1, link = 2ms..6ms)),   link = 2ms..6ms));
      replicas += (2, (r = new Replica((id = 2, link = 40ms..60ms)), link = 40ms..60ms));
      goto Idle;
    }
  }

  state Idle {
    on eRead do (r: tRead) {
      var i: int;
      client = r.client; replies = 0; newest = -1;
      while (i < sizeof(replicas)) {
        send replicas[i].r, eReplicaQ, (coord = this, key = r.key) latency replicas[i].link;
        i = i + 1;
      }
      goto Waiting;
    }
    ignore eReplicaR;                        // stragglers from the previous read
  }

  state Waiting {
    on eReplicaR do (r: tReplicaR) {
      replies = replies + 1;
      if (r.version > newest) { newest = r.version; }
      if (replies == 2) { send client, eReadResp, newest; goto Idle; }
    }
    defer eRead;
  }
}

spec ReadSLA observes eRead, eReadResp {
  var t0: duration;
  start state S {
    on eRead do { t0 = now(); }
    on eReadResp do { assert arrival() - t0 <= 30ms, format("quorum read took up to {0}", arrival() - t0); }
  }
}

machine Client {
  start state Init {
    entry (c: Coordinator) { send c, eRead, (client = this, key = 7); }
    ignore eReadResp;
  }
}

machine Main {
  start state Init { entry { new Client(new Coordinator()); } }
}

test tcQuorumRead [main=Main]: assert ReadSLA in { Main, Client, Coordinator, Replica };
// snappea check tcQuorumRead          -> bug (pessimistic: may wait on replica 2)
// snappea check tcQuorumRead --timed  -> no bug
