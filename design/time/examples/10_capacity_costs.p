/*****************************************************************************
 * 10. Non-time costs: read capacity, retry amplification, and a rate limit.
 *
 * Shows that additive costs need no new language feature: a spec monitor sums
 * fields of observed events, exactly as in today's P. Only the per-second
 * rate limit uses the time extension (`now()`), which is why time is the part
 * worth building into the language.
 *
 * Properties:
 *   CostPerRequest: each client request consumes at most 8 RCU and causes at
 *                   most 3 Db calls, retries included.
 *   RcuRateLimit:   at most 40 RCU in any 1s window (table capacity).
 *
 * A request reads 1..4 rows at 1 RCU per row, and throttled reads are still
 * billed. The Api retries a throttled read up to 3 times, so a 4-row request
 * that is throttled twice costs 12 RCU: CostPerRequest fails.
 *****************************************************************************/

type tQuery  = (reqId: int, api: Api, rows: int);
type tDbCall = (reqId: int, rcu: int);

event eClientReq : (reqId: int, rows: int) latency 1ms..3ms;
event eQuery     : tQuery                  latency 1ms..3ms;
event eQueryResp : (reqId: int, throttled: bool) latency 1ms..3ms;
event eDbCall    : tDbCall;                 // announced: one call and its RCU

machine Db {
  start state Serving {
    on eQuery do (q: tQuery) {
      var throttled: bool;
      throttled = $;
      announce eDbCall, (reqId = q.reqId, rcu = q.rows);   // throttled reads still bill
      elapse 2ms..6ms;
      send q.api, eQueryResp, (reqId = q.reqId, throttled = throttled);
    }
  }
}

machine Api {
  var db: Db;
  var tries: map[int, int];
  var rowsOf: map[int, int];
  start state Init { entry (d: Db) { db = d; goto Serving; } }
  state Serving {
    on eClientReq do (r: (reqId: int, rows: int)) {
      tries[r.reqId] = 1; rowsOf[r.reqId] = r.rows;
      send db, eQuery, (reqId = r.reqId, api = this, rows = r.rows);
    }
    on eQueryResp do (resp: (reqId: int, throttled: bool)) {
      if (resp.throttled && tries[resp.reqId] < 3) {
        tries[resp.reqId] = tries[resp.reqId] + 1;
        elapse 10ms;                                       // backoff
        send db, eQuery, (reqId = resp.reqId, api = this, rows = rowsOf[resp.reqId]);
      }
    }
  }
}

spec CostPerRequest observes eDbCall {
  var rcu: map[int, int];
  var calls: map[int, int];
  start state S {
    on eDbCall do (c: tDbCall) {
      if (!(c.reqId in rcu)) { rcu[c.reqId] = 0; calls[c.reqId] = 0; }
      rcu[c.reqId] = rcu[c.reqId] + c.rcu;
      calls[c.reqId] = calls[c.reqId] + 1;
      assert rcu[c.reqId] <= 8,   format("request {0} used {1} RCU", c.reqId, rcu[c.reqId]);
      assert calls[c.reqId] <= 3, format("request {0} made {1} Db calls", c.reqId, calls[c.reqId]);
    }
  }
}

spec RcuRateLimit observes eDbCall {
  var log: seq[(at: duration, rcu: int)];
  start state S {
    on eDbCall do (c: tDbCall) {
      var i: int;
      var inWindow: int;
      log += (sizeof(log), (at = now(), rcu = c.rcu));
      while (i < sizeof(log)) {
        // Uncertain comparison: both outcomes are explored, so a call that
        // *could* fall in the same window as this one is counted on some
        // schedule. That is the intended (conservative) reading of a limit.
        if (now() - log[i].at < 1s) { inWindow = inWindow + log[i].rcu; }
        i = i + 1;
      }
      assert inWindow <= 40, format("{0} RCU within 1s", inWindow);
    }
  }
}

machine Client {
  start state Init {
    entry (api: Api) {
      var i: int;
      while (i < 5) {
        send api, eClientReq, (reqId = i, rows = choose(4) + 1);
        i = i + 1;
      }
    }
  }
}

machine Main {
  start state Init { entry { new Client(new Api(new Db())); } }
}

test tcCost [main=Main]: assert CostPerRequest, RcuRateLimit in { Main, Client, Api, Db };
