// Monitor real application events. There are no model machines or P test drivers.
event eStocked: (units: int);
event eReserved: (order: string, units: int);
event eReleased: (order: string, units: int);

// emit's key gives each product its own stock and reservations.
spec InventoryCorrect observes eStocked, eReserved, eReleased {
  var available: int;
  var reservations: map[string, int];

  start state Unstocked {
    on eStocked do (p: (units: int)) {
      assert p.units >= 0, "negative initial stock";
      available = p.units;
      goto Ready;
    }
    on eReserved do { assert false, "reservation before stock setup"; }
    on eReleased do { assert false, "release before stock setup"; }
  }

  state Ready {
    on eStocked do { assert false, "product stocked twice"; }
    on eReserved do (p: (order: string, units: int)) {
      assert p.units > 0, "reservation must be positive";
      assert !(p.order in reservations), "duplicate reservation";
      available = available - p.units;
      assert available >= 0, "reservation exceeds available stock";
      reservations[p.order] = p.units;
    }
    on eReleased do (p: (order: string, units: int)) {
      assert p.order in reservations, "release without a reservation";
      assert p.units == reservations[p.order], "released quantity differs from reservation";
      available = available + p.units;
      reservations -= p.order;
    }
  }
}
