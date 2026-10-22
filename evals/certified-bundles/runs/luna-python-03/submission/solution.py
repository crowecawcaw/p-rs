import hashlib
import json
from contract import digest

_ROOT = 'certified-change-bundles/root'
_PREFIX = 'certified-change-bundles/record/'
_ALLOWED = {'alice', 'bob', 'carol'}


def _clone(value):
    return json.loads(json.dumps(value, ensure_ascii=True, separators=(',', ':')))


def _canon(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=True)


class Service:
    def __init__(self, store, initial_heads):
        self.store = store
        self.initial_heads = _clone(initial_heads)

    async def _tip(self):
        ver, root = await self.store.read(_ROOT)
        if root is None:
            genesis = {'prev': None, 'kind': 'genesis', 'data': {'heads': self.initial_heads}}
            key = _PREFIX + hashlib.sha256(_canon(genesis).encode()).hexdigest()
            ok = await self.store.cas(key, 0, genesis)
            if not ok:
                _, existing = await self.store.read(key)
                if existing != genesis:
                    raise RuntimeError('record hash collision')
            ok = await self.store.cas(_ROOT, ver, {'tip': key})
            if not ok:
                return await self._tip()
            return await self.store.read(_ROOT)
        return ver, root

    async def _snapshot(self):
        ver, root = await self._tip()
        key = root['tip']
        records = []
        while key is not None:
            _, rec = await self.store.read(key)
            if rec is None:
                raise RuntimeError('missing committed record')
            records.append(rec)
            key = rec['prev']
        records.reverse()
        heads = {}
        bundles = {}
        publish_ids = {}
        for rec in records:
            kind, data = rec['kind'], rec['data']
            if kind == 'genesis':
                heads = _clone(data['heads'])
            elif kind == 'submit':
                bid = data['bundle']
                if bid not in bundles:
                    updates = _clone(data['updates'])
                    bundles[bid] = {'revision': 1, 'updates': updates, 'digest': digest(updates), 'approvals': set(), 'ci': False, 'published': False}
            elif kind == 'revise':
                b = bundles[data['bundle']]
                b['revision'] = data['revision']
                b['updates'] = _clone(data['updates'])
                b['digest'] = digest(b['updates'])
                b['approvals'] = set()
                b['ci'] = False
            elif kind == 'approve':
                bundles[data['bundle']]['approvals'].add(data['reviewer'])
            elif kind == 'ci':
                bundles[data['bundle']]['ci'] = data['passed']
            elif kind == 'publish':
                publish_ids[data['request_id']] = data['outcome']
                if data['outcome'] == 'published' and data.get('applied'):
                    b = bundles[data['bundle']]
                    for branch, change in b['updates'].items():
                        heads[branch] = change['new']
                    b['published'] = True
        return ver, root['tip'], heads, bundles, publish_ids

    async def _commit(self, basever, basetip, kind, data):
        rec = {'prev': basetip, 'kind': kind, 'data': _clone(data)}
        encoded = _canon(rec).encode()
        key = _PREFIX + hashlib.sha256(encoded).hexdigest()
        ok = await self.store.cas(key, 0, rec)
        if not ok:
            _, old = await self.store.read(key)
            if old != rec:
                raise RuntimeError('record hash collision')
        return await self.store.cas(_ROOT, basever, {'tip': key})

    async def submit(self, request_id, updates):
        updates = _clone(updates)
        while True:
            ver, tip, _, bundles, _ = await self._snapshot()
            if request_id in bundles:
                b = bundles[request_id]
                return {'bundle': request_id, 'revision': 1, 'digest': b['digest'] if b['revision'] == 1 else await self._submitted_digest(request_id, tip)}
            data = {'bundle': request_id, 'updates': updates}
            if await self._commit(ver, tip, 'submit', data):
                return {'bundle': request_id, 'revision': 1, 'digest': digest(updates)}

    async def _submitted_digest(self, request_id, tip):
        # Walk back to the immutable original submission after later revisions.
        key = tip
        while key is not None:
            _, rec = await self.store.read(key)
            if rec['kind'] == 'submit' and rec['data']['bundle'] == request_id:
                return digest(rec['data']['updates'])
            key = rec['prev']
        raise RuntimeError('missing submission')

    async def revise(self, bundle, expected_revision, updates):
        updates = _clone(updates)
        while True:
            ver, tip, _, bundles, _ = await self._snapshot()
            b = bundles[bundle]
            if expected_revision != b['revision']:
                return 'stale'
            if b['published']:
                return 'published'
            data = {'bundle': bundle, 'revision': b['revision'] + 1, 'updates': updates}
            if await self._commit(ver, tip, 'revise', data):
                return 'updated'

    async def approve(self, bundle, revision, reviewer):
        if reviewer not in _ALLOWED:
            return False
        while True:
            ver, tip, _, bundles, _ = await self._snapshot()
            b = bundles[bundle]
            if revision != b['revision'] or b['published']:
                return False
            if reviewer in b['approvals']:
                return True
            if await self._commit(ver, tip, 'approve', {'bundle': bundle, 'reviewer': reviewer}):
                return True

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        while True:
            ver, tip, _, bundles, _ = await self._snapshot()
            b = bundles[bundle]
            if revision != b['revision'] or candidate_digest != b['digest'] or b['published']:
                return False
            passed = bool(passed)
            if b['ci'] == passed:
                return True
            if await self._commit(ver, tip, 'ci', {'bundle': bundle, 'passed': passed}):
                return True

    async def publish(self, bundle, revision, request_id):
        while True:
            ver, tip, heads, bundles, outcomes = await self._snapshot()
            if request_id in outcomes:
                return outcomes[request_id]
            b = bundles[bundle]
            if revision != b['revision']:
                outcome = 'stale'
            elif b['published']:
                outcome = 'published'
            elif len(b['approvals']) < 2 or not b['ci']:
                outcome = 'not_ready'
            elif any(heads[branch] != change['expected'] for branch, change in b['updates'].items()):
                outcome = 'conflict'
            else:
                outcome = 'published'
            applied = outcome == 'published' and not b['published']
            data = {'bundle': bundle, 'revision': revision, 'request_id': request_id, 'outcome': outcome, 'applied': applied}
            if await self._commit(ver, tip, 'publish', data):
                return outcome

    async def read_heads(self, branches):
        _, _, heads, _, _ = await self._snapshot()
        return {branch: heads[branch] for branch in branches}

    async def get_status(self, bundle):
        _, _, _, bundles, _ = await self._snapshot()
        b = bundles[bundle]
        return {'revision': b['revision'], 'updates': _clone(b['updates']), 'digest': b['digest'], 'approvals': sorted(b['approvals']), 'ci': b['ci'], 'published': b['published']}
