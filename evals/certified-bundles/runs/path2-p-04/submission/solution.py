"""Durable, linearizable change-bundle coordination over a single-key CAS store."""
import hashlib
import json

from contract import digest

_ROOT = "certified-change-bundles/root"
_AUTHORIZED = {"alice", "bob", "carol"}


def _json_copy(value):
    return json.loads(json.dumps(value, ensure_ascii=True, separators=(",", ":")))


def _encoded(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


class Service:
    def __init__(self, store, initial_heads):
        self.store = store
        self._initial_heads = _json_copy(initial_heads)

    async def _put(self, value):
        """Store an immutable JSON object at its content hash."""
        body = _encoded(value)
        key = "certified-change-bundles/object/" + hashlib.sha256(body.encode("utf-8")).hexdigest()
        if not await self.store.cas(key, 0, value):
            _, existing = await self.store.read(key)
            # A SHA-256 collision is outside the practical model, but do not
            # silently use another value if one occurs.
            if _encoded(existing) != body:
                raise RuntimeError("content address collision")
        return key.rsplit("/", 1)[1]

    async def _obj(self, object_id):
        if object_id is None:
            return None
        _, value = await self.store.read("certified-change-bundles/object/" + object_id)
        return value

    async def _node(self, key, value, priority, left, right):
        return await self._put({"k": key, "v": value, "p": priority, "l": left, "r": right})

    @staticmethod
    def _priority(key):
        return hashlib.sha256(key.encode("utf-8")).hexdigest()

    async def _get(self, root, key):
        current = root
        while current is not None:
            node = await self._obj(current)
            if key == node["k"]:
                return node["v"]
            current = node["l"] if key < node["k"] else node["r"]
        return None

    async def _items(self, root):
        """Yield a persistent treap's key/value pairs in key order."""
        stack = []
        current = root
        while current is not None or stack:
            while current is not None:
                node = await self._obj(current)
                stack.append(node)
                current = node["l"]
            node = stack.pop()
            yield node["k"], node["v"]
            current = node["r"]

    async def _updates_root(self, updates):
        root = None
        for branch, value in sorted(updates.items()):
            root = await self._set(root, branch, value)
        return root

    async def _set(self, root, key, value):
        """Persistent treap insertion/update; all reachable nodes are immutable."""
        if root is None:
            return await self._node(key, value, self._priority(key), None, None)
        node = await self._obj(root)
        if key == node["k"]:
            return await self._node(key, value, node["p"], node["l"], node["r"])
        if key < node["k"]:
            left = await self._set(node["l"], key, value)
            left_node = await self._obj(left)
            if left_node["p"] < node["p"]:
                new_right = await self._node(node["k"], node["v"], node["p"], left_node["r"], node["r"])
                return await self._node(left_node["k"], left_node["v"], left_node["p"], left_node["l"], new_right)
            return await self._node(node["k"], node["v"], node["p"], left, node["r"])
        right = await self._set(node["r"], key, value)
        right_node = await self._obj(right)
        if right_node["p"] < node["p"]:
            new_left = await self._node(node["k"], node["v"], node["p"], node["l"], right_node["l"])
            return await self._node(right_node["k"], right_node["v"], right_node["p"], new_left, right_node["r"])
        return await self._node(node["k"], node["v"], node["p"], node["l"], right)

    async def _root(self):
        while True:
            version, root = await self.store.read(_ROOT)
            if root is not None:
                return version, root
            heads = None
            for name, commit in sorted(self._initial_heads.items()):
                heads = await self._set(heads, name, commit)
            initial = {"heads": heads, "bundles": None, "publishes": None}
            if await self.store.cas(_ROOT, version, initial):
                return version + 1, initial

    async def _read_bundle(self, bundle_root, bundle):
        value = await self._get(bundle_root, bundle)
        if value is None:
            # Contract excludes unknown bundles.
            raise KeyError(bundle)
        return value

    async def submit(self, request_id, updates):
        updates = _json_copy(updates)
        new_digest = digest(updates)
        while True:
            version, root = await self._root()
            prior = await self._get(root["bundles"], request_id)
            if prior is not None:
                return {"bundle": request_id, "revision": 1, "digest": prior["submitted_digest"]}
            record = {
                "revision": 1,
                "updates_root": await self._updates_root(updates),
                "digest": new_digest,
                "submitted_digest": new_digest,
                "approvals": [],
                "ci": False,
                "published": False,
            }
            bundles = await self._set(root["bundles"], request_id, record)
            proposed = {"heads": root["heads"], "bundles": bundles, "publishes": root["publishes"]}
            if await self.store.cas(_ROOT, version, proposed):
                return {"bundle": request_id, "revision": 1, "digest": new_digest}

    async def revise(self, bundle, expected_revision, updates):
        updates = _json_copy(updates)
        new_digest = digest(updates)
        while True:
            version, root = await self._root()
            old = await self._read_bundle(root["bundles"], bundle)
            if expected_revision != old["revision"]:
                return "stale"
            if old["published"]:
                return "published"
            revised = dict(old)
            revised.update({"revision": old["revision"] + 1,
                            "updates_root": await self._updates_root(updates),
                            "digest": new_digest, "approvals": [], "ci": False})
            bundles = await self._set(root["bundles"], bundle, revised)
            proposed = {"heads": root["heads"], "bundles": bundles, "publishes": root["publishes"]}
            if await self.store.cas(_ROOT, version, proposed):
                return "updated"

    async def approve(self, bundle, revision, reviewer):
        if reviewer not in _AUTHORIZED:
            return False
        while True:
            version, root = await self._root()
            old = await self._read_bundle(root["bundles"], bundle)
            if revision != old["revision"] or old["published"]:
                return False
            if reviewer in old["approvals"]:
                return True
            revised = dict(old)
            revised["approvals"] = sorted(old["approvals"] + [reviewer])
            bundles = await self._set(root["bundles"], bundle, revised)
            proposed = {"heads": root["heads"], "bundles": bundles, "publishes": root["publishes"]}
            if await self.store.cas(_ROOT, version, proposed):
                return True

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        while True:
            version, root = await self._root()
            old = await self._read_bundle(root["bundles"], bundle)
            if revision != old["revision"] or candidate_digest != old["digest"] or old["published"]:
                return False
            revised = dict(old)
            revised["ci"] = bool(passed)
            bundles = await self._set(root["bundles"], bundle, revised)
            proposed = {"heads": root["heads"], "bundles": bundles, "publishes": root["publishes"]}
            if await self.store.cas(_ROOT, version, proposed):
                return True

    async def publish(self, bundle, revision, request_id):
        while True:
            version, root = await self._root()
            outcome = await self._get(root["publishes"], request_id)
            if outcome is not None:
                return outcome
            current = await self._read_bundle(root["bundles"], bundle)
            if revision != current["revision"]:
                result = "stale"
            elif current["published"]:
                result = "published"
            elif len(set(current["approvals"]) & _AUTHORIZED) < 2 or not current["ci"]:
                result = "not_ready"
            else:
                conflicts = False
                async for branch, update in self._items(current["updates_root"]):
                    head = await self._get(root["heads"], branch)
                    if head != update["expected"]:
                        conflicts = True
                        break
                if conflicts:
                    result = "conflict"
                else:
                    heads = root["heads"]
                    async for branch, update in self._items(current["updates_root"]):
                        heads = await self._set(heads, branch, update["new"])
                    revised = dict(current)
                    revised["published"] = True
                    bundles = await self._set(root["bundles"], bundle, revised)
                    root = {"heads": heads, "bundles": bundles, "publishes": root["publishes"]}
                    result = "published"
            publishes = await self._set(root["publishes"], request_id, result)
            proposed = {"heads": root["heads"], "bundles": root["bundles"], "publishes": publishes}
            if await self.store.cas(_ROOT, version, proposed):
                return result

    async def read_heads(self, branches):
        _, root = await self._root()
        result = {}
        for branch in branches:
            result[branch] = await self._get(root["heads"], branch)
        return result

    async def get_status(self, bundle):
        _, root = await self._root()
        value = await self._read_bundle(root["bundles"], bundle)
        updates = {}
        async for branch, update in self._items(value["updates_root"]):
            updates[branch] = _json_copy(update)
        return {
            "revision": value["revision"],
            "updates": updates,
            "digest": value["digest"],
            "approvals": sorted(value["approvals"]),
            "ci": value["ci"],
            "published": value["published"],
        }
