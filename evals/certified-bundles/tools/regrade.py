#!/usr/bin/env python3
"""Regrade frozen artifacts after a documented harness correction; no solver feedback."""
import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
import json
from pathlib import Path
import run_sonnet as r
ROOT=Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser()
p.add_argument('run_ids',nargs='+')
p.add_argument('--seeds',type=int,default=64)
p.add_argument('--workers',type=int,default=3)
a=p.parse_args()
def grade(rid):
    run=ROOT/'runs'/rid
    old=run/'grading/grade.json'
    if old.exists():
        version=json.loads(old.read_text()).get('harness_version','1.0')
        archive=run/f'grading/grade-initial-v{version}.json'
        if not archive.exists(): archive.write_bytes(old.read_bytes())
    result=r.grade(run,a.seeds)
    return {'run':rid,'passed':result['passed'],'total':result['total']}
with ThreadPoolExecutor(max_workers=a.workers) as pool:
    for future in as_completed([pool.submit(grade,rid) for rid in a.run_ids]): print(json.dumps(future.result()),flush=True)
