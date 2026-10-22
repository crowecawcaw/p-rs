machine ConcurrentPublish {
  start state Init {
    entry {
      var store: BundleStore;
      store = new BundleStore();
      new Publisher((store = store, bundle = "a", request = "pa", retry = true));
      new Publisher((store = store, bundle = "b", request = "pb", retry = false));
    }
  }
}
