import hashlib
import json

from contract import digest

_ROOT_KEY = 'certified-change-bundles:root'
_BLOB_PREFIX = 'certified-change-bundles:blob:'
_AUTHORIZED = {'alice', 'bob', 'carol'}
_DEPTH = 64


def _encode(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=True)


def _copy(value):
    return json.loads(_encode(value))


def _path(key):
    return hashlib.sha256(key.encode('utf-8')).hexdigest()


class Service:
    def __init__(self, store, initial_heads):
        self.store = store
        self._initial_heads = _copy(initial_heads)

    async def _put_blob(self, value):
        raw = _encode(value)
        ident = hashlib.sha256(raw.encode('utf-8')).hexdigest()
        await self.store.cas(_BLOB_PREFIX + ident, 0, value)
        return ident

    async def _load_blob(self, ident):
        _, value = await self.store.read(_BLOB_PREFIX + ident)
        return value

    async def _store_node(self, node):
        return await self._put_blob(node)

    async def _trie_get(self, root, key):
        if root is None:
            return None
        route = _path(key)
        ident = root
        for depth in range(_DEPTH):
            node = await self._load_blob(ident)
            ident = node['c'].get(route[depth])
            if ident is None:
                return None
        node = await self._load_blob(ident)
        entry = node.get('v')
        if entry is None or entry[0] != key:
            return None
        return entry[1]

    async def _trie_set_at(self, ident, route, depth, key, value):
        if depth == _DEPTH:
            node = {} if ident is None else await self._load_blob(ident)
            node = dict(node)
            node['v'] = [key, value]
            return await self._store_node(node)
        node = {'c': {}} if ident is None else await self._load_blob(ident)
        children = dict(node.get('c', {}))
        slot = route[depth]
        children[slot] = await self._trie_set_at(children.get(slot), route, depth + 1, key, value)
        return await self._store_node({'c': children})

    async def _trie_set(self, root, key, value):
        return await self._trie_set_at(root, _path(key), 0, key, value)

    async def _trie_build_at(self, entries, depth):
        if depth == _DEPTH:
            # SHA-256 collisions are not expected; retain the original key to detect one.
            if len(entries) != 1:
                raise ValueError('key hash collision')
            key, value = entries[0]
            return await self._store_node({'v': [key, value]})
        groups = {}
        for key, value in entries:
            groups.setdefault(_path(key)[depth], []).append((key, value))
        children = {}
        for slot, group in groups.items():
            children[slot] = await self._trie_build_at(group, depth + 1)
        return await self._store_node({'c': children})

    async def _trie_build(self, mapping):
        if not mapping:
            return None
        return await self._trie_build_at(list(mapping.items()), 0)

    async def _trie_items(self, root):
        result = {}
        if root is None:
            return result
        stack = [root]
        while stack:
            ident = stack.pop()
            node = await self._load_blob(ident)
            if 'v' in node:
                key, value = node['v']
                result[key] = value
            else:
                stack.extend(node['c'].values())
        return result

    async def _root(self):
        version, value = await self.store.read(_ROOT_KEY)
        if version:
            return version, value
        heads = await self._trie_build(self._initial_heads)
        candidate = {'h': heads, 'b': None, 's': None, 'p': None}
        if await self.store.cas(_ROOT_KEY, 0, candidate):
            return await self.store.read(_ROOT_KEY)
        # Another instance initialized the root. Its state is authoritative.
        return await self.store.read(_ROOT_KEY)

    async def _change(self, mutate):
        while True:
            version, root = await self._root()
            new_root, result = await mutate(root)
            if new_root is None:
                return result
            if await self.store.cas(_ROOT_KEY, version, new_root):
                return result

    async def submit(self, request_id, updates):
        updates = _copy(updates)
        bundle_digest = digest(updates)
        response = {'bundle': request_id, 'revision': 1, 'digest': bundle_digest}

        async def mutate(root):
            old = await self._trie_get(root['s'], request_id)
            if old is not None:
                return None, old
            bundle = {
                'revision': 1,
                'updates': updates,
                'digest': bundle_digest,
                'approvals': [],
                'ci': False,
                'published': False,
            }
            bundles = await self._trie_set(root['b'], request_id, bundle)
            submissions = await self._trie_set(root['s'], request_id, response)
            changed = dict(root)
            changed['b'] = bundles
            changed['s'] = submissions
            return changed, response

        return _copy(await self._change(mutate))

    async def revise(self, bundle, expected_revision, updates):
        updates = _copy(updates)

        async def mutate(root):
            record = await self._trie_get(root['b'], bundle)
            if record['revision'] != expected_revision:
                return None, 'stale'
            if record['published']:
                return None, 'published'
            changed_record = {
                'revision': record['revision'] + 1,
                'updates': updates,
                'digest': digest(updates),
                'approvals': [],
                'ci': False,
                'published': False,
            }
            bundles = await self._trie_set(root['b'], bundle, changed_record)
            changed = dict(root)
            changed['b'] = bundles
            return changed, 'updated'

        return await self._change(mutate)

    async def approve(self, bundle, revision, reviewer):
        if reviewer not in _AUTHORIZED:
            return False

        async def mutate(root):
            record = await self._trie_get(root['b'], bundle)
            if record['revision'] != revision or record['published']:
                return None, False
            approvals = set(record['approvals'])
            approvals.add(reviewer)
            changed_record = dict(record)
            changed_record['approvals'] = sorted(approvals)
            bundles = await self._trie_set(root['b'], bundle, changed_record)
            changed = dict(root)
            changed['b'] = bundles
            return changed, True

        return await self._change(mutate)

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        async def mutate(root):
            record = await self._trie_get(root['b'], bundle)
            if (record['revision'] != revision or record['digest'] != candidate_digest
                    or record['published']):
                return None, False
            changed_record = dict(record)
            changed_record['ci'] = bool(passed)
            bundles = await self._trie_set(root['b'], bundle, changed_record)
            changed = dict(root)
            changed['b'] = bundles
            return changed, True

        return await self._change(mutate)

    async def publish(self, bundle, revision, request_id):
        async def mutate(root):
            prior = await self._trie_get(root['p'], request_id)
            if prior is not None:
                return None, prior['outcome']
            record = await self._trie_get(root['b'], bundle)
            if record['revision'] != revision:
                outcome = 'stale'
            elif record['published']:
                outcome = 'published'
            elif len(set(record['approvals']) & _AUTHORIZED) < 2 or not record['ci']:
                outcome = 'not_ready'
            else:
                heads = await self._trie_items(root['h'])
                conflict = any(heads[branch] != update['expected']
                               for branch, update in record['updates'].items())
                if conflict:
                    outcome = 'conflict'
                else:
                    for branch, update in record['updates'].items():
                        heads[branch] = update['new']
                    new_heads = await self._trie_build(heads)
                    changed_record = dict(record)
                    changed_record['published'] = True
                    new_bundles = await self._trie_set(root['b'], bundle, changed_record)
                    outcome = 'published'
                    changed = dict(root)
                    changed['h'] = new_heads
                    changed['b'] = new_bundles
                    changed['p'] = await self._trie_set(root['p'], request_id, {'outcome': outcome})
                    return changed, outcome
            changed = dict(root)
            changed['p'] = await self._trie_set(root['p'], request_id, {'outcome': outcome})
            return changed, outcome

        return await self._change(mutate)

    async def read_heads(self, branches):
        _, root = await self._root()
        result = {}
        for branch in branches:
            result[branch] = await self._trie_get(root['h'], branch)
        return result

    async def get_status(self, bundle):
        _, root = await self._root()
        record = await self._trie_get(root['b'], bundle)
        return _copy({
            'revision': record['revision'],
            'updates': record['updates'],
            'digest': record['digest'],
            'approvals': sorted(record['approvals']),
            'ci': record['ci'],
            'published': record['published'],
        })
