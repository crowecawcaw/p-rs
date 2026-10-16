/*****************************************************************************
 * 5. Retries with timeouts and exponential backoff vs. the caller's deadline.
 *
 * Shows: a timeout is a delayed self-send (`latency 300ms`), backoff is a
 * delayed self-send too (the machine stays responsive, unlike `elapse`), and
 * the retry loop accumulates time naturally.
 *
 * The server replies in 10..40ms or drops the request ($). Policy: 3 attempts,
 * 300ms timeout each, backoff 50ms then 100ms. The caller needs an answer
 * (success *or* error) within 1s.
 *
 *   attempt 1 sent @0, times out @300, backoff to 350
 *   attempt 2 sent @350, times out @650, backoff to 750
 *   attempt 3 sent @750, success by 790 or times out @1050  -> give up @1050
 *
 * The give-up path exceeds 1s: bug. The fix is a shorter per-attempt timeout
 * or fewer attempts; with 250ms timeouts the give-up path ends @900.
 *
 * Delayed sends are in flight until the scheduler delivers them, so both
 * reply-before-timeout and timeout-before-reply are explored.
 *
 * Untimed vs --timed: untimed, the 300ms timeout may be delivered before a
 * 10..40ms reply, so the checker also explores "slow" retries that cannot
 * happen. They do not produce false latency bugs here (their cost is bounded
 * by the drop path), but they do inflate retry counts, so any assertion about
 * retry amplification should be checked with --timed.
 *****************************************************************************/

type tReq  = (client: Client, attempt: int);
type tResp = (attempt: int);

event eReq     : tReq  latency 1ms..2ms;
event eResp    : tResp latency 1ms..2ms;
event eTimeout : int;                         // attempt number; latency set per send
event eBackoffDone;

machine Server {
  start state Serving {
    on eReq do (r: tReq) {
      if ($) {                                // dropped (crash, overload, network)
        return;
      }
      elapse 8ms..36ms;
      send r.client, eResp, (attempt = r.attempt);
    }
  }
}

machine Client {
  var server: Server;
  var attempt: int;
  var t0: duration;
  var backoff: seq[duration];

  start state Init {
    entry (s: Server) {
      server = s;
      backoff += (0, 50ms);
      backoff += (1, 100ms);
      t0 = now();
      goto Trying;
    }
  }

  state Trying {
    entry {
      attempt = attempt + 1;
      send server, eReq, (client = this, attempt = attempt);
      send this, eTimeout, attempt latency 300ms;
    }
    on eResp do (r: tResp) {
      if (r.attempt == attempt) { goto Done; }   // stale replies are ignored
    }
    on eTimeout do (a: int) {
      if (a != attempt) { return; }              // stale timer
      if (attempt == 3) { goto GaveUp; }
      send this, eBackoffDone latency backoff[attempt - 1];
      goto BackingOff;
    }
  }

  state BackingOff {
    on eBackoffDone goto Trying;
    ignore eResp, eTimeout;
  }

  state Done {
    entry { assert now() - t0 <= 1s, format("succeeded after up to {0}", now() - t0); }
    ignore eResp, eTimeout;
  }

  state GaveUp {
    entry { assert now() - t0 <= 1s, format("gave up after up to {0}", now() - t0); }
    ignore eResp, eTimeout;
  }
}

machine Main {
  start state Init { entry { new Client(new Server()); } }
}

test tcRetry [main=Main]: { Main, Client, Server };
