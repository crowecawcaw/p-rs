// The atomic design keeps the spec.
test tcAtomicReserve [main=AtomicReserve]:
  assert InventoryCorrect in { Inventory, OrderClient, AtomicReserve };

// The check-then-reserve design oversells: two orders both see 3 units available.
// This test case is expected to find that bug.
test tcCheckThenReserve [main=CheckThenReserve]:
  assert InventoryCorrect in { Inventory, OrderClient, CheckThenReserve };
