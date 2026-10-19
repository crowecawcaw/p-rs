// Stock and reservations per product; the key is the product.
event eStocked: (units: int);
event eReserved: (order: string, units: int);
event eAudit: (note: string); // declared, observed by no spec

spec InventoryCorrect observes eStocked, eReserved {
  var available: int;
  start state Ready {
    on eStocked do (p: (units: int)) { available = available + p.units; }
    on eReserved do (p: (order: string, units: int)) {
      assert p.units <= available, format("reservation {0} exceeds available stock", p.order);
      available = available - p.units;
    }
  }
}
