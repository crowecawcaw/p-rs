import hashlib
import json
from contract import digest

ROOT = 'cb:root'
PREFIX = 'cb:event:'
REVIEWERS = {'alice', 'bob', 'carol'}


def _enc(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=True)


def _event_id(event):
    return hashlib.sha256(_enc(event).encode()).hexdigest()


def _blank(heads):
    return {'heads': dict(heads), 'bundles': {}, 'publish_requests': {}}


def _apply(state, op):
    kind = op['kind']
    if kind == 'submit':
        b = op['bundle']
        if b not in state['bundles']:
            state['bundles'][b] = {'revision': 1, 'updates': op['updates'],
                'digest': op['digest'], 'approvals': [], 'ci': False, 'published': False}
    elif kind == 'revise':
        b = state['bundles'][op['bundle']]
        if b['revision'] == op['expected'] and not b['published']:
            b.update(revision=b['revision'] + 1, updates=op['updates'], digest=op['digest'],
                     approvals=[], ci=False)
    elif kind == 'approve':
        b = state['bundles'][op['bundle']]
        if b['revision'] == op['revision'] and not b['published'] and op['reviewer'] in REVIEWERS:
            if op['reviewer'] not in b['approvals']:
                b['approvals'].append(op['reviewer'])
    elif kind == 'ci':
        b = state['bundles'][op['bundle']]
        if b['revision'] == op['revision'] and not b['published'] and b['digest'] == op['digest']:
            b['ci'] = op['passed']
    elif kind == 'publish':
        rid = op['request_id']
        if rid not in state['publish_requests']:
            b = state['bundles'][op['bundle']]
            if b['revision'] != op['revision']:
                result = 'stale'
            elif b['published']:
                result = 'published'
            elif len(set(b['approvals']) & REVIEWERS) < 2 or not b['ci']:
                result = 'not_ready'
            elif any(state['heads'][branch] != pair['expected'] for branch, pair in b['updates'].items()):
                result = 'conflict'
            else:
                for branch, pair in b['updates'].items():
                    state['heads'][branch] = pair['new']
                b['published'] = True
                result = 'published'
            state['publish_requests'][rid] = result
    return state


class Service:
    def __init__(self, store, initial_heads):
        self.store = store
        self.initial_heads = dict(initial_heads)

    async def _ensure(self):
        version, root = await self.store.read(ROOT)
        if version:
            return version, root
        event = {'prev': None, 'op': {'kind': 'genesis', 'heads': self.initial_heads}}
        eid = _event_id(event)
        key = PREFIX + eid
        v, _ = await self.store.read(key)
        if not v:
            await self.store.cas(key, 0, event)
        # Genesis is known and immutable; initialize state pointer with one CAS.
        await self.store.cas(ROOT, 0, eid)
        return await self.store.read(ROOT)

    async def _state_at(self, eid):
        chain = []
        while eid is not None:
            _, event = await self.store.read(PREFIX + eid)
            chain.append(event)
            eid = event['prev']
        genesis = chain.pop()
        state = _blank(genesis['op']['heads'])
        for event in reversed(chain):
            _apply(state, event['op'])
        return state

    async def _mutate(self, op, decide):
        while True:
            version, eid = await self._ensure()
            state = await self._state_at(eid)
            result, should_write = decide(state)
            if not should_write:
                return result
            event = {'prev': eid, 'op': op}
            new_id = _event_id(event)
            key = PREFIX + new_id
            ev, existing = await self.store.read(key)
            if not ev:
                await self.store.cas(key, 0, event)
            elif existing != event:
                continue
            if await self.store.cas(ROOT, version, new_id):
                return result

    async def submit(self, request_id, updates):
        ups = json.loads(_enc(updates))
        dg = digest(ups)
        op = {'kind': 'submit', 'bundle': request_id, 'updates': ups, 'digest': dg}
        def decide(s):
            b = s['bundles'].get(request_id)
            if b:
                return {'bundle': request_id, 'revision': 1, 'digest': b['digest']}, False
            return {'bundle': request_id, 'revision': 1, 'digest': dg}, True
        return await self._mutate(op, decide)

    async def revise(self, bundle, expected_revision, updates):
        ups = json.loads(_enc(updates)); dg = digest(ups)
        op = {'kind':'revise','bundle':bundle,'expected':expected_revision,'updates':ups,'digest':dg}
        def decide(s):
            b=s['bundles'][bundle]
            if b['revision'] != expected_revision: return 'stale', False
            if b['published']: return 'published', False
            return 'updated', True
        return await self._mutate(op, decide)

    async def approve(self, bundle, revision, reviewer):
        op={'kind':'approve','bundle':bundle,'revision':revision,'reviewer':reviewer}
        def decide(s):
            b=s['bundles'][bundle]
            ok = reviewer in REVIEWERS and b['revision']==revision and not b['published']
            if not ok: return False, False
            return True, reviewer not in b['approvals']
        return await self._mutate(op, decide)

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        op={'kind':'ci','bundle':bundle,'revision':revision,'digest':candidate_digest,'passed':bool(passed)}
        def decide(s):
            b=s['bundles'][bundle]
            ok=b['revision']==revision and not b['published'] and b['digest']==candidate_digest
            if not ok: return False, False
            return True, b['ci'] != bool(passed)
        return await self._mutate(op, decide)

    async def publish(self, bundle, revision, request_id):
        op={'kind':'publish','bundle':bundle,'revision':revision,'request_id':request_id}
        def decide(s):
            if request_id in s['publish_requests']: return s['publish_requests'][request_id], False
            # Publish requests always append, including rejected outcomes.
            b=s['bundles'][bundle]
            if b['revision'] != revision: result='stale'
            elif b['published']: result='published'
            elif len(set(b['approvals']) & REVIEWERS)<2 or not b['ci']: result='not_ready'
            elif any(s['heads'][br] != p['expected'] for br,p in b['updates'].items()): result='conflict'
            else: result='published'
            return result, True
        return await self._mutate(op, decide)

    async def read_heads(self, branches):
        _, eid = await self._ensure()
        s = await self._state_at(eid)
        return {b:s['heads'][b] for b in branches}

    async def get_status(self, bundle):
        _, eid = await self._ensure()
        b=(await self._state_at(eid))['bundles'][bundle]
        return {'revision':b['revision'], 'updates':json.loads(_enc(b['updates'])),
                'digest':b['digest'], 'approvals':sorted(b['approvals']),
                'ci':b['ci'], 'published':b['published']}
