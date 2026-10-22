test tcConcurrentPublish [main=ConcurrentPublish]:
  assert BundleInvariant in { BundleStore, Publisher, ConcurrentPublish };
