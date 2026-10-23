Measured 1 cases across 1 project inputs; 1 cases have observed issues.
Revision: `b33f3ad408073f046ee6ab1c3866cf45ae201bed`; working tree dirty: True.
Budget: 2000 schedules, 1000 steps, 3 seed(s), 2.0s safety cap per run.
Completed exhaustive catalogs: 0; expected DynamicError cases with no common issue observed: 0.

**All issues means the union observed across every strategy and seed in this run.**
Only a completed unbounded exhaustive search certifies the catalog. A blank catalog otherwise means unknown, not bug-free.
Counts are first-discovery schedule indexes, computed after the shared catalog is formed; successful schedules count too.
Temperature-based potential liveness warnings are recorded separately and excluded from the common issue catalog: unfair DFS does not perform that check.

| Strategy | Runs finding all observed issues (issue-bearing cases) |
|---|---:|
| random | 0/3 |
| feedback | 2/3 |
| feedback-frontier | 1/3 |
| dfs | 0/3 |
| auto | 0/3 |

| Case | Known issues | Potential liveness | Catalog complete | random | feedback | feedback-frontier | dfs | auto |
|---|---:|---:|---|---:|---:|---:|---:|---:|
| tests/upstream/RegressionTests/Integration/DynamicError/Multi_Paxos_3::DefaultImpl | 4 | 0 | no | miss 3/4 (6000 tested) | 1005 (2/3) | 46 (1/3) | miss 1/4 (6000 tested) | miss 2/4 (6000 tested) |

0 project inputs excluded, unsupported, or interrupted:


Cells show median schedules among completed seeds plus completed/attempted runs. A miss retains the number of issues found and schedules actually tested.
Time caps, engine errors, unrun rows, step-bound hits and missing foreign implementations are retained in the JSON. This quick pass does not establish that unseen issues do not exist.
