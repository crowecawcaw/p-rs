#!/usr/bin/env python3
"""Render the short-prompt rerun with the discovery diagnostic kept separate."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
s=json.loads((ROOT/'results/v2-path-comparison.json').read_text()); a=s['arms']; c=a['python']; p=a['p']
d=json.loads((ROOT/'results/v2-comparison.json').read_text())
lines=['# Short design-first prompt rerun\n',
'## Result\n',
f"With Snappea installed on normal PATH, **{p['primary_all_passed']}/6 P-assigned submissions** passed all twenty primary cases, versus **{c['primary_all_passed']}/6 Python-only submissions**. All runs used Luna with high thinking and the shortened prompts. This is a small, single-task exploratory result; it does not establish a general P advantage or a causal benefit from the wording change.\n",
'| Outcome | Python only | P/Snappea on PATH |','|---|---:|---:|',
f"| All 20 primary cases passed | {c['primary_all_passed']}/6 | {p['primary_all_passed']}/6 |",
f"| Primary cases passed (not independent samples) | {c['primary_cases_passed']}/120 | {p['primary_cases_passed']}/120 |",
f"| Separate wide-bundle test passed | {c['secondary_wide_passed']}/6 | {p['secondary_wide_passed']}/6 |",
f"| Median shell bridge calls | {c['median_tool_calls']} | {p['median_tool_calls']} |",
f"| Median seconds through final tool return | {c['median_seconds_through_last_tool']:.1f} | {p['median_seconds_through_last_tool']:.1f} |",
f"| Checker actually invoked | n/a | {p['p_checker_invoked']}/6 |",
f"| First model check before first Python implementation write | n/a | {p['p_check_before_solution']}/6 |",'',
'**The short wording did not produce an observed model-check-first workflow.** All six P agents wrote Python before their first P check. These are command/file-write observations; they do not reveal unobservable internal reasoning. The rerun supports a result about P assignment and actual tool use, not a clean experiment where a checked model must precede implementation.\n',
'## Exact task wording\n',
'Common: `Read TASK.md and contract.py. Write solution.py and DESIGN.md.`\n',
'P: `Design the certified change bundle service and model it with P/Snappea. Then implement it in Python.`\n',
'Python: `Design the certified change bundle service. Then implement it in Python. Do not use P/Snappea.`\n',
'The P prompt also lists `/opt/snappea`, `/opt/pdocs/`, and `/opt/inventory/`. Both arms have the same isolated shell bridge, 900-second/80-call limits, public task/interface/tests, and no parent conversation. There is no Python simulation/testing suggestion in the prompt. Every exact delegation is saved with its run.\n',
'## Setup correction and retained diagnostic cohort\n',
'The first six new P assignments had the executable mounted at `/opt/snappea` but not on PATH, matching the old setup. Only **1/6** invoked the checker. Three wrote P source and three supplied only prose descriptions. Two unexecuted P files failed independent post-freeze checks (one declared no test case; the other had a parser error). Some agents ran `which snappea`, got no PATH match, and incorrectly inferred the tool was unavailable despite listing the mounted executable.\n',
'Controller checks confirmed `/opt/snappea version` works in those container configurations. The setup was corrected to expose the executable on PATH. Six fresh P agents then received exactly the same short wording, apart from unique run IDs. All six invoked the checker and their final source passed independent checks after freezing. The original six Python controls were reused unchanged.\n',
'The original short-prompt P assignments are **retained separately**, not silently excluded or pooled. They scored 6/6 on the primary implementation suite too, despite only one actual checker user. That is a further reason not to attribute the primary score difference to model checking. The PATH change clearly changed observed tool use in these small cohorts; the implementation scores alone do not isolate a causal design benefit.\n',
'Evidence: [original short-prompt diagnostic](results/v2-comparison.json), [PATH preflight](results/v2-path-preflight.json), [corrected comparison](results/v2-path-comparison.json), [model checks](results/v2-path-model-review.json). No solver was told hidden results or repaired after submission. There were 18 new implementation runs: six controls, six initial P assignments, and six fresh PATH-corrected P assignments.\n',
'## Failures and remaining limits\n',
'Two Python controls failed primary cases: `v2-python-01` exceeded the published quiescent per-operation store-call budget, while `v2-python-03` returned the current digest on an original submit retry after revision. No PATH-corrected P submission failed a primary case. This does not prove the P implementations satisfy every contract behavior.\n',
'The separate valid wide-bundle test passes for 3/6 PATH-corrected P submissions and 0/6 controls. It uses a 27,565-byte update map spread over 64 branches, larger than one allowed 16 KiB record. The three successful P implementations chunk the relevant data. The remaining implementations still reject a valid representation boundary; the original primary score does not include this secondary test. This probe was specified before this rerun and was not revealed to agents.\n',
'Models remain small abstractions. Most checks focus on atomic publication/parent conflicts rather than the complete service or Python refinement. `path2-p-04` declares a broader revision/approval/CI monitor, but its driver exercises a prescribed happy path; submitted monitor event coverage is 33.3%. Other retained monitor reports show 100% for their much narrower modeled event sets. Raw event coverage is not full requirement coverage.\n',
'## Individual corrected comparison\n',
'| Run | Primary | Wide probe | Shell calls | Seconds | First Python write | First P check |',
'|---|---:|---|---:|---:|---:|---:|']
for r in s['trials']:
 lines.append(f"| {r['run']} | {r['primary_passed']}/20 | {'pass' if r['secondary_wide_passed'] else 'fail'} | {r['tool_calls']} | {r['seconds_through_last_tool']:.1f} | {r['first_solution_write_call']} | {r['first_p_check_call'] if r['first_p_check_call'] is not None else '—'} |")
lines += ['',
'## Grader audit\n',
'An independent Luna high-thinking audit found one overstrict assertion: after a crashed revise has durably reserved a log slot, a new call may legally help complete it and then return `stale`. The public contract promises ordinary linearizability, not rollback of every interrupted operation or identical revise retry responses. The old verifier required `updated`.\n',
'Grader v1.1.3 accepts `updated` or a helped-completion `stale`, then requires exact final revision 2, replacement updates, cleared approvals/CI, and unready publication. All 18 new frozen submissions were graded with the same corrected version and 64 seeds. The old rejection and correction are retained. No implementation was edited or coached. Final reports have no timeouts, and frozen artifact hashes were verified. See [correction record](results/v2-grader-correction.json).\n',
'## Interpretation\n',
'The concrete setup lesson is to install the CLI on PATH and preflight it, rather than relying on agents to interpret an arbitrary mounted path. This fixed actual checker use while preserving the short task wording.\n',
'The primary suite favors the P-assigned arm in this sample, but does not demonstrate that P caused the improvement: the diagnostic P arm without meaningful tool use scored just as well, the sample is small, controls were reused, and the agents still checked models after beginning Python. To measure model-first design specifically, a future treatment must require checking the model before writing Python and capture that checkpoint. The current prompt is unchanged.\n',
'Exact prompts, logs, artifacts and grades are under runs/. Reproduce data with `tools/analyze_v2.py`, `tools/analyze_path2.py`, and the commands in README.md. Original v1 results remain in RESULTS.md.\n']
(ROOT/'RESULTS_V2.md').write_text('\n'.join(lines));print('Wrote RESULTS_V2.md')
