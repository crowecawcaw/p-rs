/*****************************************************************************
 * 9. Failover: a write must complete within 1s even if the leader crashes
 *    once while it is in flight.
 *
 * Shows: a budget over a long, multi-machine path (crash detection by missed
 * heartbeats, randomized election timeout as a range, vote round trip,
 * client timeout, redirect, retry). The election itself is abstracted: we
 * care about its duration, not its safety (see the Raft tutorial for that).
 *
 * Worst path:
 *   leader crashes just after the last heartbeat           0ms
 *   follower election timeout (randomized 150..300ms)     300ms
 *   vote request + reply, announce to Directory            15ms   -> 315ms
 *   client: its 100ms request timeout is shorter, so meanwhile it has been
 *   probing Directory, which answers "no leader" until the election ends;
 *   each probe costs up to 6ms + 50ms backoff              ~60ms  -> ~375ms
 *   write to the new leader: links + replication           30ms   -> ~405ms
 * Budget 1s: OK.  Raise the election timeout range to 500..1000ms: bug.
 * (Directory learns of the crash immediately; it stands in for a membership
 * service.)
 *
 * Under untimed scheduling heartbeats and election timeouts race freely, so
 * spurious elections are explored; they cost time but cannot shorten the
 * failover, so the worst-case bound is still meaningful. Use --timed to
 * exclude them.
 *****************************************************************************/

type tWrite = (client: Client, id: int);

event eWrite        : tWrite latency 1ms..5ms;
event eWriteAck     : int    latency 1ms..5ms;
event eNotLeader    : int    latency 1ms..5ms;
event eHeartbeat             latency 1ms..3ms;
event eTick;
event eElectTimeout : int;
event eWhoIsLeader  : Client latency 1ms..3ms;
event eLeaderIs     : Node   latency 1ms..3ms;
event eNoLeader             latency 1ms..3ms;
event eElected      : Node   latency 1ms..5ms;   // vote round trip folded in
event eCrash;
event eClientTimeout: int;
event eRetry;

machine Directory {
  var leader: Node;
  var known: bool;
  start state Serving {
    on eElected do (n: Node) { leader = n; known = true; }
    on eCrash do { known = false; }
    on eWhoIsLeader do (c: Client) {
      if (known) { send c, eLeaderIs, leader; } else { send c, eNoLeader; }
    }
  }
}

machine Node {
  var peers: seq[Node];
  var dir: Directory;
  var isLeader: bool;
  var term: int;

  start state Init {
    entry (cfg: (dir: Directory, leader: bool)) {
      dir = cfg.dir; isLeader = cfg.leader;
      if (isLeader) { send dir, eElected, this; goto Leading; }
      goto Following;
    }
    on ePeers do (p: seq[Node]) { peers = p; }
    defer eWrite;
  }

  state Leading {
    entry { send this, eTick latency 50ms; }
    on eTick do {
      var p: Node;
      foreach (p in peers) { send p, eHeartbeat; }
      send this, eTick latency 50ms;
    }
    on eWrite do (w: tWrite) {
      elapse 10ms..20ms;                         // replicate to a majority
      send w.client, eWriteAck, w.id;
    }
    on eCrash goto Crashed;                      // injected once by Main
    ignore eHeartbeat, eElectTimeout;            // from a spurious election
  }

  state Following {
    entry { term = term + 1; send this, eElectTimeout, term latency 150ms..300ms; }
    on eHeartbeat do { goto Following; }         // re-arms the timer (new term)
    on eElectTimeout do (t: int) {
      if (t != term) { return; }                 // stale: a heartbeat arrived
      elapse 2ms..10ms;                          // request votes, count majority
      send dir, eElected, this;
      isLeader = true;
      goto Leading;
    }
    on eWrite do (w: tWrite) { send w.client, eNotLeader, w.id; }
  }

  state Crashed { ignore eWrite, eHeartbeat, eElectTimeout, eCrash, eTick; }
}

event ePeers: seq[Node];

machine Client {
  var dir: Directory;
  var leader: Node;
  var t0: duration;
  var attempt: int;

  start state Init {
    entry (cfg: (dir: Directory, leader: Node)) {
      dir = cfg.dir; leader = cfg.leader;
      t0 = now();
      goto Writing;
    }
  }

  state Writing {
    entry {
      attempt = attempt + 1;
      send leader, eWrite, (client = this, id = attempt);
      send this, eClientTimeout, attempt latency 100ms;
    }
    on eWriteAck do (id: int) { if (id == attempt) { goto Done; } }
    on eNotLeader do (id: int) { if (id == attempt) { goto Probing; } }
    on eClientTimeout do (id: int) { if (id == attempt) { goto Probing; } }
  }

  state Probing {
    entry { send dir, eWhoIsLeader, this; }
    on eLeaderIs do (n: Node) { leader = n; goto Writing; }
    on eNoLeader do { send this, eRetry latency 50ms; }
    on eRetry do { goto Probing; }
    ignore eWriteAck, eNotLeader, eClientTimeout;
  }

  state Done {
    entry { assert now() - t0 <= 1s, format("write took up to {0} across failover", now() - t0); }
    ignore eWriteAck, eNotLeader, eClientTimeout, eLeaderIs, eNoLeader, eRetry;
  }
}

machine Main {
  start state Init {
    entry {
      var dir: Directory;
      var n0: Node; var n1: Node; var n2: Node;
      var all: seq[Node];
      dir = new Directory();
      n0 = new Node((dir = dir, leader = true));
      n1 = new Node((dir = dir, leader = false));
      n2 = new Node((dir = dir, leader = false));
      all += (0, n1); all += (1, n2);
      send n0, ePeers, all;
      new Client((dir = dir, leader = n0));
      if ($) { send n0, eCrash; send dir, eCrash; }   // crash at most once
    }
  }
}

test tcFailover [main=Main]: { Main, Client, Directory, Node };
