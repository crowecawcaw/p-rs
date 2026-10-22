"""Calibration implementation: immutable JSON chunks plus one atomic root pointer."""
import hashlib
import json
from contract import digest

class Service:
    def __init__(self, store, initial_heads):
        self.store = store
        self.initial = dict(initial_heads)

    async def _load(self):
        version, keys = await self.store.read('root')
        if keys is None:
            return version, {'heads': dict(self.initial), 'bundles': {}, 'requests': {}}
        pieces = []
        for key in keys:
            _, piece = await self.store.read(key)
            pieces.append(piece)
        return version, json.loads(''.join(pieces))

    async def _save(self, version, state):
        encoded = json.dumps(state, sort_keys=True, separators=(',', ':'))
        keys = []
        for offset in range(0, len(encoded), 4000):
            piece = encoded[offset:offset+4000]
            key = 'blob/' + hashlib.sha256(piece.encode()).hexdigest()
            await self.store.cas(key, 0, piece)
            keys.append(key)
        return await self.store.cas('root', version, keys)

    async def _change(self, fn):
        while True:
            version, state = await self._load()
            result, changed = fn(state)
            if not changed or await self._save(version, state):
                return result

    async def submit(self, request_id, updates):
        def apply(s):
            if request_id in s['bundles']:
                return s['bundles'][request_id]['original'], False
            result = {'bundle': request_id, 'revision': 1, 'digest': digest(updates)}
            s['bundles'][request_id] = {'revision': 1, 'updates': updates, 'digest': digest(updates),
                'approvals': [], 'ci': False, 'published': False, 'original': result}
            return result, True
        return await self._change(apply)

    async def revise(self, bundle, expected_revision, updates):
        def apply(s):
            b = s['bundles'][bundle]
            if b['revision'] != expected_revision: return 'stale', False
            if b['published']: return 'published', False
            b.update(revision=expected_revision+1, updates=updates, digest=digest(updates), approvals=[], ci=False)
            return 'updated', True
        return await self._change(apply)

    async def approve(self, bundle, revision, reviewer):
        def apply(s):
            b = s['bundles'][bundle]
            if b['revision'] != revision or b['published'] or reviewer not in ('alice','bob','carol'):
                return False, False
            b['approvals'] = sorted(set(b['approvals']) | {reviewer})
            return True, True
        return await self._change(apply)

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        def apply(s):
            b = s['bundles'][bundle]
            if b['revision'] != revision or b['published'] or b['digest'] != candidate_digest:
                return False, False
            b['ci'] = b['ci'] or bool(passed)
            return True, True
        return await self._change(apply)

    async def publish(self, bundle, revision, request_id):
        def apply(s):
            if request_id in s['requests']: return s['requests'][request_id], False
            b = s['bundles'][bundle]
            if b['revision'] != revision: result = 'stale'
            elif b['published']: result = 'published'
            elif len(b['approvals']) < 2 or not b['ci']: result = 'not_ready'
            elif any(s['heads'][k] != u['expected'] for k,u in b['updates'].items()): result = 'conflict'
            else:
                s['heads'].update({k:u['new'] for k,u in b['updates'].items()})
                b['published'] = True
                result = 'published'
            s['requests'][request_id] = result
            return result, True
        return await self._change(apply)

    async def read_heads(self, branches):
        _, state = await self._load()
        return {k: state['heads'][k] for k in branches}

    async def get_status(self, bundle):
        _, state = await self._load()
        return {k:v for k,v in state['bundles'][bundle].items() if k != 'original'}
