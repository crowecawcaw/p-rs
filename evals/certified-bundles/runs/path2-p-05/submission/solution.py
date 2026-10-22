"""Durable, linearizable bundle coordination over a CAS-only store."""
import hashlib
import json
from contract import digest


def _json_copy(value):
    return json.loads(json.dumps(value, separators=(",", ":"), ensure_ascii=True))


def _canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


class Service:
    """A persistent treap addressed by content hash, published through one CAS root."""

    _ROOT = "certified-bundles/root-v1"
    _NODE_PREFIX = "certified-bundles/node-v1/"
    _BLOB_PREFIX = "certified-bundles/blob-v1/"

    def __init__(self, store, initial_heads):
        self.store = store
        self._initial_heads = _json_copy(initial_heads)

    @staticmethod
    def _key(kind, ident):
        return _canonical([kind, ident])

    @staticmethod
    def _priority(key):
        return hashlib.sha256(key.encode("utf-8")).hexdigest()

    async def _node(self, node_id):
        if node_id is None:
            return None
        _, node = await self.store.read(self._NODE_PREFIX + node_id)
        return node

    async def _write_immutable(self, prefix, value):
        encoded = _canonical(value)
        item_id = hashlib.sha256(encoded.encode("utf-8")).hexdigest()
        store_key = prefix + item_id
        ok = await self.store.cas(store_key, 0, value)
        if not ok:
            _, existing = await self.store.read(store_key)
            if existing != value:
                raise RuntimeError("content-address collision")
        return item_id

    async def _pack(self, value):
        encoded = _canonical(value)
        if len(encoded.encode("utf-8")) <= 6000:
            return {"i": value}
        # ensure_ascii makes this safe to split by characters and keeps each
        # immutable blob record well below the store's 16 KiB ceiling.
        next_id = None
        chunks = [encoded[i:i + 6000] for i in range(0, len(encoded), 6000)]
        for chunk in reversed(chunks):
            next_id = await self._write_immutable(self._BLOB_PREFIX,
                                                 {"data": chunk, "next": next_id})
        return {"b": next_id}

    async def _unpack(self, packed):
        if "i" in packed:
            return packed["i"]
        cursor = packed["b"]
        pieces = []
        while cursor is not None:
            _, blob = await self.store.read(self._BLOB_PREFIX + cursor)
            pieces.append(blob["data"])
            cursor = blob["next"]
        return json.loads("".join(pieces))

    async def _make(self, key, packed_value, left, right):
        node = {"k": key, "v": packed_value, "l": left, "r": right,
                "p": self._priority(key)}
        return await self._write_immutable(self._NODE_PREFIX, node)

    async def _get(self, root, key):
        cursor = root
        while cursor is not None:
            node = await self._node(cursor)
            if key == node["k"]:
                return await self._unpack(node["v"])
            cursor = node["l"] if key < node["k"] else node["r"]
        return None

    async def _put(self, root, key, value):
        return await self._put_packed(root, key, await self._pack(value))

    async def _put_packed(self, root, key, packed_value):
        if root is None:
            return await self._make(key, packed_value, None, None)
        node = await self._node(root)
        if key == node["k"]:
            return await self._make(key, packed_value, node["l"], node["r"])
        if key < node["k"]:
            child = await self._put_packed(node["l"], key, packed_value)
            child_node = await self._node(child)
            if child_node["p"] < node["p"]:
                new_right = await self._make(node["k"], node["v"], child_node["r"], node["r"])
                return await self._make(child_node["k"], child_node["v"], child_node["l"], new_right)
            return await self._make(node["k"], node["v"], child, node["r"])
        child = await self._put_packed(node["r"], key, packed_value)
        child_node = await self._node(child)
        if child_node["p"] < node["p"]:
            new_left = await self._make(node["k"], node["v"], node["l"], child_node["l"])
            return await self._make(child_node["k"], child_node["v"], new_left, child_node["r"])
        return await self._make(node["k"], node["v"], node["l"], child)

    async def _ensure(self):
        while True:
            version, root_value = await self.store.read(self._ROOT)
            if version != 0:
                return version, root_value["node"]
            root = None
            for branch, head in self._initial_heads.items():
                root = await self._put(root, self._key("head", branch), head)
            if await self.store.cas(self._ROOT, 0, {"node": root}):
                return 1, root

    async def _transact(self, decide):
        """Run a deterministic decision at a root snapshot; retry failed root CAS."""
        while True:
            version, root_value = await self.store.read(self._ROOT)
            if version == 0:
                await self._ensure()
                continue
            root = root_value["node"]
            result, writes = await decide(root)
            if not writes:
                return result
            new_root = root
            for key, value in writes:
                new_root = await self._put(new_root, key, value)
            if await self.store.cas(self._ROOT, version, {"node": new_root}):
                return result

    async def submit(self, request_id, updates):
        copied = _json_copy(updates)
        candidate_digest = digest(copied)
        submit_key = self._key("submit", request_id)
        bundle_key = self._key("bundle", request_id)
        response = {"bundle": request_id, "revision": 1, "digest": candidate_digest}
        record = {"revision": 1, "updates": copied, "digest": candidate_digest,
                  "approvals": [], "ci": False, "published": False}

        async def decide(root):
            prior = await self._get(root, submit_key)
            if prior is not None:
                return (_json_copy(prior), [])
            return (_json_copy(response), [(bundle_key, record), (submit_key, response)])

        return await self._transact(decide)

    async def revise(self, bundle, expected_revision, updates):
        copied = _json_copy(updates)
        candidate_digest = digest(copied)
        key = self._key("bundle", bundle)

        async def decide(root):
            current = await self._get(root, key)
            if expected_revision != current["revision"]:
                return ("stale", [])
            if current["published"]:
                return ("published", [])
            changed = {"revision": current["revision"] + 1, "updates": copied,
                       "digest": candidate_digest, "approvals": [], "ci": False,
                       "published": False}
            return ("updated", [(key, changed)])

        return await self._transact(decide)

    async def approve(self, bundle, revision, reviewer):
        if reviewer not in ("alice", "bob", "carol"):
            return False
        key = self._key("bundle", bundle)

        async def decide(root):
            current = await self._get(root, key)
            if revision != current["revision"] or current["published"]:
                return (False, [])
            if reviewer in current["approvals"]:
                return (True, [])
            changed = _json_copy(current)
            changed["approvals"].append(reviewer)
            return (True, [(key, changed)])

        return await self._transact(decide)

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        key = self._key("bundle", bundle)

        async def decide(root):
            current = await self._get(root, key)
            if (revision != current["revision"] or current["digest"] != candidate_digest
                    or current["published"]):
                return (False, [])
            changed = _json_copy(current)
            changed["ci"] = bool(passed)
            return (True, [(key, changed)])

        return await self._transact(decide)

    async def publish(self, bundle, revision, request_id):
        outcome_key = self._key("publish", request_id)
        bundle_key = self._key("bundle", bundle)

        async def decide(root):
            prior = await self._get(root, outcome_key)
            if prior is not None:
                return (prior, [])
            current = await self._get(root, bundle_key)
            if revision != current["revision"]:
                outcome = "stale"
                writes = [(outcome_key, outcome)]
                return (outcome, writes)
            if current["published"]:
                outcome = "published"
                return (outcome, [(outcome_key, outcome)])
            if len(set(current["approvals"])) < 2 or not current["ci"]:
                outcome = "not_ready"
                return (outcome, [(outcome_key, outcome)])
            for branch, change in current["updates"].items():
                head = await self._get(root, self._key("head", branch))
                if head != change["expected"]:
                    outcome = "conflict"
                    return (outcome, [(outcome_key, outcome)])
            writes = []
            for branch, change in current["updates"].items():
                writes.append((self._key("head", branch), change["new"]))
            changed = _json_copy(current)
            changed["published"] = True
            writes.append((bundle_key, changed))
            writes.append((outcome_key, "published"))
            return ("published", writes)

        return await self._transact(decide)

    async def read_heads(self, branches):
        _, root = await self._ensure()
        result = {}
        for branch in branches:
            result[branch] = await self._get(root, self._key("head", branch))
        return result

    async def get_status(self, bundle):
        _, root = await self._ensure()
        current = await self._get(root, self._key("bundle", bundle))
        return {"revision": current["revision"], "updates": _json_copy(current["updates"]),
                "digest": current["digest"], "approvals": sorted(current["approvals"]),
                "ci": current["ci"], "published": current["published"]}
