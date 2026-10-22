# Certified bundle design eval

Run commands below from `evals/certified-bundles/`.

A runnable Python implementation benchmark comparing agents with and without
P/Snappea. The task is a protected multi-repository VCS service: revision-bound
approvals/CI, atomic publication, overlapping requests, durable request outcomes,
and crash recovery. No skill is included.

**The original v1 comparison used 12 independent solver submissions, six per arm.** The latest short-prompt rerun is in [RESULTS_V2.md](RESULTS_V2.md). See [RESULTS.md](RESULTS.md) for the original results and interpretation, and
[results/luna-comparison.json](results/luna-comparison.json) for machine-readable data.

## Short-prompt rerun

The rerun is complete: see [RESULTS_V2.md](RESULTS_V2.md). The corrected comparison uses six new Python controls and six fresh P agents with Snappea on PATH. Six earlier short-prompt P discovery runs are retained separately. Data: results/v2-path-comparison.json; discovery diagnostic: results/v2-comparison.json. All use Luna high thinking.

## Files

- `public/TASK.md`: full requirements, input domain, storage constraints, deliverables.
- `public/contract.py`: Python interface and candidate digest.
- `public/runtime.py`: durable single-key CAS with seeded cooperative scheduling.
- `public/test_public.py`: four public interface/behavior examples.
- `private/oracle.py`: independent state machine and linearizability checker.
- `private/grade.py`: twenty primary cases: certification, idempotency, concurrent
  histories, crash sweeps, generated traces, record growth, quiescent progress.
- `private/stress_wide.py`: separate exploratory valid-input probe: a 27,565-byte,
  64-branch update map that must be represented across multiple 16 KiB records.
- `private/reference.py`: calibration implementation; never supplied to agents.
- `tools/collaboration_run.py`: provision/freeze/grade collaboration-agent trials.
- `tools/prompts.py`: current short, design-first task prompts (v2).
- `tools/delegation.py`: current isolated solver instructions; forked with no parent history. Completed v1 instructions are saved in each run/delegation.txt and results/prompt-v1/.
- `tools/solver_shell.py`: budgeted/logged bridge into network-disabled containers.
- `tools/run_sonnet.py`: optional Anthropic API runner, not used for Luna.
- `tools/analyze_luna.py`: primary score, command/time metrics, model coverage,
  and supplemental probe outcomes, kept separate.
- `tools/review_models.py`: post-freeze model mutations and reachability probes.
- `tools/calibrate.py`: nine known-wrong implementations; all are rejected.
- `tools/test_harness.py`: seven checks of storage, oracle, and report behavior.

## Reproduce frozen-submission results

Use Python 3.11+ and Docker; no Python packages are required. The pinned Python
3.12 container image is specified in tools/run_sonnet.py. Pull it beforehand if
needed. Before fresh P runs or model rechecks, stage the checker:

```sh
python tools/bootstrap.py
# Or use a local build and explicitly select its docs/examples:
python tools/bootstrap.py --binary ../../target/release/snappea --source ../..
```

The default downloads the checksum-verified Linux x86-64 v0.1.0 release. It links
docs/examples from this checkout; use `--source` to select a historical checkout.
`vendor/provenance.json` records the original pilot binary/source archive hashes.
The original source archive was downloaded from moving main and its exact commit
was not recorded, so reconstructing that original docs snapshot is not guaranteed.
Frozen submissions, prompts, transcripts, grades and aggregate data are included;
binaries, duplicate working copies, smoke runs and source archives are excluded.
The large v1 transcript is losslessly gzip-compressed; analysis scripts read both
formats. Historical delegation paths identify the original environment; generate
a fresh delegation with `start` instead of reusing those absolute paths.

```sh
python tools/test_harness.py -v
python private/grade.py private/reference.py --seeds 64 --output results/reference-current.json
python tools/regrade.py luna-python-01 luna-p-01 luna-python-02 luna-p-02 luna-python-03 luna-p-03 luna-python-04 luna-p-04 luna-python-05 luna-p-05 luna-python-06 luna-p-06 --seeds 64
python tools/stress_all.py
python tools/review_models.py
python tools/analyze_luna.py
```

The original comparison used grader v1.1.2. The current grader is v1.1.3, which permits a legal helped completion of an interrupted revise; see RESULTS_V2.md. Grading allows 180 seconds per case so a
storage layout with many internal boundaries is not penalized merely for making
crash sweeping expensive. The service itself still has the published per-operation
limit of 5,000 store calls in quiescence. A typo in `--only` cannot report an empty
success. A broken submission import produces a scored failure report.

Each final grade includes hashes of the grader, runtime, and oracle. Original
pre-correction reports and grader snapshots are retained under runs/*/grading and
results/harness-v*. Generated schedules are bounded tests, not correctness proofs.

## Run fresh Luna agents

The controller provisions an isolated workspace for each independent solver:

```sh
python tools/collaboration_run.py start --run-id luna-python-07 --arm python
python tools/collaboration_run.py start --run-id luna-p-07 --arm p
```

Using the collaboration tool, spawn each with `model="gpt-6-luna"`,
`reasoning_effort="high"`, `fork_turns="none"`, and its saved
`runs/<run-id>/delegation.txt` as the message. Do not pass this README, hidden tests,
reference implementation, prior results, or other solver context to the solver.
The delegation instructs the agent to execute every command through the bridge;
only its own public workspace and (for the P arm) P docs/example/binary are mounted.

After its final response, freeze and grade:

```sh
python tools/collaboration_run.py finish --run-id luna-python-07 --seeds 64
python tools/collaboration_run.py finish --run-id luna-p-07 --seeds 64
```

Never send hidden feedback back to a solver. Alternate arm launch order; use the
same task/version/budgets for both. The current comparison used 900 seconds from
first command and 80 shell calls per solver, with no parent context. Token usage
is not exposed by the collaboration backend and was not measured or fabricated.
Shell transcripts, final submissions, checksums, delegated prompts and model/
thinking settings are saved. Full internal model reasoning is not available.

Containers run with host UID/GID, no Linux capabilities, read-only root, resource
limits and no network. Collaboration agents are instructed to use only the bridge;
the collaboration tool itself does not revoke their broader host-tool capabilities.
This is research isolation, not a hostile-code security boundary. No credentials
enter the solver containers. The grader assumes compliance with the public store/
I/O restrictions and is not hardened against malicious Python submissions.

## Measurement and interpretation

Primary measure: submissions passing all twenty primary cases. Test cases and
schedule seeds are not independent samples: the independent unit is a solver run.
The two cohorts are recorded separately: the first six submissions were regraded
after correcting a harness cap; the second six saw the corrected public runtime.
No implementation was edited after submission, and no solver received hidden
feedback. The wide-bundle test was motivated after inspecting submissions, so it
is explicitly supplemental, not silently folded into the primary result.

Elapsed time means first tool call through last tool return, excluding controller
freeze/grade delays. It includes solver work during that interval. Command counts
are shell bridge calls; a call may contain several commands. Timing is descriptive
and can be affected by shared compute and scheduling. The suite tests explicit P
use, not spontaneous tool discovery.

Always inspect P model relevance, reachability, and requirements covered. A
successful checker exit does not establish that a monitor ran, that a model covers
the whole task, or that Python implements the model. The comparison includes the
vacuous-model assignment in the P arm rather than silently excluding it.

After the cohort, the bridge was improved to return bounded stdout/stderr with byte
counts while retaining full streams on disk. All twelve solver runs used bridge
v1; future runs use v2. A snapshot of v1 is in results/bridge-v1. Do not mix future
cohorts into this result without recording that instrumentation change.

## Optional Sonnet backend

Not used in this comparison. Configure ANTHROPIC_API_KEY through normal secret
configuration; do not put it in the repository. Choose an available exact model ID:

```sh
python tools/run_sonnet.py --run-id sonnet-python-01 --arm python --model claude-sonnet-4-6
python tools/run_sonnet.py --run-id sonnet-p-01 --arm p --model claude-sonnet-4-6
```

The API adapter has scripted protocol/isolation smoke coverage but has not been
verified with a live Anthropic response in this session. Its token-budget setting
is distinct from the collaboration backend's time/call budgets.

## Prompt revision v2

New runs use short design-first prompts. The P arm asks for a design and P/Snappea model before Python implementation. The control asks for a design then Python implementation, with no suggestion to simulate or test in Python. Operational isolation and budgets are shared. The original results used v1. The v2 comparison has now run; see RESULTS_V2.md. The P executable is installed on normal PATH and controller provisioning now requires a successful `snappea version` preflight.

## Reproduce the v2 data

```sh
python tools/analyze_v2.py
python tools/stress_all.py --prefix path2-p- --output results/v2-path-wide-stress.json
python tools/review_v2_models.py --prefix path2-p- --output results/v2-path-model-review.json
python tools/analyze_path2.py
python tools/write_results_v2.py
```

The original Python controls are v2-python-01..06. The fresh PATH-corrected P runs are path2-p-01..06. Initial v2-p-01..06 are a separate discovery diagnostic. Do not pool these treatment cohorts or count reused controls twice. New provisioning always checks that `snappea` resolves and executes on PATH. Prompt wording remains v2.
