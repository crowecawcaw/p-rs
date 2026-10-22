import hashlib
import json

from contract import digest

_ROOT = 'cb:root'
_LOG_PREFIX = 'cb:log:'
_ZERO = None


def _canon(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=True)


def _log_key(record):
    encoded = _canon(record).encode('utf-8')
    return _LOG_PREFIX + hashlib.sha256(encoded).hexdigest()


class Service:
    def __init__(self, store, initial_heads):
        self.store = store
        self._initial_heads = dict(initial_heads)

    def _empty(self):
        return {
            'heads': dict(self._initial_heads),
            'bundles': {},
            'publishes': {},
        }

    def _apply(self, state, event):
        kind = event['kind']
        if kind == 'submit':
            bid = event['bundle']
            state['bundles'][bid] = {
                'revision': 1,
                'updates': event['updates'],
                'digest': event['digest'],
                'approvals': [],
                'ci': False,
                'published': False,
                'initial_response': event['response'],
            }
        elif kind == 'revise':
            b = state['bundles'][event['bundle']]
            b['revision'] = event['revision']
            b['updates'] = event['updates']
            b['digest'] = event['digest']
            b['approvals'] = []
            b['ci'] = False
        elif kind == 'approve':
            b = state['bundles'][event['bundle']]
            if event['reviewer'] not in b['approvals']:
                b['approvals'].append(event['reviewer'])
        elif kind == 'ci':
            state['bundles'][event['bundle']]['ci'] = event['passed']
        elif kind == 'publish':
            state['publishes'][event['request_id']] = event['outcome']
            if event['outcome'] == 'published':
                b = state['bundles'][event['bundle']]
                for branch, change in b['updates'].items():
                    state['heads'][branch] = change['new']
                b['published'] = True
        else:
            raise RuntimeError('unknown log event')

    async def _initialize(self):
        root_version, root = await self.store.read(_ROOT)
        if root is not None:
            return root_version, root
        genesis = {'prev': None, 'event': {'kind': 'genesis', 'heads': self._initial_heads}}
        key = _log_key(genesis)
        written, _ = await self.store.read(key)
        if written == 0:
            await self.store.cas(key, 0, genesis)
        ok = await self.store.cas(_ROOT, 0, key)
        if ok:
            return await self.store.read(_ROOT)
        return await self.store.read(_ROOT)

    async def _snapshot(self):
        root_version, root = await self._initialize()
        # The root is immutable for this snapshot. A later commit can advance the
        # pointer while this chain is read without affecting the captured state.
        records = []
        cursor = root
        while cursor is not None:
            _, record = await self.store.read(cursor)
            if record is None:
                # A committed pointer can only reference a previously durable log
                # node. Retrying handles any transiently lost initialization reply.
                root_version, root = await self._initialize()
                records = []
                cursor = root
                continue
            records.append(record)
            cursor = record['prev']
        state = self._empty()
        for record in reversed(records):
            event = record['event']
            if event['kind'] == 'genesis':
                state['heads'] = dict(event['heads'])
            else:
                self._apply(state, event)
        return root_version, root, state

    async def _append(self, root_version, root, event):
        record = {'prev': root, 'event': event}
        key = _log_key(record)
        key_version, existing = await self.store.read(key)
        if key_version == 0:
            await self.store.cas(key, 0, record)
        elif existing != record:
            raise RuntimeError('content-addressed log collision')
        return await self.store.cas(_ROOT, root_version, key)

    async def submit(self, request_id, updates):
        updates_copy = json.loads(_canon(updates))
        result_digest = digest(updates_copy)
        response = {'bundle': request_id, 'revision': 1, 'digest': result_digest}
        while True:
            version, root, state = await self._snapshot()
            existing = state['bundles'].get(request_id)
            if existing is not None:
                return dict(existing['initial_response'])
            event = {
                'kind': 'submit', 'bundle': request_id, 'updates': updates_copy,
                'digest': result_digest, 'response': response,
            }
            if await self._append(version, root, event):
                return dict(response)

    async def revise(self, bundle, expected_revision, updates):
        updates_copy = json.loads(_canon(updates))
        new_digest = digest(updates_copy)
        while True:
            version, root, state = await self._snapshot()
            b = state['bundles'][bundle]
            if expected_revision != b['revision']:
                return 'stale'
            if b['published']:
                return 'published'
            event = {
                'kind': 'revise', 'bundle': bundle,
                'revision': b['revision'] + 1,
                'updates': updates_copy, 'digest': new_digest,
            }
            if await self._append(version, root, event):
                return 'updated'

    async def approve(self, bundle, revision, reviewer):
        if reviewer not in ('alice', 'bob', 'carol'):
            return False
        while True:
            version, root, state = await self._snapshot()
            b = state['bundles'][bundle]
            if revision != b['revision'] or b['published']:
                return False
            if reviewer in b['approvals']:
                return True
            event = {'kind': 'approve', 'bundle': bundle, 'reviewer': reviewer}
            if await self._append(version, root, event):
                return True

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        while True:
            version, root, state = await self._snapshot()
            b = state['bundles'][bundle]
            if (revision != b['revision'] or candidate_digest != b['digest']
                    or b['published']):
                return False
            passed_value = bool(passed)
            if b['ci'] == passed_value:
                return True
            event = {'kind': 'ci', 'bundle': bundle, 'passed': passed_value}
            if await self._append(version, root, event):
                return True

    async def publish(self, bundle, revision, request_id):
        while True:
            version, root, state = await self._snapshot()
            if request_id in state['publishes']:
                return state['publishes'][request_id]
            b = state['bundles'][bundle]
            if revision != b['revision']:
                outcome = 'stale'
            elif b['published']:
                outcome = 'published'
            elif len(set(b['approvals']).intersection(('alice', 'bob', 'carol'))) < 2 or not b['ci']:
                outcome = 'not_ready'
            elif any(state['heads'][branch] != change['expected']
                     for branch, change in b['updates'].items()):
                outcome = 'conflict'
            else:
                outcome = 'published'
            event = {
                'kind': 'publish', 'request_id': request_id,
                'bundle': bundle, 'outcome': outcome,
            }
            if await self._append(version, root, event):
                return outcome

    async def read_heads(self, branches):
        _, _, state = await self._snapshot()
        return {branch: state['heads'][branch] for branch in branches}

    async def get_status(self, bundle):
        _, _, state = await self._snapshot()
        b = state['bundles'][bundle]
        return {
            'revision': b['revision'],
            'updates': json.loads(_canon(b['updates'])),
            'digest': b['digest'],
            'approvals': sorted(b['approvals']),
            'ci': b['ci'],
            'published': b['published'],
        }
