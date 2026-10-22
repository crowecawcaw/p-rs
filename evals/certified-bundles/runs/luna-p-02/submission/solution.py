import hashlib
import json
from contract import digest

ROOT = 'cb:root'


def enc(x):
    return json.dumps(x, sort_keys=True, separators=(',', ':'), ensure_ascii=True)


class Service:
    def __init__(self, store, initial_heads):
        self.store = store
        self.initial_heads = dict(initial_heads)

    async def _state(self):
        ver, ptr = await self.store.read(ROOT)
        events = []
        while ptr is not None:
            _, node = await self.store.read('cb:node:' + ptr)
            if node is None:
                raise RuntimeError('broken committed log')
            events.append(node['event'])
            ptr = node['parent']
        s = {'heads': dict(self.initial_heads), 'bundles': {}, 'publish_ids': {}}
        for e in reversed(events):
            t = e['type']
            if t == 'submit':
                s['bundles'][e['id']] = {'revision': 1, 'updates': e['updates'], 'digest': e['digest'], 'submit_digest': e['digest'], 'approvals': [], 'ci': False, 'published': False}
            elif t == 'revise':
                b = s['bundles'][e['id']]
                b.update(revision=e['revision'], updates=e['updates'], digest=e['digest'], approvals=[], ci=False)
            elif t == 'approve':
                b = s['bundles'][e['id']]
                if e['reviewer'] not in b['approvals']: b['approvals'].append(e['reviewer'])
            elif t == 'ci':
                s['bundles'][e['id']]['ci'] = e['passed']
            elif t == 'publish':
                s['publish_ids'][e['request_id']] = e['outcome']
                if e['outcome'] == 'published' and e['apply']:
                    for branch, upd in e['updates'].items(): s['heads'][branch] = upd['new']
                    s['bundles'][e['id']]['published'] = True
        return ver, await self.store.read(ROOT), s

    async def _commit(self, ver, root_pair, event):
        _, ptr = root_pair
        raw = enc({'parent': ptr, 'event': event})
        node_id = hashlib.sha256(raw.encode()).hexdigest()
        node_key = 'cb:node:' + node_id
        nv, old = await self.store.read(node_key)
        if old is None:
            await self.store.cas(node_key, nv, {'parent': ptr, 'event': event})
        elif old != {'parent': ptr, 'event': event}:
            raise RuntimeError('content hash collision')
        return await self.store.cas(ROOT, ver, node_id)

    async def _update(self, decide):
        while True:
            ver, root_pair, s = await self._state()
            event, answer, noop = decide(s)
            if noop: return answer
            if await self._commit(ver, root_pair, event): return answer

    async def submit(self, request_id, updates):
        u = json.loads(enc(updates))
        dg = digest(u)
        def decide(s):
            if request_id in s['bundles']:
                b = s['bundles'][request_id]
                return None, {'bundle': request_id, 'revision': 1, 'digest': b['submit_digest']}, True
            return {'type':'submit','id':request_id,'updates':u,'digest':dg}, {'bundle':request_id,'revision':1,'digest':dg}, False
        return await self._update(decide)

    async def revise(self, bundle, expected_revision, updates):
        u=json.loads(enc(updates)); dg=digest(u)
        def decide(s):
            b=s['bundles'][bundle]
            if b['revision'] != expected_revision: return None,'stale',True
            if b['published']: return None,'published',True
            return {'type':'revise','id':bundle,'revision':expected_revision+1,'updates':u,'digest':dg},'updated',False
        return await self._update(decide)

    async def approve(self, bundle, revision, reviewer):
        if reviewer not in ('alice','bob','carol'): return False
        def decide(s):
            b=s['bundles'][bundle]
            if b['revision'] != revision or b['published']: return None,False,True
            return {'type':'approve','id':bundle,'reviewer':reviewer},True,False
        return await self._update(decide)

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        def decide(s):
            b=s['bundles'][bundle]
            if b['revision'] != revision or b['published'] or candidate_digest != b['digest']: return None,False,True
            val=bool(passed)
            if b['ci'] == val: return None,True,True
            return {'type':'ci','id':bundle,'passed':val},True,False
        return await self._update(decide)

    async def publish(self, bundle, revision, request_id):
        def decide(s):
            if request_id in s['publish_ids']: return None,s['publish_ids'][request_id],True
            b=s['bundles'][bundle]
            if revision != b['revision']: out='stale'
            elif b['published']: out='published'
            elif len(set(b['approvals']) & {'alice','bob','carol'}) < 2 or not b['ci']: out='not_ready'
            elif any(s['heads'][branch] != upd['expected'] for branch,upd in b['updates'].items()): out='conflict'
            else: out='published'
            apply = out == 'published' and not b['published']
            e={'type':'publish','id':bundle,'request_id':request_id,'outcome':out,'apply':apply,'updates':b['updates'] if apply else {}}
            return e,out,False
        return await self._update(decide)

    async def read_heads(self, branches):
        _, _, s = await self._state()
        return {b:s['heads'][b] for b in branches}

    async def get_status(self, bundle):
        _, _, s = await self._state()
        b=s['bundles'][bundle]
        return {'revision':b['revision'],'updates':json.loads(enc(b['updates'])),'digest':b['digest'],'approvals':sorted(b['approvals']),'ci':b['ci'],'published':b['published']}
