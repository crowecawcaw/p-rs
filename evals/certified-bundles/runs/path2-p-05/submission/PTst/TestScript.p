test tcCompetingPublishes [main=CompetingPublishes]:
  assert AtomicHeads in { Coordinator, PublishClient, CompetingPublishes };
