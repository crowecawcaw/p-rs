machine TwoPublishers {
  start state Init {
    entry {
      var s: StoreModel;
      s = new StoreModel();
      new Publisher((store = s, request = "p1"));
      new Publisher((store = s, request = "p2"));
    }
  }
}
