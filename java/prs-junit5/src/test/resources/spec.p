event eAdd: int;
event eTake: int;
event eOther;
event eTuple: (units: int);
event eBegin;
event eEnd;

spec Counter observes eAdd, eTake {
  var available: int;
  start state Ready {
    on eAdd do (n: int) { available = available + n; }
    on eTake do (n: int) {
      available = available - n;
      assert available >= 0, "counter below zero";
    }
  }
}

spec Other observes eOther {
  start state Ready { on eOther do {} }
}

spec Snapshot observes eTuple {
  start state Ready {
    on eTuple do (p: (units: int)) { assert p.units > 0, "snapshot changed"; }
  }
}

spec Completion observes eBegin, eEnd {
  start state Ready { on eBegin goto Pending; }
  hot state Pending { on eEnd goto Ready; }
}
