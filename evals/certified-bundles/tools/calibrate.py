#!/usr/bin/env python3
"""Mutation checks: the hidden suite must reject each known design error."""
import json
from pathlib import Path
import subprocess
import sys
ROOT = Path(__file__).resolve().parents[1]
source = (ROOT/'private/reference.py').read_text()
mutations = {
    'stale_ci_after_revision': ("digest=digest(updates), approvals=[], ci=False", "digest=digest(updates)"),
    'wrong_ci_digest_accepted': (" or b['digest'] != candidate_digest", ""),
    'duplicate_approvals_count': ("sorted(set(b['approvals']) | {reviewer})", "sorted(b['approvals'] + [reviewer])"),
    'ci_failure_ignored': ("b['ci'] = bool(passed)", "b['ci'] = b['ci'] or bool(passed)"),
    'no_parent_validation': ("elif any(s['heads'][k] != u['expected'] for k,u in b['updates'].items()): result = 'conflict'", "elif False: result = 'conflict'"),
    'rejected_results_not_memoized': ("s['requests'][request_id] = result", "\n            if result == 'published': s['requests'][request_id] = result"),
    'submit_retry_current_revision': ("return s['bundles'][request_id]['original'], False", "return {'bundle':request_id, 'revision':s['bundles'][request_id]['revision'], 'digest':s['bundles'][request_id]['digest']}, False"),
    'lost_cas_update': ("if not changed or await self._save(version, state):", "if not changed or (await self._save(version, state)) or True:"),
    'read_heads_per_branch': ("return {k: state['heads'][k] for k in branches}", "result = {}\n        for k in branches:\n            _, state = await self._load()\n            result[k] = state['heads'][k]\n        return result"),
}
results = []
for name,(old,new) in mutations.items():
    assert old in source, name
    out = ROOT/'results/mutants'/name
    out.mkdir(parents=True,exist_ok=True)
    (out/'solution.py').write_text(source.replace(old,new))
    proc = subprocess.run([sys.executable,str(ROOT/'private/grade.py'),str(out/'solution.py'),
        '--seeds','12','--timeout','10','--output',str(out/'grade.json')],capture_output=True,text=True)
    report = json.loads((out/'grade.json').read_text())
    rejected = not report['all_passed']
    item = {'mutant':name,'detected':rejected,'passed':report['passed'],'total':report['total'],
            'killed_by':[x['case'] for x in report['cases'] if not x['passed']]}
    results.append(item)
    print(name, 'DETECTED' if rejected else 'SURVIVED', f"{report['passed']}/{report['total']}",flush=True)
(ROOT/'results/calibration.json').write_text(json.dumps(results,indent=2))
raise SystemExit(0 if all(x['detected'] for x in results) else 1)
