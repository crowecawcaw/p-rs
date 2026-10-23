# Next-choice feedback experiment

This experiment's code was removed after it, since it did not justify a default; its
source is in the history up to `5c2d0e6`. Follow-up: [frontier checkpoints](frontier-guidance.md) recover the staged-prefix misses and improve the broad-corpus result.

Keep the existing defaults. Prefix-guided exploration gives mixed results: it ties the broad corpus discovery rate and regresses the largest delivery workload. This tests one inexpensive context and policy, not all forms of online guidance.

## Policy

The opt-in `CheckerOptions.feedback_choice_guidance` (since removed) applied to native `Feedback` with a random scheduler. At each newly generated input or scheduler random choice, a 25% exploration branch selects uniformly among the least-tried alternatives for that context. Other choices retain the existing proposal. Context is a rolling hash of the controlled-choice prefix, the input/scheduler stream identity, and the domain size. It is not a complete program-state hash or a measure of coverage reward.

Visit counts persist between schedules, the prefix resets each schedule, and at most 262,144 alternative counters are retained. Once the cap is reached, existing contexts still get guidance and new contexts follow their original proposals. Recorded prefixes remain intact; mutations can change them through the existing mutation operators. Executed overrides are recorded in the corpus. No choices are rejected and no runs are pruned. Compatibility behavior and automatic search defaults are unchanged.

## Results

The broad corpus inventoried 459 project inputs, with 495 expanded runnable test cases. Of these, 145 cases had observed issues. Three seeds give 435 issue-bearing runs per strategy. Each run had a 200-schedule, 1,000-step budget and a 0.25-second safety cap. Catalogs use the union discovered across all strategies and seeds, including random, DFS, and auto. This is a quick pass: 13 expected DynamicError cases had no common issue observed, and unknown catalogs do not establish safety.

| Policy | Runs finding every observed issue |
|---|---:|
| Existing feedback | 421/435 |
| Guide every choice | 417/435 |
| Guide fresh choices, preserve recorded prefixes | 421/435 |

For the refined policy, paired issue-bearing runs improved on 13 seeds, tied on 404, and regressed on 18. Charging 400 schedules for a miss, mean schedules to all observed issues went from 16.04 to 16.47. Most corpus cases fail immediately or have no discoveries, so these aggregate means should not be treated as a broad speedup metric.

The synthetic suite uses five models, three assertion sites each, 10,000 schedules per run, and seeds 0–39. Seeds 0–19 were used during refinement; seeds 20–39 were checked afterward without further policy changes. Both existing and refined feedback found every site in all 200 model/seed runs.

| Model | Existing feedback, mean schedules to all sites | Refined guide | Change |
|---|---:|---:|---:|
| branch-8 | 247.25 | 216.95 | −12.3% |
| branch-12 | 544.13 | 524.18 | −3.7% |
| opaque-8 | 350.38 | 305.95 | −12.7% |
| delivery-4 | 195.48 | 159.18 | −18.6% |
| delivery-5 | 709.28 | 941.78 | +32.8% |

These pooled means hide seed sensitivity: branch-12 regressed on seeds 0–19 but improved on 20–39. The initial policy, which could override recorded prefixes, needed 2,437.6 schedules on branch-12 on seeds 0–19, versus 486.1 for the baseline. Preserving productive prefixes removed most of that regression.

Elapsed-time evidence does not establish a gain. In the separate-seed synthetic pass, total feedback time was 0.221 seconds versus 0.226 seconds with guidance, despite fewer guided schedules. Across 1,476 uncapped corpus pairs that both executed 200 schedules, summed elapsed time was 10.53 versus 11.29 seconds. These short wall-clock measurements share a host with other work and include checker overhead; treat them as descriptive, not a controlled throughput benchmark.

## Artifacts and reproduction

Base revision: `b33f3ad408073f046ee6ab1c3866cf45ae201bed`; Rust 1.99.1. Measurements were made in a dirty working tree. Each compressed JSON file records its policy and the SHA-256 of a source patch against the base revision. Patches include the benchmark harness and tests. Decompress with Python `gzip.open` or `gzip -dc`.

- [Refined corpus raw data](choice-corpus.json.gz) (per-case report: `scripts/bench-corpus.py --report-from`).
- [Refined synthetic seeds 0–19](choice-synthetic.json.gz) and [seeds 20–39](choice-holdout.json.gz).
- The initial all-choice policy's results and both source diffs were removed; the
  refined policy shipped (off by default) in `3fa5c63`..`5c2d0e6`.

At `5c2d0e6` (the flag no longer exists):

```sh
python3 scripts/bench-corpus.py --choice-guidance --seeds 3 --seconds-per-run 0.25 \
  --out /tmp/choice-corpus.json --report /tmp/choice-corpus.md
python3 scripts/bench-search.py --choice-guidance --out /tmp/choice-synthetic.json
python3 scripts/bench-search.py --choice-guidance --seed-start 20 --out /tmp/choice-holdout.json
```

The benchmark flag adds `feedback-choice` alongside existing strategies. Without it, benchmark defaults remain unchanged. The experiment currently applies to standalone feedback, not auto's feedback phase.

Validation passed: workspace tests, formatting, Clippy across all targets, workspace build, emit-only build, and Python corpus-accounting tests. Added tests check seeded determinism, input/delivery replay, compatibility isolation, bounded exploration counts, recording executed choices, and preservation of corpus prefixes.

A further experiment could use semantic context and reward useful continuations, while retaining the prefix-preservation constraint. That needs its own comparison; the current results do not justify enabling this policy by default.
