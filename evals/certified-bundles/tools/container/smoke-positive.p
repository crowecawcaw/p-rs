event eAllowed: (value: int);

spec Positive observes eAllowed {
  start state Init {
    on eAllowed do (p: (value: int)) {
      assert p.value > 0, "value must be positive";
    }
  }
}
