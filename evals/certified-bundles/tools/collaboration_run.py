#!/usr/bin/env python3
"""Provision/freeze/grade a collaboration-agent trial without sharing hidden tests."""
import argparse
import fcntl
import hashlib
import json
from pathlib import Path
import time
import run_sonnet as r
from delegation import prompt as delegation_prompt
from prompts import VERSION as PROMPT_VERSION
ROOT=Path(__file__).resolve().parents[1]

def start(run_id,arm):
    run,work,prompt=r.prepare(run_id,arm)
    delegation=delegation_prompt(run_id,arm)
    (run/'delegation.txt').write_text(delegation)
    result=r.command(r.container_args('snappea-eval-'+run_id,work,arm))
    if result.returncode: raise RuntimeError(result.stderr)
    preflight=None
    if arm=='p':
        check=r.command(['docker','exec','snappea-eval-'+run_id,'bash','-c',
            'command -v snappea && snappea version && python -c "import snappea; from importlib.metadata import version; print(version(\'pysnappea\'))"'])
        preflight={'exit_code':check.returncode,'stdout':check.stdout,'stderr':check.stderr}
        if check.returncode:
            r.command(['docker','rm','-f','snappea-eval-'+run_id])
            raise RuntimeError('Snappea preflight failed: '+check.stderr)
    meta={'run_id':run_id,'arm':arm,'prompt_version':r.prompt_version(),'prompt_variant':r.prompt_variant(arm),'model':'gpt-6-luna','reasoning_effort':'high',
        'backend':'collaboration','tool_preflight':preflight,'environment_revision':'pysnappea-tests','snappea_on_path':arm=='p','tool_bridge_version':2,'delegation_prompt_sha256':hashlib.sha256(delegation.encode()).hexdigest(),'image':r.image_for_arm(arm),'limits':{'seconds':900,'tool_calls':80,'generated_tokens':None},
        'public_sha256':{n:hashlib.sha256((ROOT/'public'/n).read_bytes()).hexdigest() for n in r.PUBLIC},
        'prompt_sha256':hashlib.sha256(prompt.encode()).hexdigest(),'smoke':False,
        'status':'running','seconds':None,'generated_tokens':None,'input_tokens':None,'tool_calls':0,
        'provisioned_unix':time.time()}
    if arm=='p':
        binary, source = r.tool_paths()
        revision = r.command(['git','-C',str(source),'rev-parse','HEAD'])
        meta['snappea_binary_sha256'] = hashlib.sha256(binary.read_bytes()).hexdigest()
        meta['docs_source_revision'] = revision.stdout.strip() if revision.returncode == 0 else None
    (run/'metadata.json').write_text(json.dumps(meta,indent=2))
    print(json.dumps({'run':run_id,'arm':arm,'container':'snappea-eval-'+run_id}))

def finish(run_id,status,seeds):
    run=ROOT/'runs'/run_id
    with (run/'bridge.lock').open('a') as lock:
        fcntl.flock(lock,fcntl.LOCK_EX)
        state_path=run/'bridge-state.json'
        state=json.loads(state_path.read_text()) if state_path.exists() else {'started':time.time(),'calls':0}
        state['closed']=True
        state_path.write_text(json.dumps(state,indent=2))
        r.command(['docker','rm','-f','snappea-eval-'+run_id])
        r.frozen_submission(run,run/'work')
        meta=json.loads((run/'metadata.json').read_text())
        meta.update(status=status,seconds=round(time.time()-state['started'],3),tool_calls=state['calls'],
            public_files_unchanged=all(hashlib.sha256((run/'work'/n).read_bytes()).hexdigest()==meta['public_sha256'][n] for n in r.PUBLIC))
        (run/'metadata.json').write_text(json.dumps(meta,indent=2))
    report=r.grade(run,seeds)
    print(json.dumps({'run':run_id,'passed':report['passed'],'total':report['total']}))

if __name__=='__main__':
    p=argparse.ArgumentParser()
    p.add_argument('action',choices=['start','finish'])
    p.add_argument('--run-id',required=True)
    p.add_argument('--arm',choices=['p','python'])
    p.add_argument('--status',default='agent_finished')
    p.add_argument('--seeds',type=int,default=64)
    a=p.parse_args()
    if a.action=='start': start(a.run_id,a.arm)
    else: finish(a.run_id,a.status,a.seeds)
