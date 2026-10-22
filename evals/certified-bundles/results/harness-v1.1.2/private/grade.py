#!/usr/bin/env python3
"""Run privately after freezing a submission. No agent-visible hidden feedback."""
import argparse
import asyncio
import copy
import importlib.util
import hashlib
import json
from pathlib import Path
import sys
import time
import traceback

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'public'))
from contract import digest
from runtime import Store, Crash
from oracle import Oracle, linearizable

HEADS = {'schema':'s0', 'service':'v0', 'client':'c0'}
U = {'schema':{'expected':'s0','new':'s1'}, 'service':{'expected':'v0','new':'v1'}}
V = {'schema':{'expected':'s0','new':'s2'}, 'client':{'expected':'c0','new':'c2'}}

class Grader:
    def __init__(self, service, seeds):
        self.Service = service
        self.seeds = seeds
        self.case_trace = []

    def env(self, seed=0, heads=None):
        heads = heads or HEADS
        store = Store(seed)
        return store, self.Service(store, heads.copy()), Oracle(heads)

    async def call(self, s, model, method, *args):
        self.case_trace.append({'method':method,'args':args})
        got = await getattr(s, method)(*copy.deepcopy(args))
        want = model.call(method, args)
        assert got == want, f'{method}{args}: expected {want!r}, got {got!r}'
        return got

    async def ready(self, s, m, bid='b', updates=None):
        updates = updates or U
        await self.call(s,m,'submit',bid,updates)
        for name in ['alice','bob']:
            await self.call(s,m,'approve',bid,1,name)
        await self.call(s,m,'record_ci',bid,1,digest(updates),True)

    async def test_basic_durability(self):
        store,s,m = self.env()
        await self.ready(s,m)
        await self.call(s,m,'publish','b',1,'p')
        s = self.Service(store, HEADS)
        await self.call(s,m,'read_heads',list(HEADS))
        await self.call(s,m,'get_status','b')

    async def test_distinct_authorized_approvals(self):
        _,s,m = self.env()
        await self.call(s,m,'submit','b',U)
        for name in ['alice','alice','mallory','root']:
            await self.call(s,m,'approve','b',1,name)
        await self.call(s,m,'record_ci','b',1,digest(U),True)
        await self.call(s,m,'publish','b',1,'p')
        await self.call(s,m,'get_status','b')

    async def test_ci_exact_candidate(self):
        _,s,m = self.env()
        await self.call(s,m,'submit','b',U)
        for name in ['alice','bob']: await self.call(s,m,'approve','b',1,name)
        changed_base = copy.deepcopy(U)
        changed_base['schema']['expected'] = 'wrong-base'
        await self.call(s,m,'record_ci','b',1,digest(changed_base),True)
        await self.call(s,m,'publish','b',1,'p')

    async def test_failing_ci_revokes_success(self):
        _,s,m = self.env()
        await self.ready(s,m)
        await self.call(s,m,'record_ci','b',1,digest(U),False)
        await self.call(s,m,'publish','b',1,'p')

    async def test_revision_evidence_and_submit_retry(self):
        _,s,m = self.env()
        await self.ready(s,m)
        await self.call(s,m,'revise','b',1,V)
        await self.call(s,m,'submit','b',U)
        await self.call(s,m,'approve','b',1,'carol')
        await self.call(s,m,'record_ci','b',1,digest(U),True)
        await self.call(s,m,'publish','b',2,'p')
        await self.call(s,m,'get_status','b')
        await self.call(s,m,'revise','b',1,U)

    async def test_all_rejection_memos(self):
        _,s,m = self.env()
        await self.call(s,m,'submit','b',U)
        await self.call(s,m,'publish','b',2,'stale')
        await self.call(s,m,'publish','b',1,'unready')
        await self.call(s,m,'revise','b',1,U)
        for name in ['alice','bob']: await self.call(s,m,'approve','b',2,name)
        await self.call(s,m,'record_ci','b',2,digest(U),True)
        await self.call(s,m,'publish','b',2,'stale')
        await self.call(s,m,'publish','b',1,'unready')
        await self.call(s,m,'publish','b',2,'new')

    async def test_stale_bases_and_conflict_memo(self):
        _,s,m = self.env()
        await self.ready(s,m,'a',U)
        await self.ready(s,m,'b',V)
        await self.call(s,m,'publish','a',1,'pa')
        await self.call(s,m,'publish','b',1,'pb')
        repair = {'schema':{'expected':'s1','new':'s2'},'client':{'expected':'c0','new':'c2'}}
        await self.call(s,m,'revise','b',1,repair)
        for name in ['alice','bob']: await self.call(s,m,'approve','b',2,name)
        await self.call(s,m,'record_ci','b',2,digest(repair),True)
        await self.call(s,m,'publish','b',1,'pb')
        await self.call(s,m,'publish','b',2,'pb2')

    async def test_published_bundle_cannot_reapply(self):
        _,s,m = self.env()
        await self.ready(s,m,'a',U)
        await self.call(s,m,'publish','a',1,'pa')
        nxt = {'schema':{'expected':'s1','new':'s3'}}
        await self.ready(s,m,'b',nxt)
        await self.call(s,m,'publish','b',1,'pb')
        for rid in ['pa','another']:
            await self.call(s,m,'publish','a',1,rid)
        await self.call(s,m,'approve','a',1,'carol')
        await self.call(s,m,'record_ci','a',1,digest(U),False)
        await self.call(s,m,'revise','a',1,V)
        await self.call(s,m,'read_heads',list(HEADS))

    async def test_no_mutable_aliases(self):
        _,s,m = self.env()
        updates = copy.deepcopy(U)
        # Pass the actual object to the service, so mutating it tests input aliasing.
        got = await s.submit('b',updates)
        assert got == m.call('submit',('b',updates))
        updates['schema']['new'] = 'mutated-input'
        status = await self.call(s,m,'get_status','b')
        status['updates']['schema']['new'] = 'mutated-output'
        status['approvals'].append('mallory')
        await self.call(s,m,'get_status','b')

    async def history(self, seed, setup, ops):
        store,s,m = self.env(seed)
        await setup(s,m)
        before = copy.deepcopy(m)
        history = []
        counter = 0
        async def run(method,args):
            nonlocal counter
            op = {'method':method,'args':copy.deepcopy(args),'start':counter}
            counter += 1
            client = self.Service(store, HEADS.copy())
            op['result'] = await getattr(client,method)(*copy.deepcopy(args))
            op['end'] = counter
            counter += 1
            history.append(op)
        await asyncio.gather(*(run(method,args) for method,args in ops))
        # Post-quiescence observations constrain the final state too.
        for method,args in [('read_heads',(list(HEADS),)),('get_status',('b',))]:
            await run(method,args)
        self.case_trace = history
        assert linearizable(before,history), f'non-linearizable history seed={seed}: {history!r}'

    async def test_concurrent_overlapping_publish(self):
        async def setup(s,m):
            await self.ready(s,m,'a',U)
            await self.ready(s,m,'b',V)
        for seed in range(self.seeds):
            await self.history(seed,setup,[('publish',('a',1,'pa')),('publish',('b',1,'pb')),
                ('read_heads',(list(HEADS),)),('get_status',('b',))])

    async def test_concurrent_evidence_revision_publish(self):
        async def setup(s,m): await self.call(s,m,'submit','b',U)
        for seed in range(self.seeds):
            await self.history(seed,setup,[('approve',('b',1,'alice')),('approve',('b',1,'bob')),
                ('record_ci',('b',1,digest(U),True)),('revise',('b',1,V)),('publish',('b',1,'p'))])

    async def test_concurrent_duplicate_requests(self):
        async def setup(s,m): await self.ready(s,m)
        for seed in range(self.seeds):
            await self.history(seed,setup,[('publish',('b',1,'p')),('publish',('b',1,'p')),
                ('publish',('b',1,'other')),('read_heads',(list(HEADS),))])

    async def test_concurrent_independent_progress(self):
        async def setup(s,m):
            await self.ready(s,m,'a',{'schema':{'expected':'s0','new':'s1'}})
            await self.ready(s,m,'b',{'client':{'expected':'c0','new':'c1'}})
        for seed in range(self.seeds):
            await self.history(seed,setup,[('publish',('a',1,'pa')),('publish',('b',1,'pb')),
                ('read_heads',(list(HEADS),))])

    async def test_initialization_race(self):
        for seed in range(self.seeds):
            store,s,m = self.env(seed)
            clients = [s,self.Service(store,HEADS)]
            result = await asyncio.gather(clients[0].submit('b',U), clients[1].submit('c',V))
            for bid,updates,got in zip(['b','c'],[U,V],result):
                assert got == m.call('submit',(bid,updates))
            await self.call(s,m,'get_status','b')
            await self.call(s,m,'get_status','c')

    async def crash_sweep(self, setup, operation, verify):
        store,s,m = self.env()
        await setup(s,m)
        baseline = copy.deepcopy(store._records)
        start_steps, start_writes = store.steps,store.writes
        await getattr(s,operation[0])(*copy.deepcopy(operation[1]))
        steps = store.steps-start_steps
        writes = store.writes-start_writes
        # Sweep EVERY before/after storage boundary and immediately after each write.
        cuts = [('step',n) for n in range(1,steps+1)] + [('write',n) for n in range(1,writes+1)]
        assert cuts, 'operation performed no storage calls'
        for kind,cut in cuts:
            st = Store(0)
            st._records = copy.deepcopy(baseline)
            count = 0
            async def hook(event):
                nonlocal count
                count += 1
                if kind == 'step' and count == cut: raise Crash('boundary crash')
            st.hook = hook
            if kind == 'write': st.crash_at = cut
            crashed = False
            try:
                await getattr(self.Service(st,HEADS),operation[0])(*copy.deepcopy(operation[1]))
            except Crash:
                crashed = True
            assert crashed, (kind,cut,steps,writes)
            st.hook = None
            st.crash_at = None
            await verify(self.Service(st,HEADS),st)
        self.case_trace = [{'crash_points':len(cuts),'operation':operation}]

    async def test_crash_publish_atomic_and_retry(self):
        async def setup(s,m): await self.ready(s,m)
        async def verify(s,store):
            heads = await s.read_heads(list(HEADS))
            old = HEADS
            new = dict(HEADS,schema='s1',service='v1')
            assert heads in [old,new], f'torn heads: {heads}'
            status = await s.get_status('b')
            assert status['published'] == (heads == new), 'bundle status disagrees with heads'
            assert await s.publish('b',1,'p') == 'published'
            assert await s.read_heads(list(HEADS)) == new
            assert await s.publish('b',1,'p') == 'published'
        await self.crash_sweep(setup,('publish',('b',1,'p')),verify)

    async def test_crash_rejection_memo(self):
        async def setup(s,m): await self.call(s,m,'submit','b',U)
        async def verify(s,store):
            # Retry BEFORE changing readiness: fixes the interrupted invocation's outcome.
            assert await s.publish('b',1,'p') == 'not_ready'
            for who in ['alice','bob']: await s.approve('b',1,who)
            await s.record_ci('b',1,digest(U),True)
            assert await s.publish('b',1,'p') == 'not_ready'
            assert await s.publish('b',1,'fresh') == 'published'
        await self.crash_sweep(setup,('publish',('b',1,'p')),verify)

    async def test_crash_revision_atomic(self):
        async def setup(s,m): await self.ready(s,m)
        async def verify(s,store):
            b = await s.get_status('b')
            if b['revision'] == 1:
                assert b['approvals'] == ['alice','bob'] and b['ci'] and b['updates'] == U
                assert await s.revise('b',1,V) == 'updated'
            else:
                assert b['revision'] == 2 and b['approvals'] == [] and not b['ci'] and b['updates'] == V
            assert await s.publish('b',2,'p') == 'not_ready'
        await self.crash_sweep(setup,('revise',('b',1,V)),verify)

    async def test_crash_submit_retry(self):
        async def setup(s,m): pass
        async def verify(s,store):
            assert await s.submit('b',U) == {'bundle':'b','revision':1,'digest':digest(U)}
            assert (await s.get_status('b'))['updates'] == U
            await s.revise('b',1,V)
            assert await s.submit('b',U) == {'bundle':'b','revision':1,'digest':digest(U)}
        await self.crash_sweep(setup,('submit',('b',U)),verify)

    async def test_generated_sequential_traces(self):
        import random
        for seed in range(self.seeds):
            rng = random.Random(seed+12000)
            store,s,m = self.env(seed)
            for i in range(5): await self.call(s,m,'submit',f'b{i}',U)
            for step in range(60):
                bid = f'b{rng.randrange(5)}'
                b = m.bundles[bid]
                rev = b['revision'] + rng.choice([0,0,0,-1,1])
                method = rng.choice(['approve','record_ci','revise','publish','read_heads','get_status'])
                if method == 'approve': args = (bid,rev,rng.choice(['alice','bob','carol','mallory']))
                elif method == 'record_ci': args = (bid,rev,rng.choice([b['digest'],'wrong']),rng.choice([True,False]))
                elif method == 'revise': args = (bid,rev,rng.choice([U,V]))
                elif method == 'publish': args = (bid,rev,f'p{seed}-{step}')
                elif method == 'get_status': args = (bid,)
                else: args = (list(HEADS),)
                if step % 7 == 0: s = self.Service(store,HEADS)
                await self.call(s,m,method,*args)

    async def test_record_growth_and_quiescent_progress(self):
        heads = {f'repo-{i:02}':'initial-'+str(i) for i in range(48)}
        store,s,m = self.env(heads=heads)
        for i in range(100):
            branch = f'repo-{i%48:02}'
            updates = {branch:{'expected':m.heads[branch],'new':f'commit-{i}-'+('x'*80)}}
            operations = [('submit',(f'bundle-{i:04}',updates)),
                ('approve',(f'bundle-{i:04}',1,'alice')),
                ('approve',(f'bundle-{i:04}',1,'bob')),
                ('record_ci',(f'bundle-{i:04}',1,digest(updates),True)),
                ('publish',(f'bundle-{i:04}',1,f'publish-{i:04}'))]
            for method,args in operations:
                before = store.steps
                await self.call(s,m,method,*args)
                assert store.steps-before <= 10000, 'quiescent per-operation progress budget exceeded'
            if i % 10 == 0: s = self.Service(store,heads)
        await self.call(s,m,'read_heads',list(heads))
        for i in [0,10,50,99]: await self.call(s,m,'get_status',f'bundle-{i:04}')

async def main(args):
    path = Path(args.solution).resolve()
    sys.path.insert(0,str(path.parent))
    cases = sorted(name for name in dir(Grader) if name.startswith('test_') and (not args.only or args.only in name))
    if not cases:
        raise SystemExit('No test cases matched --only; refusing an empty success report.')
    load_error = None
    try:
        spec = importlib.util.spec_from_file_location('submission',path)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        service = module.Service
    except (Exception, Crash, SystemExit) as exc:
        service = None
        load_error = f'{type(exc).__name__}: {exc}'
    grader = Grader(service,args.seeds)
    results = []
    for name in cases:
        grader.case_trace = []
        start = time.monotonic()
        try:
            if load_error: raise RuntimeError('Submission import failed: ' + load_error)
            await asyncio.wait_for(getattr(grader,name)(),timeout=args.timeout)
            result = dict(case=name,passed=True)
        except (Exception, Crash, SystemExit) as exc:
            result = dict(case=name,passed=False,error=f'{type(exc).__name__}: {exc}',
                traceback=traceback.format_exc(),replay=grader.case_trace[-30:])
        result['seconds'] = round(time.monotonic()-start,4)
        results.append(result)
        print(('PASS' if result['passed'] else 'FAIL'),name, result.get('error','')[:180],flush=True)
    report = {'harness_version':'1.1.2','source_sha256':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [ROOT/'public/runtime.py', ROOT/'private/grade.py', ROOT/'private/oracle.py']},'solution':str(path),'seeds':args.seeds,'passed':sum(r['passed'] for r in results),
              'case_timeout_seconds':args.timeout,'total':len(results),'all_passed':all(r['passed'] for r in results),'cases':results}
    Path(args.output).parent.mkdir(parents=True,exist_ok=True)
    Path(args.output).write_text(json.dumps(report,indent=2))
    print(f"{report['passed']}/{report['total']} cases passed; report: {args.output}")
    return 0 if report['all_passed'] else 1

if __name__ == '__main__':
    p = argparse.ArgumentParser()
    p.add_argument('solution')
    p.add_argument('--output',default='report.json')
    p.add_argument('--seeds',type=int,default=12)
    p.add_argument('--timeout',type=float,default=180)
    p.add_argument('--only')
    raise SystemExit(asyncio.run(main(p.parse_args())))
