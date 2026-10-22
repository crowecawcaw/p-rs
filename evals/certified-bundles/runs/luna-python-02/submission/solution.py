import hashlib
import json
from contract import digest

_ROOT = 'certified-bundles/root'
_PREFIX = 'certified-bundles/event/'
_ALLOWED = {'alice', 'bob', 'carol'}

def _encode(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=True)

def _hash(value):
    return hashlib.sha256(_encode(value).encode('utf-8')).hexdigest()

def _empty(heads):
    return {'heads': dict(heads), 'bundles': {}, 'submits': {}, 'publishes': {}}

def _apply(state, event):
    k, p = event['kind'], event['data']
    if k == 'init':
        state['heads'] = dict(p['heads'])
    elif k == 'submit':
        state['bundles'][p['bundle']] = p['record']
        state['submits'][p['bundle']] = p['response']
    elif k == 'bundle':
        state['bundles'][p['bundle']] = p['record']
    elif k == 'heads_bundle':
        for branch, commit in p['heads'].items():
            state['heads'][branch] = commit
        state['bundles'][p['bundle']] = p['record']
    if k == 'publish_outcome' or k == 'heads_bundle':
        state['publishes'][p['request_id']] = p['outcome']

class Service:
    def __init__(self, store, initial_heads):
        self.store = store
        self._initial_heads = dict(initial_heads)

    async def _snapshot(self):
        rv, ptr = await self.store.read(_ROOT)
        if ptr is None:
            init = {'prev': None, 'kind': 'init', 'data': {'heads': self._initial_heads}}
            h = _hash(init)
            await self.store.cas(_PREFIX + h, 0, init)
            await self.store.cas(_ROOT, rv, h)
            rv, ptr = await self.store.read(_ROOT)
        chain = []
        seen = set()
        while ptr is not None:
            if ptr in seen:
                raise RuntimeError('corrupt event chain')
            seen.add(ptr)
            _, ev = await self.store.read(_PREFIX + ptr)
            if ev is None:
                raise RuntimeError('missing committed event')
            chain.append(ev)
            ptr = ev['prev']
        state = {'heads': {}, 'bundles': {}, 'submits': {}, 'publishes': {}}
        for ev in reversed(chain):
            _apply(state, ev)
        return rv, state

    async def _commit(self, rv, state, kind, data):
        # The caller's snapshot is valid only while root version remains rv.
        _, ptr = await self.store.read(_ROOT)
        event = {'prev': ptr, 'kind': kind, 'data': data}
        h = _hash(event)
        await self.store.cas(_PREFIX + h, 0, event)
        return await self.store.cas(_ROOT, rv, h)

    async def _change(self, decide):
        while True:
            rv, s = await self._snapshot()
            result = decide(s)
            if result[0] is None:
                return result[1]
            kind, data, answer = result
            if await self._commit(rv, s, kind, data):
                return answer

    async def submit(self, request_id, updates):
        updates = json.loads(_encode(updates))
        d = digest(updates)
        response = {'bundle': request_id, 'revision': 1, 'digest': d}
        record = {'revision': 1, 'updates': updates, 'digest': d,
                  'approvals': [], 'ci': False, 'published': False}
        def decide(s):
            if request_id in s['submits']:
                return (None, dict(s['submits'][request_id]))
            return ('submit', {'bundle': request_id, 'record': record, 'response': response}, response)
        return await self._change(decide)

    async def revise(self, bundle, expected_revision, updates):
        updates = json.loads(_encode(updates))
        d = digest(updates)
        def decide(s):
            old = s['bundles'][bundle]
            if old['revision'] != expected_revision: return (None, 'stale')
            if old['published']: return (None, 'published')
            rec = {'revision': old['revision'] + 1, 'updates': updates, 'digest': d,
                   'approvals': [], 'ci': False, 'published': False}
            return ('bundle', {'bundle': bundle, 'record': rec}, 'updated')
        return await self._change(decide)

    async def approve(self, bundle, revision, reviewer):
        def decide(s):
            old = s['bundles'][bundle]
            if reviewer not in _ALLOWED or old['revision'] != revision or old['published']:
                return (None, False)
            rec = dict(old)
            rec['approvals'] = sorted(set(old['approvals']) | {reviewer})
            return ('bundle', {'bundle': bundle, 'record': rec}, True)
        return await self._change(decide)

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        def decide(s):
            old = s['bundles'][bundle]
            if old['revision'] != revision or old['digest'] != candidate_digest or old['published']:
                return (None, False)
            rec = dict(old)
            rec['ci'] = bool(passed)
            return ('bundle', {'bundle': bundle, 'record': rec}, True)
        return await self._change(decide)

    async def publish(self, bundle, revision, request_id):
        def decide(s):
            if request_id in s['publishes']:
                return (None, s['publishes'][request_id])
            old = s['bundles'][bundle]
            if old['revision'] != revision: outcome = 'stale'
            elif old['published']: outcome = 'published'
            elif len(set(old['approvals']) & _ALLOWED) < 2 or not old['ci']: outcome = 'not_ready'
            elif any(s['heads'][branch] != change['expected'] for branch, change in old['updates'].items()): outcome = 'conflict'
            else:
                outcome = 'published'
                newheads = {branch: change['new'] for branch, change in old['updates'].items()}
                rec = dict(old); rec['published'] = True
                data = {'request_id': request_id, 'outcome': outcome, 'heads': newheads,
                        'bundle': bundle, 'record': rec}
                return ('heads_bundle', data, outcome)
            return ('publish_outcome', {'request_id': request_id, 'outcome': outcome}, outcome)
        return await self._change(decide)

    async def read_heads(self, branches):
        _, state = await self._snapshot()
        return {b: state['heads'][b] for b in branches}

    async def get_status(self, bundle):
        _, state = await self._snapshot()
        b = state['bundles'][bundle]
        return {'revision': b['revision'], 'updates': json.loads(_encode(b['updates'])),
                'digest': b['digest'], 'approvals': sorted(b['approvals']),
                'ci': bool(b['ci']), 'published': bool(b['published'])}
