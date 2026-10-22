# Luna high-thinking design eval

## Result

On the original twenty-case suite, Python-only passed **5/6** submissions and P-assisted passed **4/6**. This pilot did not demonstrate an advantage from adding P. It is one task with six independent runs per arm; it does not establish a general effect in either direction.

| Measure | Python only | Python + P/Snappea |
|---|---:|---:|
| Submissions passing all 20 primary cases | 5/6 | 4/6 |
| Primary cases passed (descriptive, not independent samples) | 119/120 | 116/120 |
| Median shell bridge calls | 5.5 | 13.5 |
| Median seconds, first command through final command return | 43.2 | 90.2 |
| Supplemental wide-bundle probe passed | 1/6 | 0/6 |

Timing and command counts are descriptive. A bridge call can contain multiple commands. Token use is unavailable for collaboration subagents. All runs used gpt-6-luna with high thinking, fresh sessions without parent history, a 900-second/80-call maximum, and the same interface and public requirements. No solver received hidden feedback.

## The most useful finding: successful model checking can be vacuous

`luna-p-05` reported successful random and exhaustive checks, including all 52 exhaustive schedules. Its commit handler is unreachable: the store sends `eCommit` to the Publisher, which transitions to Done without sending it back. The store never commits or announces the event observed by its monitor.

The submitted coverage report already says **AtomicPublication event coverage: 0.0%**, with `eSnapshot` uncovered. None of the six P solvers inspected coverage.txt. An independent post-freeze probe replaced a commit-path assertion with `assert false`; exhaustive checking still passed all 52 schedules. A separate Luna high-thinking audit confirmed the event-routing error.

Evidence: [submitted model](runs/luna-p-05/submission/bundle_model.p), [submitted coverage](runs/luna-p-05/submission/PCheckerOutput/tcAtomicBundlePublish/coverage.txt), [reproducible probes](results/model-probes.json). The Python implementation in this run passes the primary suite; the invalid evidence claim is separate from its implementation score. The run remains in the P-assigned denominator.

## Models omitted requirements that failed in Python

Two P-assisted submissions, `luna-p-03` and `luna-p-06`, return the current revision digest when an original submit request is retried after revision. The contract requires the original response. Both pass their P checks because their models omit submit/revision semantics. Each fails two primary cases for this one underlying bug.

The Python-only failure, `luna-python-06`, replays an already-published bundle under another request ID and moves a branch back after a later commit. It fails the published-bundle replay case.

All six P models abstract away dynamic revision-bound approvals/CI and original submit-response replay. They mostly model an atomic commit pointer or competing publishes against a fixed initial base. Five models have a property triggered by at least one targeted mutation; this is useful evidence of non-vacuity for that property, not full requirement coverage. In `luna-p-04`, corrupting one branch is caught, but accepting stale CAS versions still passes the submitted properties.

## Separate, exploratory representation-boundary probe

A post-pilot probe submits one valid 64-branch bundle with a 27,565-byte update map against a store whose individual records are limited to 16,384 bytes. Commit IDs remain valid opaque strings under TASK.md. Eleven of twelve implementations fail because they place a complete event, bundle, or page entry in one record. Only `luna-python-04` passes. The reference implementation passes.

This was added after inspecting submissions and is **not silently folded into the original primary score**. It shows a missing part of the suite and representation design, rather than a deep concurrency result. If the intended domain restricts commit IDs to fixed-length hashes, specify that bound and adjust this probe explicitly. Detailed results: [wide-stress.json](results/wide-stress.json).

## Individual results

| Submission | Cohort | Primary cases | Shell calls | Seconds through final command | Wide probe |
|---|---|---:|---:|---:|---|
| luna-p-01 | pilot_regraded | 20/20 | 22 | 103.1 | fail |
| luna-p-02 | pilot_regraded | 20/20 | 13 | 56.1 | fail |
| luna-p-03 | pilot_regraded | 18/20 | 13 | 70.5 | fail |
| luna-p-04 | runtime_fixed | 20/20 | 14 | 94.5 | fail |
| luna-p-05 | runtime_fixed | 20/20 | 13 | 86.0 | fail |
| luna-p-06 | runtime_fixed | 18/20 | 23 | 141.3 | fail |
| luna-python-01 | pilot_regraded | 20/20 | 2 | 27.7 | fail |
| luna-python-02 | pilot_regraded | 20/20 | 7 | 39.9 | fail |
| luna-python-03 | pilot_regraded | 20/20 | 5 | 42.4 | fail |
| luna-python-04 | runtime_fixed | 20/20 | 5 | 44.1 | pass |
| luna-python-05 | runtime_fixed | 20/20 | 7 | 55.4 | fail |
| luna-python-06 | runtime_fixed | 19/20 | 6 | 45.0 | fail |

The first cohort produced Python-only 3/3 versus P-assisted 2/3 after regrading. The second cohort, which saw the corrected runtime, produced 2/3 in each arm. The aggregate one-run difference is not a reliable estimate of treatment effect.

## Concrete Snappea changes these runs motivate

| Surface | Observed evidence | Next change to evaluate |
|---|---|---|
| Successful CLI output | P05 passes despite 0% monitor event coverage | Show uncovered monitors in the final summary; add a strict coverage/reachability mode for CI |
| Starting a model | P01 repeatedly has no declared test; P02 writes malformed project XML; several runs confuse machines and spec monitors | A minimal working starter command/template, with direct-file and project examples |
| Runtime errors | P03 receives an upstream-style map KeyNotFoundException and then adds membership guards | Include P source location and an actionable map lookup explanation |
| Verbosity | P06 requests a time-bounded verbose check, emits 51,975,114 stdout bytes in 3.135 seconds, and exits 137 in the resource-limited container | Separate progress verbosity from full schedule traces; bound or redirect full traces |
| Design evidence | Models pass without covering revision, certification, or retry requirements | Report requirement-to-property links and review model mutation coverage |

Exit 137 does not by itself establish why the verbose run was killed; no OOM diagnosis is claimed. Existing help does describe verbose schedule logging, but the agent still chose it. These observations are stronger evidence for scaffolding, coverage surfacing, and output controls than for renaming individual command flags.

## Harness corrections and verification

- Removed an accidental cumulative 250,000-boundary cap, which contradicted the published per-operation budget and penalized valid event-log designs. Regraded all frozen submissions; no solver was coached or repaired.
- Increased grading time per case to 180 seconds: an exhaustive crash sweep for a deep immutable-trie implementation exceeded 45 seconds. The final comparison has no timeout failures. This grading allowance is separate from solver and service budgets.
- Corrected input-alias testing so the actual argument object is mutated, rather than a detached copy.
- An unmatched --only filter now fails rather than reporting all([]) success. Broken imports and unexpected Crash exceptions produce scored reports.
- Seven harness regression checks pass; the calibration reference passes; all nine seeded flawed implementations are rejected.
- Every primary result is produced by grader v1.1.2 using 64 schedule/generation seeds in seeded cases. Grade reports contain source hashes. Earlier reports and grader snapshots are retained.
- After all solvers finished, bounded bridge output was added and smoke-tested; full stdout/stderr remain on disk. This is instrumentation for future runs, not a silent change within this cohort.

## Interpretation and next experiment

The eval now distinguishes implementation correctness, cost of using P, and quality of the claimed model evidence. The immediate product experiment is to surface zero-coverage monitors in successful check output, then rerun fresh P-assigned agents and measure whether they repair the vacuous model. A second experiment can test a working model starter against current docs/errors. Keep those treatment changes separate so their effects are interpretable.

This task permits a shared CAS commit pointer, and agents generally use that to serialize the logical state machine. That is a valid design choice under the contract and reduces the need for a sophisticated protocol. To investigate P on harder coordination, add a separately versioned task with a real operational constraint such as coordination across independently owned repositories. Do not forbid a correct simple design merely to force a P win.

The suite is finite, crash sweeps and concurrency histories are bounded, and concurrency-plus-crash combinations are not comprehensively explored. Collaboration isolation is enforced for executed container commands and instructed at the agent-tool level, not a hostile-agent sandbox. The pilot measures instructed tool use, not whether an agent independently discovers Snappea. No skill was added.

## Reproduce

See [README.md](README.md). Frozen implementations, actual public files, delegated prompts, command streams, model artifacts, coverage reports, source checksums, and hidden grade reports are under runs/. [luna-comparison.json](results/luna-comparison.json) is the main data file. All model probes are reproduced with tools/review_models.py; supplemental stress is reproduced with tools/stress_all.py.
