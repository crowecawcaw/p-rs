#!/usr/bin/env python3
"""Run the supplemental wide-bundle probe against selected frozen submissions."""
from concurrent.futures import ThreadPoolExecutor,as_completed
import json
import argparse
from pathlib import Path
import run_sonnet as r
ROOT=Path(__file__).resolve().parents[1]
def check(run):
    out=run/'grading'
    args=['docker','run','--rm','--network','none','--cap-drop','ALL','--security-opt','no-new-privileges',
        '--user',f'{r.os.getuid()}:{r.os.getgid()}','--memory','768m','--cpus','1','--read-only',
        '--tmpfs','/tmp:rw,size=64m','--mount',f'type=bind,src={ROOT}/private,dst=/grader/private,readonly',
        '--mount',f'type=bind,src={ROOT}/public,dst=/grader/public,readonly',
        '--mount',f'type=bind,src={run}/submission,dst=/submission,readonly',
        '--mount',f'type=bind,src={out},dst=/output',r.IMAGE,
        'python','/grader/private/stress_wide.py','/submission/solution.py','--output','/output/wide.json']
    p=r.command(args,timeout=60)
    if not (out/'wide.json').exists(): raise RuntimeError(p.stderr)
    result=json.loads((out/'wide.json').read_text()); result['run']=run.name
    return result
if __name__=='__main__':
    parser=argparse.ArgumentParser()
    parser.add_argument('--prefix',default='luna-')
    parser.add_argument('--output',default='results/wide-stress.json')
    args=parser.parse_args()
    runs=sorted((ROOT/'runs').glob(args.prefix+'*'))
    if not runs: raise SystemExit('No runs matched prefix')
    results=[]
    with ThreadPoolExecutor(max_workers=4) as pool:
        for f in as_completed([pool.submit(check,run) for run in runs]):
            result=f.result();results.append(result)
            print(result['run'],result['passed'],result.get('error',''),flush=True)
    (ROOT/args.output).parent.mkdir(parents=True,exist_ok=True)
    (ROOT/args.output).write_text(json.dumps(sorted(results,key=lambda r:r['run']),indent=2))
