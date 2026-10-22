import hashlib
import json
from contract import digest

_ROOT = 'change-bundles:root'
_PREFIX = 'change-bundles:event:'
_REVIEWERS = {'alice', 'bob', 'carol'}


def _json(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=True)


class Service:
    def __init__(self, store, initial_heads):
        self.store = store
        # The input is immutable by convention, so detach it from caller mutation.
        self._initial_heads = dict(initial_heads)

    async def _snapshot(self):
        """Return one committed view, rebuilding it from the immutable event chain."""
        while True:
            version, root = await self.store.read(_ROOT)
            if root is None:
                initial = {'init': self._initial_heads}
                key = _PREFIX + hashlib.sha256(_json(initial).encode()).hexdigest()
                ev_version, ev = await self.store.read(key)
                if ev_version == 0:
                    await self.store.cas(key, 0, {'prev': None, 'data': initial})
                # A competing initializer may have won; only the root CAS decides.
                if await self.store.cas(_ROOT, 0, {'head': key}):
                    return 1, {'head': key}, {'heads': dict(self._initial_heads), 'bundles': {}, 'submits': {}, 'publishes': {}}
                continue
            head = root['head']
            events = []
            while head is not None:
                _, event = await self.store.read(head)
                events.append(event['data'])
                head = event['prev']
            state = {'heads': {}, 'bundles': {}, 'submits': {}, 'publishes': {}}
            for data in reversed(events):
                if 'init' in data:
                    state['heads'] = dict(data['init'])
                if 'bundle' in data:
                    bid, bundle = data['bundle']
                    state['bundles'][bid] = bundle
                    if 'submit' in data:
                        state['submits'][bid] = data['submit']
                if 'heads' in data:
                    state['heads'].update(data['heads'])
                if 'publish' in data:
                    rid, outcome = data['publish']
                    state['publishes'][rid] = outcome
            return version, root, state

    async def _commit(self, version, root, data):
        key_material = {'prev': root['head'], 'data': data}
        key = _PREFIX + hashlib.sha256(_json(key_material).encode()).hexdigest()
        event_value = {'prev': root['head'], 'data': data}
        ev_version, _ = await self.store.read(key)
        if ev_version == 0:
            await self.store.cas(key, 0, event_value)
        return await self.store.cas(_ROOT, version, {'head': key})

    async def submit(self, request_id, updates):
        updates = json.loads(_json(updates))
        d = digest(updates)
        while True:
            snap = await self._snapshot()
            version, root, state = snap
            if request_id in state['bundles']:
                original = state['submits'][request_id]
                return {'bundle': request_id, 'revision': 1, 'digest': original}
            bundle = {'revision': 1, 'updates': updates, 'digest': d,
                      'approvals': [], 'ci': False, 'published': False}
            if await self._commit(version, root, {'bundle': [request_id, bundle], 'submit': d}):
                return {'bundle': request_id, 'revision': 1, 'digest': d}

    async def revise(self, bundle, expected_revision, updates):
        updates = json.loads(_json(updates))
        while True:
            version, root, state = await self._snapshot()
            old = state['bundles'][bundle]
            if old['revision'] != expected_revision:
                return 'stale'
            if old['published']:
                return 'published'
            new = {'revision': old['revision'] + 1, 'updates': updates,
                   'digest': digest(updates), 'approvals': [], 'ci': False,
                   'published': False}
            if await self._commit(version, root, {'bundle': [bundle, new]}):
                return 'updated'

    async def approve(self, bundle, revision, reviewer):
        if reviewer not in _REVIEWERS:
            return False
        while True:
            version, root, state = await self._snapshot()
            old = state['bundles'][bundle]
            if old['revision'] != revision or old['published']:
                return False
            if reviewer in old['approvals']:
                return True
            new = dict(old)
            new['approvals'] = sorted(old['approvals'] + [reviewer])
            if await self._commit(version, root, {'bundle': [bundle, new]}):
                return True

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        while True:
            version, root, state = await self._snapshot()
            old = state['bundles'][bundle]
            if (old['revision'] != revision or old['digest'] != candidate_digest
                    or old['published']):
                return False
            new = dict(old)
            new['ci'] = bool(passed)
            if new == old:
                return True
            if await self._commit(version, root, {'bundle': [bundle, new]}):
                return True

    async def publish(self, bundle, revision, request_id):
        while True:
            version, root, state = await self._snapshot()
            if request_id in state['publishes']:
                return state['publishes'][request_id]
            old = state['bundles'][bundle]
            heads_delta = None
            new = None
            if old['revision'] != revision:
                outcome = 'stale'
            elif old['published']:
                outcome = 'published'
            elif len(old['approvals']) < 2 or not old['ci']:
                outcome = 'not_ready'
            elif any(state['heads'][branch] != item['expected']
                     for branch, item in old['updates'].items()):
                outcome = 'conflict'
            else:
                outcome = 'published'
                heads_delta = {branch: item['new'] for branch, item in old['updates'].items()}
                new = dict(old)
                new['published'] = True
            data = {'publish': [request_id, outcome]}
            if new is not None:
                data['bundle'] = [bundle, new]
                data['heads'] = heads_delta
            if await self._commit(version, root, data):
                return outcome

    async def read_heads(self, branches):
        snap = await self._snapshot()
        state = snap[2]
        return {branch: state['heads'][branch] for branch in branches}

    async def get_status(self, bundle):
        state = (await self._snapshot())[2]
        b = state['bundles'][bundle]
        return {'revision': b['revision'], 'updates': json.loads(_json(b['updates'])),
                'digest': b['digest'], 'approvals': list(b['approvals']),
                'ci': b['ci'], 'published': b['published']}
