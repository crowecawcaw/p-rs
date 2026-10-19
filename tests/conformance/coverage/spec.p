// Every reservation is eventually shipped; the key is the product.
event eReserved: (order: string, units: int);
event eShipped: (order: string);

spec EveryReservationShips observes eReserved, eShipped {
  var pending: int;
  start cold state Idle {
    on eReserved do { pending = pending + 1; goto Waiting; }
  }
  hot state Waiting {
    on eReserved do { pending = pending + 1; }
    on eShipped do {
      pending = pending - 1;
      if (pending == 0) { goto Idle; }
    }
  }
}
