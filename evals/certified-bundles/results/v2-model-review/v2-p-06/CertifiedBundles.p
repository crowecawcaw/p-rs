event eRootCommit: (headsAdvanced: int, memoized: bool);

spec AtomicBundlePublish observes eRootCommit {
  start state Open {
    on eRootCommit do (p: (headsAdvanced: int, memoized: bool)) {
      assert p.headsAdvanced == 0 || p.headsAdvanced == 2,
        "a bundle must update all target heads atomically";
      assert p.memoized,
        "every publish outcome must be memoized with the root transition";
    }
  }
}

machine RootWriter {
  start state Init {
    entry {
      Commit();
      goto Done;
    }
  }
  state Done {}

  fun Commit() {
    var advanced: int;
    advanced = choose(2) * 2;
    announce eRootCommit, (headsAdvanced = advanced, memoized = true);
  }
}

machine Driver {
  start state Init {
    entry {
      var first: RootWriter;
      var second: RootWriter;
      first = new RootWriter();
      second = new RootWriter();
      goto Done;
    }
  }
  state Done {}
}


test tcAtomicRootCommit [main=Driver]:
  assert AtomicBundlePublish in {RootWriter, Driver};
