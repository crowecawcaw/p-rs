/*****************************************************************************
 * 9. Failover: a write must complete within 1s even if the leader crashes
 *    at some point in the first 300ms.
 *
 * Shows: a budget over a long multi-machine path: crash at an uncertain time,
 * detection by missed heartbeats, a detection timeout given as a range,
 * takeover, the client's timeout / re-resolve / retry loop.
 *
 * Election is abstracted to a single standby that takes over (see the Raft
 * tutorial for election safety); we care about how long failover takes.
 *
 * Worst path (crash time c <= 300ms):
 *   last heartbeat reaches the standby                 <= c + 3ms
 *   detection timer (150..300ms) expires               <= c + 303ms
 *   takeover + announce to Directory                   <= c + 318ms
 *   client notices on its next attempt: up to one loop of
 *   100ms timeout + resolve round trip (6ms)           <= c + 424ms
 *   write to the new leader: links + replication (30ms) <= c + 454ms
 * With c <= 300ms: about 754ms <= 1s. OK.
 * Detection timeout 500ms..1s instead: about 1.25s. Bug.
 *
 * Run with --timed. Untimed, the scheduler may keep delivering the client's
 * retries and never deliver the standby's detection timer, so the client's
 * clock grows without bound: a false bug.
 *****************************************************************************/

type tWrite = (client: Client, id: int);

event eWrite        : tWrite latency 1ms..5ms;
event eWriteAck     : int    latency 1ms..5ms;
event eHeartbeat             latency 1ms..3ms;
event eResolve      : Client latency 1ms..3ms;
event eResolved     : Node   latency 1ms..3ms;
event eLeaderIs     : Node   latency 1ms..5ms;
event eTick;                                  // leader's heartbeat timer
event eDetect       : int;                    // standby's detection timer
event eCrash;
event eSetup        : (dir: Directory, peer: Node, leader: bool);
event eAttemptTimeout : int;

// Name service: knows the leader it was last told about. It does not detect
// crashes; it learns of the new leader only when the standby takes over.
machine Directory {
  var leader: Node;
  start state Init {
    entry (n: Node) { leader = n; goto Serving; }
  }
  state Serving {
    on eLeaderIs do (n: Node) { leader = n; }
    on eResolve do (c: Client) { send c, eResolved, leader; }
  }
}

machine Node {
  var dir: Directory;
  var peer: Node;
  var gen: int;

  start state Init {
    on eSetup do (cfg: (dir: Directory, peer: Node, leader: bool)) {
      dir = cfg.dir; peer = cfg.peer;
      if (cfg.leader) { goto Leading; }
      goto Standby;
    }
    defer eWrite, eCrash;
  }

  state Leading {
    entry { send this, eTick latency 50ms; }
    on eTick do {
      if (peer != null) { send peer, eHeartbeat; }
      send this, eTick latency 50ms;
    }
    on eWrite do (w: tWrite) {
      elapse 10ms..20ms;                       // replicate
      send w.client, eWriteAck, w.id;
    }
    on eCrash goto Crashed;
    ignore eHeartbeat, eDetect;
  }

  state Standby {
    entry {
      gen = gen + 1;
      send this, eDetect, gen latency 150ms..300ms;
    }
    on eHeartbeat goto Standby;                // re-arm (old timer becomes stale)
    on eDetect do (g: int) {
      if (g != gen) { return; }
      elapse 2ms..10ms;                        // take over: fence old leader, load state
      send dir, eLeaderIs, this;
      peer = null;
      goto Leading;
    }
    ignore eWrite;                             // not the leader: client will time out
  }

  state Crashed {
    ignore eWrite, eHeartbeat, eDetect, eTick, eCrash;
  }
}

machine Client {
  var dir: Directory;
  var leader: Node;
  var t0: duration;
  var attempt: int;

  start state Init {
    entry (d: Directory) { dir = d; t0 = now(); goto Resolving; }
  }

  state Resolving {
    entry { send dir, eResolve, this; }
    on eResolved do (n: Node) { leader = n; goto Writing; }
    ignore eWriteAck, eAttemptTimeout;
  }

  state Writing {
    entry {
      attempt = attempt + 1;
      send leader, eWrite, (client = this, id = attempt);
      send this, eAttemptTimeout, attempt latency 100ms;
    }
    on eWriteAck do (id: int) { if (id == attempt) { goto Done; } }
    on eAttemptTimeout do (id: int) { if (id == attempt) { goto Resolving; } }
  }

  state Done {
    entry { assert now() - t0 <= 1s, format("write took up to {0} across failover", now() - t0); }
    ignore eWriteAck, eAttemptTimeout, eResolved;
  }
}

machine Main {
  start state Init {
    entry {
      var dir: Directory;
      var leader: Node;
      var standby: Node;
      leader = new Node();
      standby = new Node();
      dir = new Directory(leader);
      send leader, eSetup, (dir = dir, peer = standby, leader = true);
      send standby, eSetup, (dir = dir, peer = null, leader = false);
      new Client(dir);
      send leader, eCrash latency 0ms..300ms;  // after eSetup (FIFO per link)
    }
  }
}

test tcFailover [main=Main]: { Main, Client, Directory, Node };
// prs check tcFailover --timed
