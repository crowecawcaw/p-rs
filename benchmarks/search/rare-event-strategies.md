# Follow-up: remaining rare event failures

The default checkpoint change is in [draft PR #104](https://github.com/crowecawcaw/snappea/pull/104), commit `a013132c5469243af4b825c63ddc81c5c3744085`, rebased onto main at `5cb6bd9`. It enables checkpoints in native random feedback and automatic search's feedback phase. Workspace tests, Clippy, formatting, workspace build, emit-only build, and corpus-accounting checks passed after the rebase.

Follow-up experiments are on `codex/explore-rare-event-frontiers`. The PR's defaults are unchanged by this research.

## Strategies tested

The remaining OSR and Multi-Paxos failures require particular event orderings. I compared:

- State-entry checkpoints in addition to handler-count checkpoints.
- Existing partial-order sampling (`pos`, `feedbackpos`), priority change sampling (`pct`, `feedbackpct` with bound 2), and state-based Q-learning (`rl`).
- A new combination that gives the checkpoint corpus a partial-order scheduler as well as a random scheduler. Fresh generators can use partial-order sampling; mutations and checkpoint continuations retain their parent's scheduler family.

The new combination requires replayable partial-order checkpoints. The checkpoint saves consumed random choices and reconstructs priorities by replaying those choices from the start. It does not clone a running VM or carry a half-executed priority list into a new execution.

The first mixed trial used POS for 25% of fresh generators. A subsequent trial used 100%, which was then frozen before measuring seeds 20–39. This percentage applies to fresh generators, not all schedules: corpus mutations and checkpoint resumes still inherit existing generators, including the initial random one. Both policies retain the 50% checkpoint continuation selection and the existing mutation/fresh-sampling split for other attempts.

## Focused results

Each run used 200 schedules, 1,000 steps, and a 1-second safety cap. All strategies in each comparison share the union of observed issue sites. OSR has three observed sites; Multi_Paxos_3 has four, including the rare unhandled `accept` event.

| Strategy | OSR, seeds 0–19 | OSR, seeds 20–39 | Multi_Paxos_3, seeds 0–19 | Multi_Paxos_3, seeds 20–39 |
|---|---:|---:|---:|---:|
| New default: random feedback + checkpoints | 4/20 | 2/20 | 1/20 | 1/20 |
| POS + checkpoints, 100% of fresh generators | 17/20 | 19/20 | 5/20 | 3/20 |
| Existing plain POS | 20/20 | 20/20 | 3/20 | 4/20 |

The checkpoint/POS combination therefore improves OSR from **6/40 to 36/40** and Multi_Paxos_3 from **2/40 to 8/40**. Plain POS solves OSR more consistently, but its broader regressions make it unsuitable as a universal replacement.

State-entry checkpoints alone did not help consistently: on seeds 0–19 they completed OSR in 3/20 runs versus 4/20 for handler checkpoints, and Multi_Paxos_3 in 1/20 versus 1/20. In the initial screen, state-based Q-learning completed OSR in 8/20 and Multi_Paxos_3 in 0/20; the PCT variants completed OSR in 1–2/20 and Multi-Paxos in 0/20. The 25% mixed trial completed OSR in 11/20 and Multi-Paxos in 2/20. Negative results are retained.

## Broad check and an additional failure

I also measured all 495 runnable cases with seeds 0, 1, 2, 20, 21, 22, a 200-schedule / 1,000-step budget, and a 0.25-second safety cap. The candidate found an additional failure in **Multi_Paxos_4** on held-out seed **22**, schedule **47**:

```text
PaxosNode(#) received event 'PImplementation.accept' that cannot be handled.
```

This is a newly observed failure site in the experiment's model catalog, not a claim about a new production defect. The raw result retains its first discovery. Because it expands the catalog, I recomputed one union across both seed batches and all compared strategies before scoring:

| Strategy | Runs finding all observed sites, issue-bearing cases |
|---|---:|
| New default: random feedback + checkpoints | 854/870 |
| POS + checkpoints, 100% of fresh generators | 860/870 |
| Existing plain POS | 814/870 |

The new default scored 860/870 in the previous report because that earlier catalog did not contain the Multi_Paxos_4 accept site. Its lower score against this expanded catalog is not a change in its seeded behavior. The combined report contains every case, miss, bound, and time cap. No strategy in this follow-up is exhaustive, so none of these catalogs is certified complete.

The candidate remains experimental. It improves the targeted failures and discovers an additional site, but most Multi_Paxos_3 seeds still miss the rare accept path. A useful next experiment would allocate fresh partial-order runs when corpus mutations repeatedly return to known early failures, while preserving productive long prefixes. That adaptive policy has not been implemented or measured here.

## Reproduction

The research patch applies to PR commit `a013132c5469243af4b825c63ddc81c5c3744085`. JSON measurements were made in a dirty working tree and record the patch SHA-256. Historical screens predate unused POS-checkpoint support; their selected policies do not call it. Rust version: 1.99.1.

```sh
python3 scripts/bench-corpus.py \
  --strategies feedback-frontier feedback-state-frontier feedbackpos feedbackpct pos pct rl \
  --strategy-bound 2 --filter /OSR --seeds 20 --seconds-per-run 1 --out /tmp/screen-osr.json
python3 scripts/bench-corpus.py \
  --strategies feedback-frontier feedback-race-frontier pos --pos-percent 100 \
  --filter Multi_Paxos_3 --seed-start 20 --seeds 20 --seconds-per-run 1 --out /tmp/paxos-holdout.json
python3 scripts/bench-corpus.py \
  --strategies feedback-frontier feedback-race-frontier pos --pos-percent 100 \
  --seed-start 20 --seeds 3 --seconds-per-run 0.25 --out /tmp/corpus-holdout.json
```

Replace the filter with `/OSR` or omit it for the full corpus. Use seeds 0–19 for the development batch and 20–39 for the focused holdout batch. `feedback-frontier` explicitly selects the new default policy; `feedback-race-frontier` is the opt-in prototype. State-entry and POS sampling options remain off in ordinary checking. Automatic mode does not use the research policies.

- [Source patch](rare-event.patch).
- [Shared broad-corpus report](rare-corpus-shared.md) and [raw data](rare-corpus-shared.json.gz).
- Focused initial screens: [OSR](rare-rare-osr-screen.json.gz), [Multi-Paxos](rare-rare-paxos-screen.json.gz).
- State-entry trials: [OSR](rare-state-osr.json.gz), [Multi-Paxos](rare-state-paxos.json.gz).
- 25% fresh-POS trials: [OSR](rare-race-osr-trial.json.gz), [Multi-Paxos](rare-race-paxos-trial.json.gz).
- 100% fresh-POS trials: [OSR](rare-pos100-osr.json.gz), [Multi-Paxos](rare-pos100-paxos.json.gz).
- Focused holdouts: [OSR](rare-pos100-osr-holdout.json.gz), [Multi-Paxos](rare-pos100-paxos-holdout.json.gz).
- Individual broad passes: [seeds 0–2](rare-pos100-corpus.json.gz), [seeds 20–22](rare-pos100-corpus-holdout.json.gz).

Workspace tests and Clippy passed for the prototype. Additional tests verify reconstruction of POS priorities from a consumed checkpoint and replay of staged data/delivery failures using both random and POS fresh generators. Python corpus accounting and formatting checks passed.
