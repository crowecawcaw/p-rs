#!/usr/bin/env python3
"""Produce an honest summary: never count calibration/smoke runs as agent trials."""
import json
import argparse
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
parser=argparse.ArgumentParser()
parser.add_argument('--prefix',default='luna-')
parser.add_argument('--output',default='results/summary.json')
args=parser.parse_args()
trials=[]
smoke=[]
for path in sorted((ROOT/'runs').glob(args.prefix+'*/metadata.json')):
    meta=json.loads(path.read_text())
    report_path=path.parent/'grading/grade.json'
    report=json.loads(report_path.read_text()) if report_path.exists() else None
    item={'run':meta['run_id'],'arm':meta['arm'],'model':meta['model'],
        'status':meta['status'],'seconds':meta['seconds'],'generated_tokens':meta['generated_tokens'],
        'tool_calls':meta['tool_calls'],'public_files_unchanged':meta['public_files_unchanged'],
        'passed':report['passed'] if report else None,'total':report['total'] if report else None,
        'all_passed':report['all_passed'] if report else None,
        'failed_cases':[c['case'] for c in report['cases'] if not c['passed']] if report else []}
    if meta['smoke']: smoke.append(item)
    else: trials.append(item)
summary={'selected_prefix':args.prefix,'agent_trials':trials,'smoke_runs':smoke,'arms':{}}
for arm in ['python','p']:
    valid=[r for r in trials if r['arm']==arm and r['status']!='infrastructure_error']
    summary['arms'][arm]={'runs':len(valid),'all_passed':sum(r['all_passed'] is True for r in valid)}
summary['comparison_available']=all(summary['arms'][arm]['runs']>0 for arm in ['python','p'])
if not summary['comparison_available']:
    summary['interpretation']='No P versus Python-only agent comparison is available. Smoke and mutation results do not measure agent design quality.'
(ROOT/args.output).parent.mkdir(parents=True,exist_ok=True)
(ROOT/args.output).write_text(json.dumps(summary,indent=2))
print(json.dumps(summary,indent=2))
