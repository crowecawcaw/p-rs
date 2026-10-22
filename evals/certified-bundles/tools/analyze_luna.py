#!/usr/bin/env python3
"""Aggregate independent solver runs; keep primary and exploratory probes separate."""
from transcripts import read_transcript
import json
from pathlib import Path
import re
import statistics
ROOT=Path(__file__).resolve().parents[1]
wide={r['run']:r for r in json.loads((ROOT/'results/wide-stress.json').read_text())}
trials=[]
for run in sorted((ROOT/'runs').glob('luna-*')):
    meta=json.loads((run/'metadata.json').read_text())
    report=json.loads((run/'grading/grade.json').read_text())
    assert report['harness_version'] in ('1.1.2','1.1.3'), (run.name,report.get('harness_version'))
    assert not any('TimeoutError' in c.get('error','') for c in report['cases']), run.name
    tools=[json.loads(line) for line in read_transcript(run).splitlines()]
    coverage=[]
    for f in (run/'submission').rglob('coverage.txt'):
        content=f.read_text()
        for name,pct in re.findall(r'Monitor: ([^\n]+)\n=+\nEvent coverage: ([0-9.]+)%',content):
            coverage.append({'path':str(f.relative_to(run/'submission')),'monitor':name,'event_coverage_percent':float(pct)})
    trials.append({'run':run.name,'arm':meta['arm'],'model':meta['model'],'thinking':meta['reasoning_effort'],
        'cohort':'pilot_regraded' if int(run.name[-2:])<=3 else 'runtime_fixed',
        'status':meta['status'],'primary_passed':report['passed'],'primary_total':report['total'],
        'primary_all_passed':report['all_passed'],'harness_version':report['harness_version'],
        'failed_cases':[{'case':c['case'],'error':c['error']} for c in report['cases'] if not c['passed']],
        'tool_calls':len(tools),'seconds_through_last_tool':round(tools[-1]['elapsed']+tools[-1]['seconds'],3),
        'tool_seconds':round(sum(x['seconds'] for x in tools),3),
        'shell_error_calls':sum(x['result']['exit_code']!=0 for x in tools),
        'public_files_unchanged':meta['public_files_unchanged'],
        'inspected_coverage':any('coverage.txt' in r['command'] and ('cat ' in r['command'] or 'read_text' in r['command']) for r in tools),
        'monitor_coverage':coverage,'supplemental_wide_passed':wide[run.name]['passed']})
summary={'model':'gpt-6-luna','thinking':'high','independent_solver_runs':len(trials),
    'primary_case_count':20,'schedule_seeds':64,'limits':{'wall_seconds':900,'shell_calls':80},
    'generated_token_usage_available':False,'arms':{},'trials':trials,
    'caveats':['One task; small exploratory cohorts; no general treatment-effect claim.',
        'First cohort saw the original runtime cap; all frozen artifacts were regraded after correcting the harness.',
        'Wide-bundle probe was added after inspecting submissions; it is separate from the original primary score.',
        'Elapsed measure ends at the final tool return, not parent processing/freeze time.',
        'Repeated schedules and test cases are not independent experimental units.']}
for arm in ['python','p']:
    group=[x for x in trials if x['arm']==arm]
    summary['arms'][arm]={'runs':len(group),'primary_all_passed':sum(x['primary_all_passed'] for x in group),
        'primary_cases_passed':sum(x['primary_passed'] for x in group),
        'primary_cases_total':sum(x['primary_total'] for x in group),
        'median_tool_calls':statistics.median(x['tool_calls'] for x in group),
        'median_seconds_through_last_tool':round(statistics.median(x['seconds_through_last_tool'] for x in group),3),
        'supplemental_wide_passed':sum(x['supplemental_wide_passed'] for x in group),
        'primary_and_supplemental_passed':sum(x['primary_all_passed'] and x['supplemental_wide_passed'] for x in group)}
(ROOT/'results/luna-comparison.json').write_text(json.dumps(summary,indent=2))
print(json.dumps(summary['arms'],indent=2))
