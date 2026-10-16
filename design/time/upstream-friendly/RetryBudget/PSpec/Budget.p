// The budget is an ordinary spec. In a spec handler, Now() is the clock of
// the machine that announced/sent the observed event.
spec ResolvesWithinDeadline observes eOutcome {
  start state Watching {
    on eOutcome do (o: (ok: bool, started: int)) {
      assert Now() - o.started <= ms(1000),
        format("resolved (ok = {0}) after up to {1}us; deadline is 1000000us",
               o.ok, Now() - o.started);
    }
  }
}
