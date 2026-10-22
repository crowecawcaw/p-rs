"""Linearizable certified change bundles over a single CAS root."""
import copy
import hashlib
import json

from contract import digest

_ROOT_KEY = "certified-bundles:root"
_NODE_PREFIX = "certified-bundles:node:"
_AUTHORIZED = {"alice", "bob", "carol"}
_DEPTH = 16


def _json(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def _key_path(key):
    return hashlib.sha256(key.encode("utf-8")).hexdigest()[:_DEPTH]


class Service:
    def __init__(self, store, initial_heads):
        self.store = store
        self._initial_heads = copy.deepcopy(initial_heads)

    async def _read_node(self, ref):
        if ref is None:
            return {"c": {}, "v": []}
        _, node = await self.store.read(_NODE_PREFIX + ref)
        # Nodes are immutable and are written before a root may reference them.
        return node

    async def _write_node(self, node):
        raw = _json(node)
        ref = hashlib.sha256(raw.encode("ascii")).hexdigest()
        await self.store.cas(_NODE_PREFIX + ref, 0, node)
        return ref

    async def _get(self, root, key):
        path = _key_path(key)
        ref = root
        for depth, slot in enumerate(path):
            node = await self._read_node(ref)
            ref = node["c"].get(slot)
            if ref is None:
                return None
        node = await self._read_node(ref)
        for candidate, value in node["v"]:
            if candidate == key:
                return copy.deepcopy(value)
        return None

    async def _put(self, root, key, value):
        path = _key_path(key)

        async def update(ref, depth):
            node = await self._read_node(ref)
            if depth == _DEPTH:
                entries = [pair for pair in node["v"] if pair[0] != key]
                entries.append([key, copy.deepcopy(value)])
                entries.sort(key=lambda pair: pair[0])
                return await self._write_node({"c": node["c"], "v": entries})
            slot = path[depth]
            children = dict(node["c"])
            children[slot] = await update(children.get(slot), depth + 1)
            return await self._write_node({"c": children, "v": node["v"]})

        return await update(root, 0)

    async def _ensure_initialized(self):
        version, root_value = await self.store.read(_ROOT_KEY)
        if version:
            return
        root = None
        for branch, head in sorted(self._initial_heads.items()):
            root = await self._put(root, "h:" + branch, head)
        if await self.store.cas(_ROOT_KEY, 0, {"map": root}):
            return
        # Another instance initialized first. Its initial heads are guaranteed equal.

    async def _transact(self, operation):
        await self._ensure_initialized()
        while True:
            version, state = await self.store.read(_ROOT_KEY)
            root = state["map"]
            result, writes = await operation(root)
            if not writes:
                return result
            new_root = root
            for key, value in writes.items():
                new_root = await self._put(new_root, key, value)
            if await self.store.cas(_ROOT_KEY, version, {"map": new_root}):
                return result

    async def submit(self, request_id, updates):
        updates_copy = copy.deepcopy(updates)
        candidate_digest = digest(updates_copy)

        async def op(root):
            old = await self._get(root, "b:" + request_id)
            if old is not None:
                return ({"bundle": request_id, "revision": 1, "digest": old["initial_digest"]}, {})
            record = {
                "revision": 1,
                "updates": updates_copy,
                "digest": candidate_digest,
                "initial_digest": candidate_digest,
                "approvals": [],
                "ci": False,
                "published": False,
            }
            response = {"bundle": request_id, "revision": 1, "digest": candidate_digest}
            return response, {"b:" + request_id: record}

        return copy.deepcopy(await self._transact(op))

    async def revise(self, bundle, expected_revision, updates):
        updates_copy = copy.deepcopy(updates)
        candidate_digest = digest(updates_copy)

        async def op(root):
            record = await self._get(root, "b:" + bundle)
            if record["revision"] != expected_revision:
                return "stale", {}
            if record["published"]:
                return "published", {}
            record.update({
                "revision": record["revision"] + 1,
                "updates": updates_copy,
                "digest": candidate_digest,
                "approvals": [],
                "ci": False,
            })
            return "updated", {"b:" + bundle: record}

        return await self._transact(op)

    async def approve(self, bundle, revision, reviewer):
        if reviewer not in _AUTHORIZED:
            return False

        async def op(root):
            record = await self._get(root, "b:" + bundle)
            if record["revision"] != revision or record["published"]:
                return False, {}
            if reviewer in record["approvals"]:
                return True, {}
            record["approvals"].append(reviewer)
            record["approvals"].sort()
            return True, {"b:" + bundle: record}

        return await self._transact(op)

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        async def op(root):
            record = await self._get(root, "b:" + bundle)
            if (record["revision"] != revision or record["published"]
                    or record["digest"] != candidate_digest):
                return False, {}
            record["ci"] = bool(passed)
            return True, {"b:" + bundle: record}

        return await self._transact(op)

    async def publish(self, bundle, revision, request_id):
        memo_key = "p:" + request_id

        async def op(root):
            prior = await self._get(root, memo_key)
            if prior is not None:
                return prior, {}
            record = await self._get(root, "b:" + bundle)
            if record["revision"] != revision:
                outcome = "stale"
            elif record["published"]:
                outcome = "published"
            elif len(set(record["approvals"]) & _AUTHORIZED) < 2 or not record["ci"]:
                outcome = "not_ready"
            else:
                targets = record["updates"]
                heads = {}
                conflict = False
                for branch, change in targets.items():
                    head = await self._get(root, "h:" + branch)
                    heads[branch] = head
                    if head != change["expected"]:
                        conflict = True
                if conflict:
                    outcome = "conflict"
                else:
                    outcome = "published"
                    record["published"] = True
                    # Heads, bundle state, and the request outcome are committed by
                    # one root CAS below, so readers see one atomic publication.
                    writes = {"b:" + bundle: record, memo_key: outcome}
                    writes.update({"h:" + branch: change["new"]
                                   for branch, change in targets.items()})
                    return outcome, writes
            return outcome, {memo_key: outcome}

        return await self._transact(op)

    async def read_heads(self, branches):
        await self._ensure_initialized()
        _, state = await self.store.read(_ROOT_KEY)
        root = state["map"]
        result = {}
        for branch in branches:
            result[branch] = await self._get(root, "h:" + branch)
        return result

    async def get_status(self, bundle):
        await self._ensure_initialized()
        _, state = await self.store.read(_ROOT_KEY)
        record = await self._get(state["map"], "b:" + bundle)
        return {
            "revision": record["revision"],
            "updates": copy.deepcopy(record["updates"]),
            "digest": record["digest"],
            "approvals": sorted(record["approvals"]),
            "ci": record["ci"],
            "published": record["published"],
        }
