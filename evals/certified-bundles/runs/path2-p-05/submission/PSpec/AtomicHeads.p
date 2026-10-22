event eHeads: (left: int, right: int);

// Every successful bundle in this model targets both branches. A partial write
// would produce unequal heads and violate this invariant.
spec AtomicHeads observes eHeads {
  start state Stable {
    on eHeads do (heads: (left: int, right: int)) {
      assert heads.left == heads.right, "bundle exposed a partial head update";
    }
  }
}
