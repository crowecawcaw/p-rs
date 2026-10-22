#!/usr/bin/env python3
"""Analyze the short design-first prompt cohort without changing v1 results."""
from transcripts import read_transcript
import json
from pathlib import Path
import re
import statistics
ROOT=Path(__file__).resolve().parents[1]
wide={r['run']:r for r in json.loads((ROOT/'results/v2-wide-stress.json').read_text())}
trials=[]
for run in sorted((ROOT/'runs').glob('v2-*')):
    meta=json.loads((run/'metadata.json').read_text())
    grade=json.loads((run/'grading/grade.json').read_text())
    assert meta['prompt_version']==2 and grade['harness_version']=='1.1.3'
    tools=[json.loads(line) for line in read_transcript(run).splitlines()]
    def check(row):
        return any('--help' not in line and re.search(r'(?:^|[;&])\s*/opt/snappea\s+check\b',line) for line in row['command'].splitlines())
    def write_solution(row):
        cmd=row['command']
        return bool(re.search(r'cat\s*>\s*solution\.py',cmd) or re.search(r"open\(['\"]solution\.py['\"],\s*['\"]w",cmd) or ('solution.py' in cmd and 'write_text(' in cmd))
    checks=[r for r in tools if check(r)]
    writes=[r for r in tools if write_solution(r)]
    coverage=[]
    for path in (run/'submission').rglob('coverage.txt'):
        for name,pct in re.findall(r'Monitor: ([^\n]+)\n=+\nEvent coverage: ([0-9.]+)%',path.read_text()):
            coverage.append({'path':str(path.relative_to(run/'submission')),'monitor':name,'event_coverage_percent':float(pct)})
    trials.append({'run':run.name,'arm':meta['arm'],'model':meta['model'],'thinking':meta['reasoning_effort'],
        'prompt_version':meta['prompt_version'],'bridge_version':meta['tool_bridge_version'],'status':meta['status'],
        'harness_version':grade['harness_version'],'primary_passed':grade['passed'],'primary_total':grade['total'],'primary_all_passed':grade['all_passed'],
        'failed_cases':[{'case':c['case'],'error':c['error']} for c in grade['cases'] if not c['passed']],
        'tool_calls':len(tools),'seconds_through_last_tool':round(tools[-1]['elapsed']+tools[-1]['seconds'],3),
        'shell_error_calls':sum(r['result']['exit_code']!=0 for r in tools),
        'first_p_check_call':checks[0]['call'] if checks else None,
        'first_solution_write_call':writes[0]['call'] if writes else None,
        'p_check_before_solution':bool(checks and writes and checks[0]['call']<writes[0]['call']),
        'monitor_coverage':coverage,'public_files_unchanged':meta['public_files_unchanged'],
        'supplemental_wide_passed':wide[run.name]['passed']})
summary={'cohort':'short-design-first-v2','model':'gpt-6-luna','thinking':'high','prompt_version':2,
    'independent_solver_runs':len(trials),'primary_case_count':20,'schedule_seeds':64,
    'limits':{'wall_seconds':900,'shell_calls':80},'arms':{},'trials':trials,
    'caveats':['One task; six runs per arm; no general treatment-effect claim.',
        'Wide-bundle test is a separately scored secondary outcome, known to the evaluator before this cohort.',
        'V1 used longer prompts and an older output bridge; cross-cohort timing changes are not attributable solely to wording.',
        'Phase order is derived from command logs and should be checked against model relevance and success.']}
for arm in ['python','p']:
    group=[r for r in trials if r['arm']==arm]
    summary['arms'][arm]={'runs':len(group),'primary_all_passed':sum(r['primary_all_passed'] for r in group),
        'primary_cases_passed':sum(r['primary_passed'] for r in group),'primary_cases_total':sum(r['primary_total'] for r in group),
        'median_tool_calls':statistics.median(r['tool_calls'] for r in group),
        'median_seconds_through_last_tool':round(statistics.median(r['seconds_through_last_tool'] for r in group),3),
        'supplemental_wide_passed':sum(r['supplemental_wide_passed'] for r in group),
        'primary_and_supplemental_passed':sum(r['primary_all_passed'] and r['supplemental_wide_passed'] for r in group),
        'p_check_before_solution':sum(r['p_check_before_solution'] for r in group)}
(ROOT/'results/v2-comparison.json').write_text(json.dumps(summary,indent=2))
print(json.dumps(summary['arms'],indent=2))
