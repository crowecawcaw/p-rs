"""Harness self-checks, not submission tests."""
import asyncio
import copy
import importlib.util
from pathlib import Path
import sys
import unittest
import subprocess
import tempfile
import json
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'public'))
sys.path.insert(0,str(ROOT/'private'))
from runtime import Store,Crash
from oracle import Oracle,linearizable
from contract import digest

class HarnessTests(unittest.TestCase):
    def test_real_time_order_is_respected(self):
        o=Oracle({'a':'0'})
        u={'a':{'expected':'0','new':'1'}}
        result=o.call('submit',('b',u))
        start=Oracle({'a':'0'})
        history=[dict(method='submit',args=('b',u),start=0,end=1,result=result),
            dict(method='get_status',args=('b',),start=2,end=3,result=o.call('get_status',('b',)))]
        self.assertTrue(linearizable(start,history))
        history[1]['result']['revision']=2
        self.assertFalse(linearizable(start,history))

    def test_forbidden_two_winners_is_rejected(self):
        o=Oracle({'a':'0'})
        for bid in ['b','c']:
            u={'a':{'expected':'0','new':bid}}
            for method,args in [('submit',(bid,u)),('approve',(bid,1,'alice')),
                ('approve',(bid,1,'bob')),('record_ci',(bid,1,digest(u),True))]: o.call(method,args)
        h=[dict(method='publish',args=(bid,1,'p'+bid),start=i,end=i+3,result='published')
           for i,bid in enumerate(['b','c'])]
        self.assertFalse(linearizable(o,h))
        h[1]['result']='conflict'
        self.assertTrue(linearizable(o,h))

class RuntimeTests(unittest.IsolatedAsyncioTestCase):
    async def test_lost_response_is_durable(self):
        s=Store(crash_at=1)
        with self.assertRaises(Crash): await s.cas('x',0,{'a':1})
        self.assertEqual(await s.read('x'),(1,{'a':1}))
        self.assertFalse(await s.cas('x',0,{'a':2}))

    async def test_long_history_is_not_a_per_call_violation(self):
        s=Store()
        s.steps=250000
        self.assertEqual(await s.read('x'),(0,None))

    async def test_copy_isolation(self):
        s=Store()
        value={'a':[1]}
        await s.cas('x',0,value)
        value['a'].append(2)
        _,read=await s.read('x')
        self.assertEqual(read,{'a':[1]})
        read['a'].append(3)
        self.assertEqual((await s.read('x'))[1],{'a':[1]})

class ReportTests(unittest.TestCase):
    def test_empty_filter_is_not_success(self):
        with tempfile.TemporaryDirectory() as d:
            out=Path(d)/'grade.json'
            p=subprocess.run([sys.executable,str(ROOT/'private/grade.py'),str(ROOT/'private/reference.py'),
                '--only','no-such-case','--output',str(out)],capture_output=True,text=True)
            self.assertNotEqual(p.returncode,0)
            self.assertFalse(out.exists())

    def test_bad_submission_import_is_a_scored_failure(self):
        with tempfile.TemporaryDirectory() as d:
            source=Path(d)/'solution.py'; out=Path(d)/'grade.json'
            source.write_text("from runtime import Crash\nraise Crash('broken submission import')\n")
            p=subprocess.run([sys.executable,str(ROOT/'private/grade.py'),str(source),
                '--output',str(out)],capture_output=True,text=True)
            self.assertEqual(p.returncode,1)
            report=json.loads(out.read_text())
            self.assertEqual(report['passed'],0)
            self.assertEqual(report['total'],20)
            self.assertFalse(report['all_passed'])

if __name__=='__main__': unittest.main()
