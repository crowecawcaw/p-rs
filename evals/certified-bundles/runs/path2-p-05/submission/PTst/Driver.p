fun StartCompetingPublishes() {
  var coordinator: Coordinator;
  coordinator = new Coordinator();
  new PublishClient((coordinator = coordinator, request = 1, proposed = 1));
  new PublishClient((coordinator = coordinator, request = 2, proposed = 2));
}

machine CompetingPublishes {
  start state Init { entry { StartCompetingPublishes(); } }
}
