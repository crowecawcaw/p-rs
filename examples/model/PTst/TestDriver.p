// Three orders of 1 or 2 units each compete for 3 units of stock.
fun SetupInventory(checkFirst: bool) {
  var inventory: Inventory;
  var i: int;
  inventory = new Inventory(3);
  while (i < 3) {
    new OrderClient((inventory = inventory, order = format("order-{0}", i),
                     units = choose(2) + 1, checkFirst = checkFirst));
    i = i + 1;
  }
}

machine AtomicReserve {
  start state Init { entry { SetupInventory(false); } }
}

machine CheckThenReserve {
  start state Init { entry { SetupInventory(true); } }
}
