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

Measurements compare code commit `63686f4` with the original feedback scheduler at
`9983d8d`, using Rust 1.99.1 and suite version 1. The checked-in raw results are
[baseline.json](baseline.json), [guided.json](guided.json), and
[holdout.json](holdout.json). Each records an exact commit and `dirty: false`.
Random and both exhaustive strategies produced identical per-seed results before
and after the change on seeds 0–19.

On seeds 0–19, standalone feedback found all three bugs in **100/100 model runs**,
compared with **80/100** for original feedback and **83/100** for random. DFS and
randomized DFS completed all 100. On the separate seeds 20–39, final feedback
completed **100/100**, random **81/100**, and both exhaustive strategies **100/100**.
The automatic portfolio completed 99/100 on seeds 0–19 and 98/100 on seeds 20–39;
its initial exhaustive probe and work allocation mean it does not inherit the
standalone feedback scheduler's full improvement.

The table shows mean schedules to all bugs, charging **20,000** for an unfinished
10,000-schedule run (the report's penalized mean). This includes misses, rather
than comparing only the easiest completed seeds.

| Model | Original feedback | Final feedback | Random | DFS | Final feedback, seeds 20–39 |
|---|---:|---:|---:|---:|---:|
| branch-8 | 1,914.3 | 287.1 | 1,359.2 | 27.0 | 207.3 |
| branch-12 | 17,927.6 | 486.1 | 17,383.7 | 39.0 | 602.1 |
| opaque-8 | 1,971.5 | 349.6 | 1,406.2 | 513.0 | 351.1 |
| delivery-4 | 453.9 | 175.2 | 204.6 | 427.0 | 215.8 |
| delivery-5 | 6,113.1 | 682.9 | 4,702.7 | 3,699.0 | 735.6 |

DFS still wins decisively on the finite branch trees. Random has a lower median
on delivery-4 for seeds 0–19 (101.5 versus 154.5 schedules), even though feedback
has a better mean. On delivery-5, final feedback's median is 577.5 schedules,
versus 3,549 for random (19/20 complete) and 3,699 for DFS (20/20 complete).
These synthetic results establish improvements on specific search shapes, not a
universal advantage over random or exhaustive checking.

The experiments exposed three problems: timeline-only novelty misses data-only
branches; source `if` coverage misses progress inside conjunctions; and mutating
several input choices can repeatedly destroy rare prefixes. The final scheduler
uses statement/branch probes plus short-circuit outcomes, retains event/scenario
novelty for repeated stages, favors recent productive seeds, and mixes single
input changes with fresh suffixes. Distinct failing data prefixes get a bounded
share of mutations to find neighboring failure sites. Scheduling mutations retain
the broader operator: applying suffix resampling to scheduling choices regressed
the delivery workloads. After 128 schedules without progress, fresh sampling rises
from 25% to 75%, while corpus mutations continue. Upstream compatibility retains
its existing timeline/MinHash behavior and random stream.

Reproduce the separate-seed check with:

```sh
python3 scripts/bench-search.py --seed-start 20 --out holdout.json
```

The broad corpus benchmark runs every expanded test case from `examples/`, the
vendored tutorials, `design/time/`, the existing guided-prefix model, and every
upstream `Correct`/`DynamicError` regression directory:

```sh
python3 scripts/bench-corpus.py --out corpus.json --report corpus.md
```

It defaults to one seed, 200 schedules, 1,000 steps and a 0.1-second safety cap per
strategy/case, with two project processes. Use `--budget`, `--steps`, `--seeds`,
`--seconds-per-run`, and `--jobs` for deeper passes; `--scope examples` omits the
upstream regression directories and `--filter Inventory` selects matching inputs.
Every case gets per-strategy schedules to every distinct failure site and to all
sites in a shared catalog (the union found by all strategies/seeds). It computes
these first-discovery counts after the runs, so no strategy gets an easier target
because it missed a bug. A completed exhaustive run can certify the catalog;
otherwise **all observed issues does not mean all possible issues**. A case with
no discoveries has an unknown total unless exhaustion completed, never a bogus
zero schedules to all bugs. The report explicitly lists expected-bug cases with
no common issue observed.

Potential liveness temperature warnings are kept separately because unfair DFS
does not perform that check. Missing foreign implementations are execution
limitations, not model bugs. Static-error tests have no schedules and are listed
as excluded; compilation/resolution failures, runtime errors, step/time caps and
unrun measurements remain visible. The language application examples share the
inventory P model, so their Go/Python/Java harnesses do not create separate P
schedule spaces. Time examples use the checker's default time options; missing
profile/adapters remain explicit limitations of this quick pass.

The per-commit workflow also appends the broad corpus results to `corpus.jsonl`
on the `benchmarks` branch and uploads the complete Markdown per-case table.
Verify accounting with `python3 -m unittest discover -s scripts -p test_bench_corpus.py`.

The checked-in [full per-case corpus report](corpus.md) and
[compressed raw measurements](corpus.json.gz) record a three-seed quick pass at
commit `befd956` (clean working tree). All 459 model inputs were inventoried:
289 resolved/compiled projects supplied **495 expanded test cases**; 148 static
error inputs were excluded as not schedulable, and 22 inputs had explicit
compilation/resolution limitations. There are 5,940 strategy/seed measurements.
Among 145 cases with observed common issues, feedback found the entire shared
catalog in **421/435** runs, random in **415/435**, DFS in **408/435**, and auto in
**421/435**. These counts include all successful schedules preceding discovery.
The broad improvement is modest and several cases still favor random.

Exhaustion certified 404 case catalogs. Thirteen expected DynamicError cases had
no common issue observed; the other unexhausted catalogs are provisional. The
quick 200-schedule, 1,000-step pass and 0.1-second safety cap leave misses and
potential liveness warnings visible. Its per-run catalog digest is stored in the
JSON: compare results across commits only when the catalogs and bounds match,
or recompute against a shared union catalog from the raw discoveries.

Reproduce this measured pass with:

```sh
python3 scripts/bench-corpus.py --seeds 3 --out corpus.json --report corpus.md
```

The gzip file contains ordinary JSON and can be read with `gzip -dc
benchmarks/search/corpus.json.gz` or Python's `gzip.open`.
