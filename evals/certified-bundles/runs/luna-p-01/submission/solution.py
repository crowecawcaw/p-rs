import hashlib
import json
from contract import digest

_AUTH = {'alice', 'bob', 'carol'}
_ROOT = 'ccb:root'

class Service:
    def __init__(self, store, initial_heads):
        self.store = store
        self.initial_heads = dict(initial_heads)

    @staticmethod
    def _enc(value):
        return json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=True)

    async def _get_node(self, node_id):
        _, node = await self.store.read('ccb:node:' + node_id)
        return node

    async def _put_node(self, node):
        raw = self._enc(node)
        ident = hashlib.sha256(raw.encode()).hexdigest()
        key = 'ccb:node:' + ident
        await self.store.cas(key, 0, node)
        return ident

    @staticmethod
    def _path(key):
        return hashlib.sha256(key.encode()).hexdigest()

    async def _lookup(self, root, key):
        if not root:
            return None
        path = self._path(key)
        node_id = root
        for depth, digit in enumerate(path):
            node = await self._get_node(node_id)
            if not node:
                return None
            node_id = node['c'].get(digit)
            if node_id is None:
                return None
        node = await self._get_node(node_id)
        if not node or node.get('k') != key:
            return None
        return node.get('v')

    async def _set(self, root, key, value):
        path = self._path(key)
        async def rec(node_id, depth):
            if depth == len(path):
                return await self._put_node({'k': key, 'v': value})
            node = await self._get_node(node_id) if node_id else None
            children = dict(node['c']) if node else {}
            d = path[depth]
            children[d] = await rec(children.get(d), depth + 1)
            return await self._put_node({'c': children})
        return await rec(root, 0)

    async def _snapshot(self):
        while True:
            ver, root = await self.store.read(_ROOT)
            if ver:
                return ver, root or ''
            if await self.store.cas(_ROOT, 0, ''):
                continue

    async def _transact(self, fn):
        while True:
            version, root = await self._snapshot()
            result, changes = await fn(root)
            if not changes:
                return result
            new_root = root
            for key, value in changes.items():
                new_root = await self._set(new_root, key, value)
            if await self.store.cas(_ROOT, version, new_root):
                return result

    async def submit(self, request_id, updates):
        d = digest(updates)
        async def op(root):
            existing = await self._lookup(root, 'b:' + request_id)
            if existing is not None:
                return {'bundle': request_id, 'revision': 1, 'digest': existing['origin_digest']}, {}
            bundle = {'revision': 1, 'updates': updates, 'digest': d,
                      'origin_digest': d, 'approvals': [], 'ci': False, 'published': False}
            return {'bundle': request_id, 'revision': 1, 'digest': d}, {'b:' + request_id: bundle}
        return await self._transact(op)

    async def revise(self, bundle, expected_revision, updates):
        async def op(root):
            b = await self._lookup(root, 'b:' + bundle)
            if b['revision'] != expected_revision: return 'stale', {}
            if b['published']: return 'published', {}
            nb = dict(b); nb.update(revision=b['revision'] + 1, updates=updates, digest=digest(updates), approvals=[], ci=False)
            return 'updated', {'b:' + bundle: nb}
        return await self._transact(op)

    async def approve(self, bundle, revision, reviewer):
        if reviewer not in _AUTH: return False
        async def op(root):
            b = await self._lookup(root, 'b:' + bundle)
            if b['revision'] != revision or b['published']: return False, {}
            approvals = sorted(set(b['approvals']) | {reviewer})
            nb = dict(b); nb['approvals'] = approvals
            return True, ({'b:' + bundle: nb} if approvals != b['approvals'] else {})
        return await self._transact(op)

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        async def op(root):
            b = await self._lookup(root, 'b:' + bundle)
            if b['revision'] != revision or b['published'] or candidate_digest != b['digest']: return False, {}
            nb = dict(b); nb['ci'] = bool(passed)
            return True, {'b:' + bundle: nb}
        return await self._transact(op)

    async def publish(self, bundle, revision, request_id):
        async def op(root):
            pk = 'p:' + request_id
            prior = await self._lookup(root, pk)
            if prior is not None: return prior, {}
            b = await self._lookup(root, 'b:' + bundle)
            if b['revision'] != revision: result = 'stale'
            elif b['published']: result = 'published'
            elif len(set(b['approvals']) & _AUTH) < 2 or not b['ci']: result = 'not_ready'
            else:
                conflict = False
                for branch, item in b['updates'].items():
                    head = await self._lookup(root, 'h:' + branch)
                    if (self.initial_heads[branch] if head is None else head) != item['expected']:
                        conflict = True
                if conflict: result = 'conflict'
                else:
                    result = 'published'
                    heads = {branch: item['new'] for branch, item in b['updates'].items()}
                    nb = dict(b); nb['published'] = True
                    ch = {'b:' + bundle: nb, pk: result}
                    ch.update({'h:' + branch: head for branch, head in heads.items()})
                    return result, ch
            return result, {pk: result}
        return await self._transact(op)

    async def read_heads(self, branches):
        _, root = await self._snapshot()
        result = {}
        for branch in branches:
            value = await self._lookup(root, 'h:' + branch)
            result[branch] = self.initial_heads[branch] if value is None else value
        return result

    async def get_status(self, bundle):
        _, root = await self._snapshot()
        b = await self._lookup(root, 'b:' + bundle)
        return {k: b[k] for k in ('revision', 'updates', 'digest', 'approvals', 'ci', 'published')}
