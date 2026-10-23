Measured 6 cases across 1 project inputs; 6 cases have observed issues.
Revision: `b33f3ad408073f046ee6ab1c3866cf45ae201bed`; working tree dirty: True.
Budget: 200 schedules, 1000 steps, 20 seed(s), 1.0s safety cap per run.
Completed exhaustive catalogs: 3; expected DynamicError cases with no common issue observed: 0.

**All issues means the union observed across every strategy and seed in this run.**
Only a completed unbounded exhaustive search certifies the catalog. A blank catalog otherwise means unknown, not bug-free.
Counts are first-discovery schedule indexes, computed after the shared catalog is formed; successful schedules count too.
Temperature-based potential liveness warnings are recorded separately and excluded from the common issue catalog: unfair DFS does not perform that check.

| Strategy | Runs finding all observed issues (issue-bearing cases) |
|---|---:|
| random | 0/120 |
| feedback | 34/120 |
| feedback-frontier | 120/120 |
| dfs | 120/120 |
| auto | 120/120 |

| Case | Known issues | Potential liveness | Catalog complete | random | feedback | feedback-frontier | dfs | auto |
|---|---:|---:|---|---:|---:|---:|---:|---:|
| crates/snappea/tests/models/guided_prefix.p::tcDataPrefix___depth_12 | 1 | 0 | yes | miss 0/1 (4000 tested) | 82 (20/20) | 22.5 (20/20) | 13 (20/20) | 7 (20/20) |
| crates/snappea/tests/models/guided_prefix.p::tcDataPrefix___depth_20 | 1 | 0 | yes | miss 0/1 (4000 tested) | 153.5 (10/20) | 40.5 (20/20) | 21 (20/20) | 10 (20/20) |
| crates/snappea/tests/models/guided_prefix.p::tcDataPrefix___depth_24 | 1 | 0 | yes | miss 0/1 (4000 tested) | 153.5 (4/20) | 51.5 (20/20) | 25 (20/20) | 13 (20/20) |
| crates/snappea/tests/models/guided_prefix.p::tcSchedulePrefix___depth_12 | 1 | 0 | no | miss 0/1 (4000 tested) | miss 0/1 (4000 tested) | 56 (20/20) | 49 (20/20) | 31.5 (20/20) |
| crates/snappea/tests/models/guided_prefix.p::tcSchedulePrefix___depth_20 | 1 | 0 | no | miss 0/1 (4000 tested) | miss 0/1 (4000 tested) | 98.5 (20/20) | 81 (20/20) | 53 (20/20) |
| crates/snappea/tests/models/guided_prefix.p::tcSchedulePrefix___depth_24 | 1 | 0 | no | miss 0/1 (4000 tested) | miss 0/1 (4000 tested) | 121.5 (20/20) | 97 (20/20) | 63 (20/20) |

0 project inputs excluded, unsupported, or interrupted:


Cells show median schedules among completed seeds plus completed/attempted runs. A miss retains the number of issues found and schedules actually tested.
Time caps, engine errors, unrun rows, step-bound hits and missing foreign implementations are retained in the JSON. This quick pass does not establish that unseen issues do not exist.
