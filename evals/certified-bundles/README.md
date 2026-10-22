# Certified bundle design eval

A runnable Python implementation benchmark comparing agents with and without
P/Snappea. The task is a protected multi-repository VCS service: revision-bound
approvals/CI, atomic publication, overlapping requests, durable request outcomes,
and crash recovery. No skill is included.

This directory contains the eval setup. Run outputs, submissions, transcripts,
checker output, and generated reports stay local and are ignored by Git.
Run commands below from `evals/certified-bundles/`.

## Files

- `public/`: task requirements, Python interface, durable single-key CAS runtime,
  and public behavior examples.
- `private/oracle.py`: independent state machine and linearizability checker.
- `private/grade.py`: twenty primary cases covering certification, idempotency,
  concurrent histories, crash sweeps, generated traces, record growth, and progress.
- `private/stress_wide.py`: separate exploratory valid-input probe with a
  64-branch update map that must span multiple 16 KiB records.
- `private/reference.py`: calibration implementation; never supplied to agents.
- `private/calibration-p/`: source model contrasting split and atomic publication.
- `tools/calibrate.py`: nine known-wrong implementations the grader must reject.
- `tools/test_harness.py`: storage, oracle, and report self-checks.
- `tools/collaboration_run.py`: provision, freeze, and grade isolated solver trials.
- `tools/prompts.py`, `tools/delegation.py`: shared task prompts and solver instructions.
- `tools/solver_shell.py`: budgeted, logged bridge into network-disabled containers.
- `tools/run_sonnet.py`: optional Anthropic API runner.
- `tools/regrade.py`, `tools/stress_all.py`, `tools/summarize.py`: grade and summarize
  selected local runs without counting calibration or smoke runs as agent trials.

## Check the harness

Use Python 3.11+. No Python packages are required for local harness checks:

```sh
python tools/test_harness.py -v
python private/grade.py private/reference.py --seeds 64 --output results/reference.json
python tools/calibrate.py
```

The current grader is v1.1.3. Each grade records hashes of the grader, runtime, and
oracle. Cases have a 180-second timeout; the public per-operation limit remains
5,000 store calls in quiescence. Generated schedules are bounded tests, not proofs.
The reference calibrates the primary suite; it is not a complete solution to the
separate exploratory wide-input probe.

## Prepare isolated runs

Solver runs require Docker and the pinned Python 3.12 image specified in
`tools/run_sonnet.py`. Pull that image before starting network-disabled containers.
For P runs, stage the checker and docs:

```sh
python tools/bootstrap.py
# Or select a local build and checkout:
python tools/bootstrap.py --binary ../../target/release/snappea --source ../..
```

The default downloads the checksum-verified Linux x86-64 v0.1.0 release and links
this checkout's docs/examples. Use `--source` to select a pinned checkout and record
its commit when comparing runs. Downloaded binaries and staged sources are ignored.
P provisioning checks that `snappea` resolves and executes on PATH.

## Run independent agents

```sh
python tools/collaboration_run.py start --run-id luna-python-01 --arm python
python tools/collaboration_run.py start --run-id luna-p-01 --arm p
```

Using the collaboration tool, spawn each solver with `model="gpt-6-luna"`,
`reasoning_effort="high"`, `fork_turns="none"`, and its generated
`runs/<run-id>/delegation.txt` as the message. Do not pass this README, hidden tests,
the reference implementation, or other solver context to the solver. Only its
public workspace and (for the P arm) checker, docs, and examples are mounted.

After each solver's final response, freeze and grade:

```sh
python tools/collaboration_run.py finish --run-id luna-python-01 --seeds 64
python tools/collaboration_run.py finish --run-id luna-p-01 --seeds 64
python tools/summarize.py --prefix luna- --output results/summary.json
# Optional: regrade frozen submissions and run the separate exploratory probe.
python tools/regrade.py luna-python-01 luna-p-01 --seeds 64
python tools/stress_all.py --prefix luna- --output results/wide-stress.json
```

Never send hidden feedback back to a solver. Alternate arm launch order and use
identical task versions and budgets. The bridge allows 900 seconds from the first
command and 80 shell calls per solver. Token usage is not exposed by the
collaboration backend. Prompts, metadata, transcripts, frozen submissions, checksums,
and grades are generated under `runs/`; aggregates and calibration outputs go under
`results/`. Both directories are ignored, including newly generated submissions.

Containers use host UID/GID, no Linux capabilities, a read-only root, resource
limits, and no network. Agent instructions restrict commands to the bridge, but
do not revoke broader host-tool access. This is research isolation, not a
hostile-code security boundary. The grader assumes compliance with public store
and I/O restrictions.

## Interpret measurements

The primary measure is submissions passing all twenty primary cases. The
independent unit is a solver run; cases and schedule seeds are not independent
samples. Keep exploratory wide-input probes separate from the primary score.
Inspect P model relevance, monitor reachability, and requirements covered: a
successful checker exit alone does not establish coverage or implementation fidelity.
Record prompt, harness, checker, docs, and bridge versions for each comparison.

## Optional Sonnet backend

Configure `ANTHROPIC_API_KEY` through normal secret configuration and choose an
available exact model ID:

```sh
python tools/run_sonnet.py --run-id sonnet-python-01 --arm python --model claude-sonnet-4-6
python tools/run_sonnet.py --run-id sonnet-p-01 --arm p --model claude-sonnet-4-6
```

`python tools/test_runner_protocol.py` exercises the API loop, submission freezing,
and container grading with scripted responses and no live API call. Its outputs
are local smoke artifacts, excluded from agent comparisons.

## Compare checker output with one extra sentence

Prompt version 3 shares “Design and implement the service in Python.” The P arm
adds “First, model it with P/Snappea and check what could go wrong.” Tool paths
are supplied separately; there is no extra testing advice for either arm.

To compare old and new output, use three independent cohorts: Python-only, P
with the baseline binary, and P with the modified binary. Keep both P cohorts'
docs/examples identical. Provision each with explicit binary/source overrides:

```sh
SNAPPEA_EVAL_BINARY=/absolute/path/to/baseline/snappea SNAPPEA_EVAL_SOURCE=/absolute/path/to/shared-source python tools/collaboration_run.py start --run-id pilot-base-01 --arm p
SNAPPEA_EVAL_BINARY=/absolute/path/to/modified/snappea SNAPPEA_EVAL_SOURCE=/absolute/path/to/shared-source python tools/collaboration_run.py start --run-id pilot-feedback-01 --arm p
python tools/collaboration_run.py start --run-id pilot-python-01 --arm python
```

Use fresh Luna high agents with generated delegations as above. Binary hashes
and docs Git revision are recorded in each P run's metadata. Freeze/grade before
inspecting hidden results. Summarize each cohort separately with `--prefix`;
inspect first model exploration versus first implementation write and whether
coverage feedback causes scenario changes. Handler coverage and distinct abstract
timelines measure the P model's exploration, not Python correctness or requirement
coverage. Preserve negative results and keep wide-input probes separate.

## Automatic reachability research probe

```sh
python tools/probe_reachability.py --binary vendor/bin/snappea-v0.1.0-x86_64-unknown-linux-gnu/snappea --output-dir results/reachability-probe
```

This copies two small calibration fixtures and automatically inserts a failing
sentinel in each commit handler. The known vacuous handler remains unexercised;
the exercised monitor fails with its unique marker. The JSON report separates
invalid variants, timeouts, other failures and markers reached within budget.
This helper is a targeted research experiment, not a general semantic mutation
feature in `snappea check`. The implementation proposal and current diversity
signals are documented in [the investigation](../../design/check-quality/mutation-checking.md).
