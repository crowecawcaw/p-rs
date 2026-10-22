#!/usr/bin/env python3
"""Supplemental post-pilot stress: a valid bundle larger than one store record.

Exploratory, not added to the original 20-case primary outcome after seeing results.
"""
import argparse
import asyncio
import importlib.util
import json
from pathlib import Path
import sys
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'public'))
from contract import digest
from runtime import Store

async def check(path):
    path=Path(path).resolve()
    sys.path.insert(0,str(path.parent))
    spec=importlib.util.spec_from_file_location('wide_submission',path)
    m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
    heads={f'repo-{i:02}':f'base-{i}-'+('a'*190) for i in range(64)}
    updates={k:{'expected':v,'new':f'next-{i}-'+('b'*190)} for i,(k,v) in enumerate(heads.items())}
    st=Store()
    s=m.Service(st,heads)
    size=len(json.dumps(updates,separators=(',',':')).encode())
    result={'solution':str(path),'branches':64,'update_map_bytes':size,'supplemental':True}
    try:
        await s.submit('wide',updates)
        for name in ['alice','bob']: assert await s.approve('wide',1,name) is True
        assert await s.record_ci('wide',1,digest(updates),True) is True
        assert await s.publish('wide',1,'pub')=='published'
        assert await m.Service(st,heads).read_heads(list(heads))=={k:v['new'] for k,v in updates.items()}
        result['passed']=True
    except Exception as e:
        result.update(passed=False,error=f'{type(e).__name__}: {e}')
    return result

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('solution');p.add_argument('--output',required=True)
    a=p.parse_args();result=asyncio.run(check(a.solution))
    Path(a.output).write_text(json.dumps(result,indent=2));print(json.dumps(result))
