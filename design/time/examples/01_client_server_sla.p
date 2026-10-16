/*****************************************************************************
 * 1. End-to-end SLA for a single request/response.
 *
 * Shows: per-event delivery latency, `elapse` for local work, and a budget
 * asserted in the client with `now()`, which includes the client's own
 * queueing (unlike a spec, which sees arrival stamps).
 *
 * Worst path: 5ms (req) + 2ms + 10ms (db) + 5ms (resp) = 22ms <= 25ms. OK.
 * Change the Bank's elapse to 4ms..14ms and the check fails with a trace that
 * ends in the critical path.
 *****************************************************************************/

type tWithdrawReq  = (client: Client, rId: int, amount: int);
type tWithdrawResp = (rId: int, ok: bool);

event eWithdrawReq : tWithdrawReq  latency 1ms..5ms;
event eWithdrawResp: tWithdrawResp latency 1ms..5ms;

machine Bank {
  var balance: int;
  start state Serving {
    entry { balance = 100; }
    on eWithdrawReq do (r: tWithdrawReq) {
      elapse 1ms..2ms;                       // validate request
      elapse 3ms..10ms;                      // read-modify-write the balance
      if (balance - r.amount >= 10) { balance = balance - r.amount; }
      send r.client, eWithdrawResp, (rId = r.rId, ok = balance >= 10);
    }
  }
}

machine Client {
  var bank: Bank;
  var t0: duration;
  var next: int;

  start state Init {
    entry (b: Bank) { bank = b; goto Requesting; }
  }

  state Requesting {
    entry {
      next = next + 1;
      t0 = now();
      send bank, eWithdrawReq, (client = this, rId = next, amount = choose(50));
    }
    on eWithdrawResp do (r: tWithdrawResp) {
      assert now() - t0 <= 25ms,
        format("withdraw {0} took up to {1}", r.rId, now() - t0);
      if (next < 3) { goto Requesting; }
    }
  }
}

machine Main {
  start state Init {
    entry { new Client(new Bank()); }
  }
}

test tcWithdrawSLA [main=Main]: { Main, Client, Bank };
