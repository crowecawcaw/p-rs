event eCommitted: (bundle: string, expected: string, newHead: string);

spec ChangeBundles observes eCommitted {
  var head: string;
  var published: set[string];

  start state Ready {
    entry { head = "s0"; }
    on eCommitted do (p: (bundle: string, expected: string, newHead: string)) {
      assert !(p.bundle in published), "bundle applied more than once";
      assert head == p.expected, "publish used a stale branch parent";
      head = p.newHead;
      published += (p.bundle);
    }
  }
}
