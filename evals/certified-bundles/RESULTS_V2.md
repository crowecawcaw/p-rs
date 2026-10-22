# Short design-first prompt rerun

## Result

With Snappea installed on normal PATH, **6/6 P-assigned submissions** passed all twenty primary cases, versus **4/6 Python-only submissions**. All runs used Luna with high thinking and the shortened prompts. This is a small, single-task exploratory result; it does not establish a general P advantage or a causal benefit from the wording change.

| Outcome | Python only | P/Snappea on PATH |
|---|---:|---:|
| All 20 primary cases passed | 4/6 | 6/6 |
| Primary cases passed (not independent samples) | 117/120 | 120/120 |
| Separate wide-bundle test passed | 0/6 | 3/6 |
| Median shell bridge calls | 4.5 | 16.0 |
| Median seconds through final tool return | 44.8 | 111.1 |
| Checker actually invoked | n/a | 6/6 |
| First model check before first Python implementation write | n/a | 0/6 |

**The short wording did not produce an observed model-check-first workflow.** All six P agents wrote Python before their first P check. These are command/file-write observations; they do not reveal unobservable internal reasoning. The rerun supports a result about P assignment and actual tool use, not a clean experiment where a checked model must precede implementation.

## Exact task wording

Common: `Read TASK.md and contract.py. Write solution.py and DESIGN.md.`

P: `Design the certified change bundle service and model it with P/Snappea. Then implement it in Python.`

Python: `Design the certified change bundle service. Then implement it in Python. Do not use P/Snappea.`

The P prompt also lists `/opt/snappea`, `/opt/pdocs/`, and `/opt/inventory/`. Both arms have the same isolated shell bridge, 900-second/80-call limits, public task/interface/tests, and no parent conversation. There is no Python simulation/testing suggestion in the prompt. Every exact delegation is saved with its run.

## Setup correction and retained diagnostic cohort

The first six new P assignments had the executable mounted at `/opt/snappea` but not on PATH, matching the old setup. Only **1/6** invoked the checker. Three wrote P source and three supplied only prose descriptions. Two unexecuted P files failed independent post-freeze checks (one declared no test case; the other had a parser error). Some agents ran `which snappea`, got no PATH match, and incorrectly inferred the tool was unavailable despite listing the mounted executable.

Controller checks confirmed `/opt/snappea version` works in those container configurations. The setup was corrected to expose the executable on PATH. Six fresh P agents then received exactly the same short wording, apart from unique run IDs. All six invoked the checker and their final source passed independent checks after freezing. The original six Python controls were reused unchanged.

The original short-prompt P assignments are **retained separately**, not silently excluded or pooled. They scored 6/6 on the primary implementation suite too, despite only one actual checker user. That is a further reason not to attribute the primary score difference to model checking. The PATH change clearly changed observed tool use in these small cohorts; the implementation scores alone do not isolate a causal design benefit.

Evidence: [original short-prompt diagnostic](results/v2-comparison.json), [PATH preflight](results/v2-path-preflight.json), [corrected comparison](results/v2-path-comparison.json), [model checks](results/v2-path-model-review.json). No solver was told hidden results or repaired after submission. There were 18 new implementation runs: six controls, six initial P assignments, and six fresh PATH-corrected P assignments.

## Failures and remaining limits

Two Python controls failed primary cases: `v2-python-01` exceeded the published quiescent per-operation store-call budget, while `v2-python-03` returned the current digest on an original submit retry after revision. No PATH-corrected P submission failed a primary case. This does not prove the P implementations satisfy every contract behavior.

The separate valid wide-bundle test passes for 3/6 PATH-corrected P submissions and 0/6 controls. It uses a 27,565-byte update map spread over 64 branches, larger than one allowed 16 KiB record. The three successful P implementations chunk the relevant data. The remaining implementations still reject a valid representation boundary; the original primary score does not include this secondary test. This probe was specified before this rerun and was not revealed to agents.

Models remain small abstractions. Most checks focus on atomic publication/parent conflicts rather than the complete service or Python refinement. `path2-p-04` declares a broader revision/approval/CI monitor, but its driver exercises a prescribed happy path; submitted monitor event coverage is 33.3%. Other retained monitor reports show 100% for their much narrower modeled event sets. Raw event coverage is not full requirement coverage.

## Individual corrected comparison

| Run | Primary | Wide probe | Shell calls | Seconds | First Python write | First P check |
|---|---:|---|---:|---:|---:|---:|
| v2-python-01 | 19/20 | fail | 5 | 45.4 | 4 | — |
| v2-python-02 | 20/20 | fail | 6 | 78.6 | 3 | — |
| v2-python-03 | 18/20 | fail | 4 | 44.3 | 4 | — |
| v2-python-04 | 20/20 | fail | 3 | 41.6 | 3 | — |
| v2-python-05 | 20/20 | fail | 6 | 45.7 | 4 | — |
| v2-python-06 | 20/20 | fail | 3 | 39.7 | 3 | — |
| path2-p-01 | 20/20 | pass | 15 | 98.0 | 6 | 14 |
| path2-p-02 | 20/20 | fail | 17 | 110.9 | 5 | 12 |
| path2-p-03 | 20/20 | fail | 15 | 119.1 | 6 | 10 |
| path2-p-04 | 20/20 | pass | 17 | 111.3 | 6 | 14 |
| path2-p-05 | 20/20 | pass | 22 | 131.7 | 5 | 7 |
| path2-p-06 | 20/20 | fail | 13 | 83.1 | 7 | 10 |

## Grader audit

An independent Luna high-thinking audit found one overstrict assertion: after a crashed revise has durably reserved a log slot, a new call may legally help complete it and then return `stale`. The public contract promises ordinary linearizability, not rollback of every interrupted operation or identical revise retry responses. The old verifier required `updated`.

Grader v1.1.3 accepts `updated` or a helped-completion `stale`, then requires exact final revision 2, replacement updates, cleared approvals/CI, and unready publication. All 18 new frozen submissions were graded with the same corrected version and 64 seeds. The old rejection and correction are retained. No implementation was edited or coached. Final reports have no timeouts, and frozen artifact hashes were verified. See [correction record](results/v2-grader-correction.json).

## Interpretation

The concrete setup lesson is to install the CLI on PATH and preflight it, rather than relying on agents to interpret an arbitrary mounted path. This fixed actual checker use while preserving the short task wording.

The primary suite favors the P-assigned arm in this sample, but does not demonstrate that P caused the improvement: the diagnostic P arm without meaningful tool use scored just as well, the sample is small, controls were reused, and the agents still checked models after beginning Python. To measure model-first design specifically, a future treatment must require checking the model before writing Python and capture that checkpoint. The current prompt is unchanged.

Exact prompts, logs, artifacts and grades are under runs/. Reproduce data with `tools/analyze_v2.py`, `tools/analyze_path2.py`, and the commands in README.md. Original v1 results remain in RESULTS.md.
