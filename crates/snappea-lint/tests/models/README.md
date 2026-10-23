# Model fixtures

`agent-p01.p`, `agent-p03.p`, and `agent-p06.p` are unmodified P sources from the
frozen v7 certified-bundles eval submissions. They retain the patterns that
motivated the rules: a guaranteed assertion, unused monitor payload fields, a
passive monitor, and an unselected spec. Only model source is included; no hidden
grader, transcripts, generated outputs, or implementation submissions are needed.

`tests/models.rs` also supplies small valid/counterexample models for each rule,
module selection, suppression scope, aliases, side effects, and liveness.
