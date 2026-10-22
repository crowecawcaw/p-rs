import hashlib
import json

from contract import digest


_ROOT = 'certified-change-bundles/root'
_PREFIX = 'certified-change-bundles/node/'


def _canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=True)


def _node_key(parent, event):
    raw = _canonical({'parent': parent, 'event': event}).encode('utf-8')
    return _PREFIX + hashlib.sha256(raw).hexdigest()


class Service:
    def __init__(self, store, initial_heads):
        self.store = store
        # Copy caller-owned data; initialization itself remains lazy.
        self._initial_heads = dict(initial_heads)

    async def _ensure_root(self):
        version, tail = await self.store.read(_ROOT)
        if version:
            return version, tail

        genesis = {'kind': 'genesis', 'heads': self._initial_heads}
        key = _node_key('', genesis)
        await self.store.cas(key, 0, genesis)
        # If another initializer won, the deterministic key contains the same
        # value. The root CAS decides which initialization becomes visible.
        await self.store.cas(_ROOT, 0, key)
        return await self.store.read(_ROOT)

    async def _snapshot(self):
        version, tail = await self._ensure_root()
        events = []
        cursor = tail
        while cursor:
            _, record = await self.store.read(cursor)
            if record is None:
                # A root is only published after its immutable node is written.
                # This branch is unreachable for a correct durable store.
                raise RuntimeError('missing committed history record')
            if record.get('kind') == 'genesis':
                state = {
                    'heads': dict(record['heads']),
                    'bundles': {},
                    'publish_results': {},
                }
                break
            events.append(record['event'])
            cursor = record['parent']
        else:
            raise RuntimeError('history has no genesis record')

        for event in reversed(events):
            kind = event['type']
            if kind == 'submit':
                state['bundles'][event['bundle']] = {
                    'revision': 1,
                    'updates': event['updates'],
                    'digest': event['digest'],
                    'approvals': [],
                    'ci': False,
                    'published': False,
                    '_submit_digest': event['digest'],
                }
            elif kind == 'revise':
                b = state['bundles'][event['bundle']]
                b.update({
                    'revision': event['revision'],
                    'updates': event['updates'],
                    'digest': event['digest'],
                    'approvals': [],
                    'ci': False,
                })
            elif kind == 'approve':
                b = state['bundles'][event['bundle']]
                if event['reviewer'] not in b['approvals']:
                    b['approvals'].append(event['reviewer'])
            elif kind == 'ci':
                state['bundles'][event['bundle']]['ci'] = event['passed']
            elif kind == 'publish':
                if event['result'] == 'published' and event['apply']:
                    state['heads'].update(event['heads'])
                    state['bundles'][event['bundle']]['published'] = True
                state['publish_results'][event['request_id']] = event['result']
            else:
                raise RuntimeError('unknown history event')
        return version, tail, state

    async def _append(self, version, tail, event):
        record = {'parent': tail, 'event': event}
        key = _node_key(tail, event)
        # A failed immutable insert is harmless: either an identical retry
        # already wrote the node, or it is an unreachable orphan from a race.
        await self.store.cas(key, 0, record)
        return await self.store.cas(_ROOT, version, key)

    async def submit(self, request_id, updates):
        updates_copy = json.loads(_canonical(updates))
        while True:
            version, tail, state = await self._snapshot()
            existing = state['bundles'].get(request_id)
            if existing is not None:
                return {
                    'bundle': request_id,
                    'revision': 1,
                    'digest': self._original_submit_digest(state, request_id),
                }
            d = digest(updates_copy)
            event = {'type': 'submit', 'bundle': request_id,
                     'updates': updates_copy, 'digest': d}
            if await self._append(version, tail, event):
                return {'bundle': request_id, 'revision': 1, 'digest': d}

    @staticmethod
    def _original_submit_digest(state, bundle):
        # The original response digest remains the digest of revision one.
        # It is retained explicitly during replay as a private state field.
        return state['bundles'][bundle]['_submit_digest']

    async def revise(self, bundle, expected_revision, updates):
        updates_copy = json.loads(_canonical(updates))
        while True:
            version, tail, state = await self._snapshot()
            b = state['bundles'][bundle]
            if expected_revision != b['revision']:
                return 'stale'
            if b['published']:
                return 'published'
            revision = b['revision'] + 1
            event = {'type': 'revise', 'bundle': bundle, 'revision': revision,
                     'updates': updates_copy, 'digest': digest(updates_copy)}
            if await self._append(version, tail, event):
                return 'updated'

    async def approve(self, bundle, revision, reviewer):
        if reviewer not in ('alice', 'bob', 'carol'):
            return False
        while True:
            version, tail, state = await self._snapshot()
            b = state['bundles'][bundle]
            if revision != b['revision'] or b['published']:
                return False
            if reviewer in b['approvals']:
                return True
            event = {'type': 'approve', 'bundle': bundle, 'reviewer': reviewer}
            if await self._append(version, tail, event):
                return True

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        while True:
            version, tail, state = await self._snapshot()
            b = state['bundles'][bundle]
            if (revision != b['revision'] or candidate_digest != b['digest']
                    or b['published']):
                return False
            event = {'type': 'ci', 'bundle': bundle, 'passed': bool(passed)}
            if await self._append(version, tail, event):
                return True

    async def publish(self, bundle, revision, request_id):
        while True:
            version, tail, state = await self._snapshot()
            if request_id in state['publish_results']:
                return state['publish_results'][request_id]
            b = state['bundles'][bundle]
            if revision != b['revision']:
                result, apply = 'stale', False
            elif b['published']:
                result, apply = 'published', False
            elif len(set(b['approvals']).intersection(('alice', 'bob', 'carol'))) < 2 or not b['ci']:
                result, apply = 'not_ready', False
            elif any(state['heads'][branch] != update['expected']
                     for branch, update in b['updates'].items()):
                result, apply = 'conflict', False
            else:
                result, apply = 'published', True
            new_heads = ({branch: update['new']
                          for branch, update in b['updates'].items()} if apply else {})
            event = {'type': 'publish', 'bundle': bundle,
                     'request_id': request_id, 'result': result,
                     'apply': apply, 'heads': new_heads}
            if await self._append(version, tail, event):
                return result

    async def read_heads(self, branches):
        _, _, state = await self._snapshot()
        return {branch: state['heads'][branch] for branch in branches}

    async def get_status(self, bundle):
        _, _, state = await self._snapshot()
        b = state['bundles'][bundle]
        return {
            'revision': b['revision'],
            'updates': json.loads(_canonical(b['updates'])),
            'digest': b['digest'],
            'approvals': sorted(b['approvals']),
            'ci': b['ci'],
            'published': b['published'],
        }
