# Feedback frontier checkpoints

Saving a successful execution prefix at the moment it reaches new progress substantially improves the staged-prefix misses. The measured experiment is now enabled by default for native random feedback and automatic search’s feedback phase. It does not eliminate every miss.

## What changed

`CheckerOptions.feedback_frontier_guidance` controls checkpoints (enabled by default) for native random-scheduler feedback. The runtime reports the machine declaration, state, and event immediately before an actual handler runs. Ignored, deferred, and unhandled events do not create checkpoints.

The strategy tracks how often each such handler executes within a run, across machine instances. A new maximum count retains the consumed scheduling and input choices at that exact point. Counts 1–64 are individual milestones; larger powers of two are milestones. Half of subsequent schedules sample a saved checkpoint, replay its prefix, and draw a fresh continuation. The other half use existing feedback selection and mutation. The runtime still restarts from the beginning; this is replay, not a cloned VM state.

Checkpoints discard unconsumed mutation suffixes and preserve the choices already executed. A checkpoint whose replay has no further random choices is retired from sampling. At most 256 checkpoints and 262,144 random choices are retained. This avoids repeatedly destroying a long successful prefix and avoids wasting attempts on deterministic terminal prefixes. Existing choice-count guidance remains experimental and off by default. It is independent and was not enabled for these measurements.

## Same-budget results

The broad corpus still inventories 459 project inputs and executes 495 expanded cases. Runs use 200 schedules, 1,000 steps, and a 0.25-second safety cap. No model was changed for this comparison.

A new unhandled `accept` failure in `Multi_Paxos_3` appeared at schedule 46 with checkpoint feedback, seed 0. It was absent from the earlier common catalog. To avoid giving different seed batches different targets, the shared report recomputes one catalog from all strategies over seeds **0, 1, 2, 20, 21, 22**.

| Seed batch | Existing feedback: all observed sites found | Checkpoint feedback |
|---|---:|---:|
| 0–2 | 418/435 | 430/435 |
| 20–22 | 415/435 | 430/435 |
| Combined, same union catalog | **833/870** | **860/870** |

These are runs over the 145 issue-bearing cases, not distinct defects. The remaining 350 cases do not add to this denominator. Ten incomplete checkpoint runs remain: five OSR runs and five Multi-Paxos runs. A catalog is the observed union, not a promise that no other issues exist. Static exclusions, compilation failures, bounds, time caps, missing adapters, and provisional catalogs remain visible in the raw data.

Against the original catalog on seeds 0–2, checkpoints recover **11 of the previous 14 feedback misses**. All eleven were staged prefixes. OSR's rare `eTimerStopped` site remains outside the 200-schedule budget for those three seeds.

The dedicated repeated-stage model supplies both data and scheduling prefixes at depths 12, 20, and 24:

| Seeds | Existing feedback | Checkpoint feedback |
|---|---:|---:|
| 0–2, six cases | 7/18 | 18/18 |
| Separate seeds 20–39, six cases | 34/120 | **120/120** |

Both use 200 schedules and 1,000 steps, with a 1-second safety cap in these focused runs. No focused run was time-capped. Depth-24 data failures appeared at schedules 28–54 on seeds 0–2; depth-24 scheduling failures appeared at schedules 114–162.

## What is still hard

At 2,000 schedules, all three OSR seeds found its three observed sites with both baseline and checkpoint feedback. This is a budget improvement, not evidence that the checkpoint policy wins on OSR. On Multi-Paxos at 2,000 schedules, baseline completed 2/3 seeds and checkpoints 1/3 against the four-site catalog: checkpoint feedback still misses the rare `accept` site on seeds 1 and 2. Adding next-choice frequency guidance to checkpoints did not fix these two models at 200 schedules; those negative trials are retained.

The five-model synthetic suite found every site in every feedback and checkpoint run on seeds 0–39. Mean schedules to all sites were:

| Model | Existing feedback | Checkpoint feedback |
|---|---:|---:|
| branch-8 | 247.25 | 247.25 |
| branch-12 | 544.13 | 544.13 |
| opaque-8 | 350.38 | 350.38 |
| delivery-4 | 195.48 | 155.43 |
| delivery-5 | 709.28 | 724.48 |

The data-only synthetic ladders have no event handlers, so checkpoint selection never activates and their choice streams stay identical. Delivery-5 is sensitive to seeds: checkpoints regressed on 0–19 and improved on 20–39. Over 1,479 matched, uncapped 200-schedule corpus pairs, summed elapsed time was 9.97 vs 11.13 seconds on the first batch, and 9.70 vs 10.88 seconds on the second. These wall-clock samples share a host with other work; they show overhead, not a controlled throughput result. The discovery gains do not imply a universal speedup.

## Liveness harness diagnosis

The 13 expected DynamicError cases with no common issue in the previous report are a separate category. One (`Liveness_10`) already produced a potential-liveness warning, which the common safety-issue catalog intentionally excludes. The other **12 selected `DefaultImpl` cases attach no monitors**. A scheduler cannot discover the unasserted liveness property.

A separate diagnostic added an explicit test asserting each file's declared monitor, using in-memory source files without modifying the vendored model or the main benchmark. Nine of those twelve then reported a liveness issue with ordinary feedback: three terminal hot-state failures and six temperature-based potential-liveness warnings. The warm-state example and two nondeterministic liveness examples remained undetected at 20 schedules, 1,000 steps, and the existing temperature threshold. These results change the asserted property and are not folded into the unchanged-corpus totals above. The corpus benchmark now records the monitors attached to each case so this distinction is inspectable.

## Reproduction and artifacts

Base revision: `b33f3ad408073f046ee6ab1c3866cf45ae201bed`; Rust 1.99.1; dirty working tree. [Source patch](frontier.patch) applies independently to that base and includes the earlier choice experiment. Compressed JSON files record the source-patch hash and policy. The combined report merges measurements already made; its seed values are explicitly listed.

- [Shared per-case report](frontier-corpus-shared.md) and [raw measurements](frontier-corpus-shared.json.gz).
- Individual corpus passes: [seeds 0–2](frontier-corpus.json.gz), [seeds 20–22](frontier-corpus-holdout.json.gz).
- Repeated-stage runs: [seeds 0–2](frontier-prefix.json.gz), [seeds 20–39](frontier-prefix-holdout.json.gz).
- Synthetic runs: [seeds 0–19](frontier-synthetic.json.gz), [seeds 20–39](frontier-synthetic-holdout.json.gz).
- Deeper runs: [OSR](frontier-osr-deep.json.gz), [Multi-Paxos](frontier-paxos-deep.json.gz).
- Combined-policy negative trials: [OSR](frontier-osr-combined.json.gz), [Multi-Paxos](frontier-paxos-combined.json.gz).
- [Explicit-monitor diagnostic](frontier-liveness-diagnostic.json.gz) and [diagnostic script](frontier-liveness-diagnostic.py).

```sh
python3 scripts/bench-corpus.py --frontier-guidance --seeds 3 --seconds-per-run 0.25 \
  --out /tmp/frontier-corpus.json --report /tmp/frontier-corpus.md
python3 scripts/bench-corpus.py --frontier-guidance --seed-start 20 --seeds 3 --seconds-per-run 0.25 \
  --out /tmp/frontier-holdout.json --report /tmp/frontier-holdout.md
python3 scripts/bench-corpus.py --frontier-guidance --filter guided_prefix --seed-start 20 --seeds 20 \
  --seconds-per-run 1 --out /tmp/frontier-prefix.json
python3 scripts/bench-search.py --frontier-guidance --out /tmp/frontier-synthetic.json
python3 scripts/bench-search.py --frontier-guidance --seed-start 20 --out /tmp/frontier-synthetic-holdout.json
python3 benchmarks/search/frontier-liveness-diagnostic.py
```

Workspace tests, Clippy across all targets, formatting, workspace and emit-only builds, and Python corpus-accounting tests passed. New tests exercise prefix truncation, checkpoint retirement, bounded retention, staged data and delivery discovery, and replay of every staged failure across three seeds.
