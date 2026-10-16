/*****************************************************************************
 * 2. Alternative paths (cache hit / miss) checked under two deployment
 *    profiles.
 *
 * Shows: protocol code with no numbers in it; latency profiles attached to
 * test cases; `extends`; per-link and per-state entries. The checker explores
 * both the hit path and the miss path (the cache's `$`), and the budget must
 * hold on each.
 *
 * Miss path = Client->Cache + Cache.Serving + Cache->Db + Db.Serving
 *             + Db->Cache + Cache.Serving + Cache->Client (default).
 * SameAZ      worst: 2 + 1 + 3 + 8 + 3 + 1 + 2 = 20ms  <= 50ms. OK.
 * CrossRegion worst: 2 + 1 + 90 + 8 + 90 + 1 + 2 = 194ms > 50ms: bug, while
 *             the hit path (2 + 1 + 2 = 5ms) still passes. The trace shows the
 *             miss branch was taken.
 * The client issues one request at a time; firing all four at once would add
 * Db queueing (4 x 8ms) to the last request, which is real but not the point
 * of this sample.
 *****************************************************************************/

type tGet  = (client: machine, key: int, id: int);
type tFill = (cache: Cache, req: tGet);
type tVal  = (id: int, val: int);

event eGet   : tGet;
event eDbRead: tFill;
event eDbVal : tGet;
event eVal   : tVal;

machine Db {
  start state Serving {
    on eDbRead do (f: tFill) { send f.cache, eDbVal, f.req; }
  }
}

machine Cache {
  var db: Db;
  var entries: set[int];
  start state Init { entry (d: Db) { db = d; goto Serving; } }
  state Serving {
    on eGet do (g: tGet) {
      if (g.key in entries && $) {          // hit (may also have been evicted)
        send g.client, eVal, (id = g.id, val = g.key);
      } else {
        send db, eDbRead, (cache = this, req = g);
      }
    }
    on eDbVal do (g: tGet) {
      entries += (g.key);
      send g.client, eVal, (id = g.id, val = g.key);
    }
  }
}

spec GetSLA observes eGet, eVal {
  var started: map[int, duration];
  start state S {
    on eGet do (g: tGet) { started[g.id] = now(); }
    on eVal do (v: tVal) {
      assert arrival() - started[v.id] <= 50ms,
        format("get {0} took up to {1}", v.id, arrival() - started[v.id]);
    }
  }
}

latency profile SameAZ {
  default            0.5ms..2ms;
  (Client, Cache)    1ms..2ms;
  (Cache, Db)        1ms..3ms;
  (Db, Cache)        1ms..3ms;
  Cache.Serving      0.2ms..1ms;     // lookup cost charged on every Cache handler
  Db.Serving         2ms..8ms;       // query cost
}

latency profile CrossRegion extends SameAZ {
  (Cache, Db)        60ms..90ms;
  (Db, Cache)        60ms..90ms;
}

machine Client {
  start state Init {
    entry (cache: Cache) {
      var i: int;
      while (i < 4) {
        send cache, eGet, (client = this, key = choose(2), id = i);
        receive { case eVal: (v: tVal) {} }
        i = i + 1;
      }
    }
  }
}

machine Main {
  start state Init { entry { new Client(new Cache(new Db())); } }
}

module Sys = { Main, Client, Cache, Db };

test tcGetSameAZ      [main=Main, latency=SameAZ]:      assert GetSLA in Sys;
test tcGetCrossRegion [main=Main, latency=CrossRegion]: assert GetSLA in Sys;
