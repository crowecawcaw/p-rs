/*****************************************************************************
 * 3. Scatter-gather: a search fans out to N shards and merges all results.
 *
 * Shows: fan-in takes the max. The aggregator's clock after the N-th reply is
 * max over shards of (send + shard work + 2 links), plus the aggregator's own
 * per-reply handling, which is sequential and so adds up (N * 0.5ms).
 *
 * Also shows head-of-line queueing: replies are handled one at a time, so a
 * large N costs even when the shards are fast. With N = 4:
 *   worst = 2 (req) + 5 (fan-out link) + 30 (slowest shard) + 5 (reply link)
 *         + 4 * 0.5 (merge) + 2 (resp) = 46ms <= 50ms.  N = 16: 52ms. Bug.
 *****************************************************************************/

type tQuery   = (client: machine, q: int);
type tShardQ  = (agg: Aggregator, q: int);
type tPartial = (shard: int, hits: int);

event eQuery    : tQuery   latency 1ms..2ms;
event eShardQ   : tShardQ  latency 1ms..5ms;
event ePartial  : tPartial latency 1ms..5ms;
event eResult   : int      latency 1ms..2ms;

machine Shard {
  var id: int;
  start state Init { entry (i: int) { id = i; goto Serving; } }
  state Serving {
    on eShardQ do (s: tShardQ) {
      elapse 5ms..30ms;                      // index scan; long tail
      send s.agg, ePartial, (shard = id, hits = choose(10));
    }
  }
}

machine Aggregator {
  var shards: seq[Shard];
  var pending: int;
  var total: int;
  var client: machine;

  start state Init {
    entry (n: int) {
      var i: int;
      while (i < n) { shards += (i, new Shard(i)); i = i + 1; }
      goto Idle;
    }
  }

  state Idle {
    on eQuery do (q: tQuery) {
      var s: Shard;
      client = q.client;
      pending = sizeof(shards);
      total = 0;
      foreach (s in shards) { send s, eShardQ, (agg = this, q = q.q); }
      goto Gathering;
    }
  }

  state Gathering {
    on ePartial do (p: tPartial) {
      elapse 0.5ms;                          // merge one partial result
      total = total + p.hits;
      pending = pending - 1;
      if (pending == 0) { send client, eResult, total; goto Idle; }
    }
    defer eQuery;
  }
}

machine Client {
  var t0: duration;
  start state Init {
    entry (agg: Aggregator) {
      t0 = now();
      send agg, eQuery, (client = this, q = 1);
    }
    on eResult do {
      assert now() - t0 <= 50ms, format("search took up to {0}", now() - t0);
    }
  }
}

machine Main {
  start state Init { entry { new Client(new Aggregator(4)); } }
}

test tcSearch [main=Main]: { Main, Client, Aggregator, Shard };
