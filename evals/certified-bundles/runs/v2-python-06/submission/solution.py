import hashlib
import json

from contract import digest


def _clone(value):
    return json.loads(json.dumps(value, ensure_ascii=True, separators=(',', ':')))


def _json(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=True)


class Service:
    """A small persistent, copy-on-write database over the store's CAS API."""

    _ROOT = 'certified-change-bundles:root'
    _TABLES = ('heads', 'bundles', 'submits', 'publishes')
    _REVIEWERS = frozenset(('alice', 'bob', 'carol'))

    def __init__(self, store, initial_heads):
        self.store = store
        self._initial_heads = _clone(initial_heads)

    async def _put_node(self, node):
        encoded = _json(node)
        node_id = hashlib.sha256(encoded.encode('utf-8')).hexdigest()
        key = 'certified-change-bundles:node:' + node_id
        version, current = await self.store.read(key)
        if version == 0:
            if await self.store.cas(key, 0, node):
                return key
            version, current = await self.store.read(key)
        # Nodes are content addressed. A collision or corrupt store is not a
        # recoverable condition, and silently accepting it would break safety.
        if current != node:
            raise RuntimeError('content-addressed node mismatch')
        return key

    async def _get_node(self, ref):
        _, node = await self.store.read(ref)
        return node

    @staticmethod
    def _path(key):
        return hashlib.sha256(key.encode('utf-8')).hexdigest()[:16]

    async def _map_get(self, root_ref, key):
        path = self._path(key)
        ref = root_ref
        for depth, digit in enumerate(path):
            if ref is None:
                return None
            node = await self._get_node(ref)
            ref = node.get('c', {}).get(digit)
        if ref is None:
            return None
        leaf = await self._get_node(ref)
        for found_key, value in leaf.get('l', []):
            if found_key == key:
                return value
        return None

    async def _map_set(self, root_ref, key, value):
        path = self._path(key)
        refs = [root_ref]
        nodes = []
        ref = root_ref
        for digit in path:
            node = await self._get_node(ref) if ref is not None else None
            nodes.append(node)
            ref = node.get('c', {}).get(digit) if node is not None else None
            refs.append(ref)
        leaf = await self._get_node(ref) if ref is not None else {'l': []}
        entries = list(leaf.get('l', []))
        for index, (found_key, _) in enumerate(entries):
            if found_key == key:
                entries[index] = [key, value]
                break
        else:
            entries.append([key, value])
        child_ref = await self._put_node({'l': entries})
        for depth in range(len(path) - 1, -1, -1):
            node = nodes[depth]
            children = dict(node.get('c', {})) if node is not None else {}
            children[path[depth]] = child_ref
            child_ref = await self._put_node({'c': children})
        return child_ref

    async def _root(self):
        while True:
            version, value = await self.store.read(self._ROOT)
            if value is not None:
                return version, value
            heads = None
            for branch, commit in self._initial_heads.items():
                heads = await self._map_set(heads, branch, commit)
            initial = {'heads': heads, 'bundles': None, 'submits': None, 'publishes': None}
            if await self.store.cas(self._ROOT, version, initial):
                return version + 1, initial

    async def _commit(self, version, root):
        return await self.store.cas(self._ROOT, version, root)

    async def submit(self, request_id, updates):
        updates = _clone(updates)
        new_bundle = {
            'revision': 1,
            'updates': updates,
            'digest': digest(updates),
            'approvals': [],
            'ci': False,
            'published': False,
        }
        response = {'bundle': request_id, 'revision': 1, 'digest': new_bundle['digest']}
        while True:
            version, root = await self._root()
            prior = await self._map_get(root['submits'], request_id)
            if prior is not None:
                return _clone(prior)
            bundles = await self._map_set(root['bundles'], request_id, new_bundle)
            submits = await self._map_set(root['submits'], request_id, response)
            updated = dict(root, bundles=bundles, submits=submits)
            if await self._commit(version, updated):
                return _clone(response)

    async def revise(self, bundle, expected_revision, updates):
        updates = _clone(updates)
        while True:
            version, root = await self._root()
            current = await self._map_get(root['bundles'], bundle)
            if expected_revision != current['revision']:
                return 'stale'
            if current['published']:
                return 'published'
            revised = {
                'revision': current['revision'] + 1,
                'updates': updates,
                'digest': digest(updates),
                'approvals': [],
                'ci': False,
                'published': False,
            }
            bundles = await self._map_set(root['bundles'], bundle, revised)
            if await self._commit(version, dict(root, bundles=bundles)):
                return 'updated'

    async def approve(self, bundle, revision, reviewer):
        if reviewer not in self._REVIEWERS:
            return False
        while True:
            version, root = await self._root()
            current = await self._map_get(root['bundles'], bundle)
            if revision != current['revision'] or current['published']:
                return False
            approvals = sorted(set(current['approvals']) | {reviewer})
            if approvals == current['approvals']:
                return True
            revised = dict(current, approvals=approvals)
            bundles = await self._map_set(root['bundles'], bundle, revised)
            if await self._commit(version, dict(root, bundles=bundles)):
                return True

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        while True:
            version, root = await self._root()
            current = await self._map_get(root['bundles'], bundle)
            if (revision != current['revision'] or current['published'] or
                    candidate_digest != current['digest']):
                return False
            revised = dict(current, ci=bool(passed))
            if revised == current:
                return True
            bundles = await self._map_set(root['bundles'], bundle, revised)
            if await self._commit(version, dict(root, bundles=bundles)):
                return True

    async def publish(self, bundle, revision, request_id):
        while True:
            version, root = await self._root()
            prior = await self._map_get(root['publishes'], request_id)
            if prior is not None:
                return prior
            current = await self._map_get(root['bundles'], bundle)
            if revision != current['revision']:
                outcome = 'stale'
            elif current['published']:
                outcome = 'published'
            elif len(current['approvals']) < 2 or not current['ci']:
                outcome = 'not_ready'
            else:
                conflict = False
                for branch, update in current['updates'].items():
                    head = await self._map_get(root['heads'], branch)
                    if head != update['expected']:
                        conflict = True
                        break
                if conflict:
                    outcome = 'conflict'
                else:
                    outcome = 'published'
            heads = root['heads']
            bundles = root['bundles']
            if outcome == 'published' and not current['published'] and revision == current['revision']:
                for branch, update in current['updates'].items():
                    heads = await self._map_set(heads, branch, update['new'])
                bundles = await self._map_set(bundles, bundle, dict(current, published=True))
            publishes = await self._map_set(root['publishes'], request_id, outcome)
            updated = dict(root, heads=heads, bundles=bundles, publishes=publishes)
            if await self._commit(version, updated):
                return outcome

    async def read_heads(self, branches):
        _, root = await self._root()
        result = {}
        for branch in branches:
            result[branch] = await self._map_get(root['heads'], branch)
        return result

    async def get_status(self, bundle):
        _, root = await self._root()
        current = await self._map_get(root['bundles'], bundle)
        return {
            'revision': current['revision'],
            'updates': _clone(current['updates']),
            'digest': current['digest'],
            'approvals': list(current['approvals']),
            'ci': current['ci'],
            'published': current['published'],
        }
