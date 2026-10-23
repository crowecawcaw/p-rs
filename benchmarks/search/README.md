Run `python3 scripts/bench-search.py --out search.json` from a Rust development
shell. The suite compares random, feedback, DFS, randomized DFS, and auto with
20 fixed seeds and a 10,000 schedule budget per model/strategy/seed. It stops each
run as soon as all three independently named assertion sites have failed.
Every schedule counts, including successful schedules and exhaustive prunes;
trace rendering replays do not count. Raw JSON records first discovery of each
bug, schedules tested, completion or censoring, seed, budget, revision and dirty
status. A missing `schedules_to_all` means incomplete, never zero schedules.

The fixed synthetic suite separates visible branch progress (8 and 12 choices),
opaque integer choices (no incremental coverage signal), and competing deliveries
(4 and 5 clients, three failing orders). All bug sites are independently
reachable by construction. These are controlled search experiments, not a claim
about arbitrary production models. DFS can beat sampling on small finite trees;
coverage cannot guide an opaque predicate before it succeeds. Keep these negative
controls when adding harder and real-world workloads. Version the suite whenever
models, bug identities, bounds or stopping rules change.

Compare completion rates first. The report gives completed-only medians and a
penalized mean, charging twice the budget for an unfinished run; this avoids
making a strategy look faster just because it misses the hardest seeds. Inspect
per-seed raw results before drawing conclusions. Wall time is deliberately not
the objective here (instrumentation and state hashing have different costs).

`.github/workflows/bench-search.yml` measures every main commit and appends its
raw result to `search.jsonl` on the existing `benchmarks` branch, using the same
retrying append helper as the throughput benchmarks. Runs have no concurrency
group so pending commits are not dropped. Manual branch runs upload artifacts
without updating main history.
