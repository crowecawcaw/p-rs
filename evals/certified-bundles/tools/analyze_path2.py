#!/usr/bin/env python3
"""Score the PATH-corrected P arm against the unchanged v2 Python controls."""
from transcripts import read_transcript
import json
from pathlib import Path
import re
import statistics
ROOT=Path(__file__).resolve().parents[1]
secondary={r['run']:r for path in ['v2-wide-stress.json','v2-path-wide-stress.json'] for r in json.loads((ROOT/'results'/path).read_text())}
model_review={r['run']:r for r in json.loads((ROOT/'results/v2-path-model-review.json').read_text())}
runs=sorted((ROOT/'runs').glob('v2-python-*'))+sorted((ROOT/'runs').glob('path2-p-*'))
trials=[]
for run in runs:
    m=json.loads((run/'metadata.json').read_text());g=json.loads((run/'grading/grade.json').read_text())
    assert m['prompt_version']==2 and g['harness_version']=='1.1.3'
    assert not any('TimeoutError' in c.get('error','') for c in g['cases'])
    logs=[json.loads(l) for l in read_transcript(run).splitlines()]
    writes=[r for r in logs if re.search(r'cat\s*>\s*solution\.py',r['command'])]
    checks=[r for r in logs if any(re.match(r'\s*(?:/opt/)?snappea\s+check\b',l) and '--help' not in l for l in r['command'].splitlines())]
    coverage=[]
    for path in (run/'submission').rglob('coverage.txt'):
        for name,pct in re.findall(r'Monitor: ([^\n]+)\n=+\nEvent coverage: ([0-9.]+)%',path.read_text()):
            coverage.append({'monitor':name,'event_coverage_percent':float(pct),'path':str(path.relative_to(run/'submission'))})
    result={'run':run.name,'arm':m['arm'],'model':m['model'],'thinking':m['reasoning_effort'],
        'prompt_version':2,'harness_version':g['harness_version'],'environment_revision':m.get('environment_revision','pre-path-fix'),
        'primary_passed':g['passed'],'primary_total':g['total'],'primary_all_passed':g['all_passed'],
        'failed_cases':[{'case':c['case'],'error':c['error']} for c in g['cases'] if not c['passed']],
        'tool_calls':len(logs),'seconds_through_last_tool':round(logs[-1]['elapsed']+logs[-1]['seconds'],3),
        'first_solution_write_call':writes[0]['call'] if writes else None,'first_p_check_call':checks[0]['call'] if checks else None,
        'p_checker_invoked':bool(checks),'p_check_before_solution':bool(checks and writes and checks[0]['call']<writes[0]['call']),
        'secondary_wide_passed':secondary[run.name]['passed'],'public_files_unchanged':m['public_files_unchanged'],
        'monitor_coverage':coverage,'postfreeze_model_review':model_review.get(run.name)}
    trials.append(result)
summary={'cohort':'short-design-first-v2-path-corrected','solver_runs_scored':12,'control_reused':True,
    'control_source':'v2-python-01 through v2-python-06','p_source':'path2-p-01 through path2-p-06',
    'diagnostic_p_runs_separate':'v2-p-01 through v2-p-06; stored in results/v2-comparison.json',
    'model':'gpt-6-luna','thinking':'high','prompt_version':2,'harness_version':'1.1.3','primary_case_count':20,'schedule_seeds':64,
    'arms':{},'trials':trials,'caveats':['One task, six independent solvers per arm; exploratory outcomes.',
        'The corrected P arm uses fresh agents; the Python controls are reused unchanged rather than rerun.',
        'P is now on normal PATH; task prompt text is unchanged except independent run IDs.',
        'V2 bridge output is bounded; v1 used an earlier bridge, so cross-cohort timing is descriptive.',
        'Phase order measures observed file writes/checks, not unobservable internal design reasoning.']}
for arm in ['python','p']:
    rs=[r for r in trials if r['arm']==arm]
    summary['arms'][arm]={'runs':len(rs),'primary_all_passed':sum(r['primary_all_passed'] for r in rs),
        'primary_cases_passed':sum(r['primary_passed'] for r in rs),'primary_cases_total':sum(r['primary_total'] for r in rs),
        'median_tool_calls':statistics.median(r['tool_calls'] for r in rs),
        'median_seconds_through_last_tool':round(statistics.median(r['seconds_through_last_tool'] for r in rs),3),
        'secondary_wide_passed':sum(r['secondary_wide_passed'] for r in rs),
        'primary_and_secondary_passed':sum(r['primary_all_passed'] and r['secondary_wide_passed'] for r in rs),
        'p_checker_invoked':sum(r['p_checker_invoked'] for r in rs),'p_check_before_solution':sum(r['p_check_before_solution'] for r in rs)}
(ROOT/'results/v2-path-comparison.json').write_text(json.dumps(summary,indent=2))
print(json.dumps(summary['arms'],indent=2))
