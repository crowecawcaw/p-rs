// A synthetic staged recovery path with a deliberately injected terminal failure.
// Each successful stage makes observable progress, so useful prefixes can be saved.
event eTry;
event eAdvance;
param depth: int;

machine RecoveryStage {
    var remaining: int;
    start state Ready {
        entry (count: int) { remaining = count; send this, eTry; }
        on eTry do { if ($) { send this, eAdvance; } }
        on eAdvance do {
            assert remaining > 1, "injected failure after rare recovery prefix";
            new RecoveryStage(remaining - 1);
        }
    }
}

machine DataDriver {
    start state Init {
        entry { new RecoveryStage(depth); }
    }
}

test param (depth in [12, 20, 24]) tcDataPrefix [main=DataDriver]: { DataDriver, RecoveryStage };

// Same shape, but progress depends exclusively on actor scheduling, not $/choose.
event eAck: bool;
event eNextRound;

machine Contender {
    start state SendAck {
        entry (args: (owner: RaceStage, preferred: bool)) {
            send args.owner, eAck, args.preferred;
        }
    }
}

machine RaceStage {
    var remaining: int;
    start state Waiting {
        entry (count: int) {
            remaining = count;
            new Contender((owner=this, preferred=false));
            new Contender((owner=this, preferred=true));
        }
        on eAck do (preferred: bool) {
            if (preferred) { send this, eNextRound; }
            goto Chosen;
        }
    }
    state Chosen {
        ignore eAck;
        on eNextRound do {
            assert remaining > 1, "injected failure after rare scheduling prefix";
            new RaceStage(remaining - 1);
        }
    }
}

machine ScheduleDriver {
    start state Init { entry { new RaceStage(depth); } }
}

test param (depth in [12, 20, 24]) tcSchedulePrefix [main=ScheduleDriver]: { ScheduleDriver, RaceStage, Contender };
