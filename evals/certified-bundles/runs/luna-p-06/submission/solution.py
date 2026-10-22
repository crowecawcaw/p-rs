"""Durable, linearizable certified change bundles."""
import hashlib
import json

from contract import digest

_ROOT = "certified-bundles:root"
_NODE = "certified-bundles:node:"


def _json(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def _key_path(key):
    return hashlib.sha256(key.encode("utf-8")).hexdigest()


class Service:
    def __init__(self, store, initial_heads):
        self.store = store
        self.initial_heads = dict(initial_heads)

    async def _store_node(self, node):
        raw = _json(node)
        h = hashlib.sha256(raw.encode("utf-8")).hexdigest()
        key = _NODE + h
        if not await self.store.cas(key, 0, node):
            _, prior = await self.store.read(key)
            if prior != node:
                raise RuntimeError("content-addressed node collision")
        return h

    async def _load(self, h):
        _, val = await self.store.read(_NODE + h)
        if val is None:
            raise RuntimeError("missing durable trie node")
        return val

    async def _get(self, root, key):
        h = root
        path = _key_path(key)
        for depth in range(64):
            node = await self._load(h)
            h = node.get(path[depth])
            if h is None:
                return None
        leaf = await self._load(h)
        return leaf.get("items", {}).get(key)

    async def _set(self, root, key, value):
        path = _key_path(key)
        async def put(h, depth):
            if depth == 64:
                leaf = {"items": {}}
                if h is not None:
                    leaf = await self._load(h)
                items = dict(leaf.get("items", {}))
                items[key] = value
                return await self._store_node({"items": items})
            node = {} if h is None else await self._load(h)
            child = await put(node.get(path[depth]), depth + 1)
            changed = dict(node)
            changed[path[depth]] = child
            return await self._store_node(changed)
        return await put(root, 0)

    async def _ensure(self):
        version, pointer = await self.store.read(_ROOT)
        if pointer is not None:
            return version, pointer["root"]
        root = await self._store_node({})
        for branch, commit in sorted(self.initial_heads.items()):
            root = await self._set(root, "head:" + branch, commit)
        if await self.store.cas(_ROOT, version, {"root": root}):
            return version + 1, root
        version, pointer = await self.store.read(_ROOT)
        return version, pointer["root"]

    async def _transact(self, edits):
        """CAS a root containing all edits; retry against newer snapshots."""
        while True:
            version, root = await self._ensure()
            new_root = root
            for key, value in edits.items():
                new_root = await self._set(new_root, key, value)
            if await self.store.cas(_ROOT, version, {"root": new_root}):
                return

    async def _snapshot_value(self, root, key):
        return await self._get(root, key)

    @staticmethod
    def _bundle_key(bundle):
        return "bundle:" + bundle

    @staticmethod
    def _publish_key(request_id):
        return "publish:" + request_id

    async def submit(self, request_id, updates):
        copied = json.loads(_json(updates))
        d = digest(copied)
        key = self._bundle_key(request_id)
        while True:
            version, root = await self._ensure()
            old = await self._get(root, key)
            if old is not None:
                return {"bundle": request_id, "revision": 1, "digest": old["digest"]}
            record = {"revision": 1, "updates": copied, "digest": d,
                      "approvals": [], "ci": False, "published": False}
            new_root = await self._set(root, key, record)
            if await self.store.cas(_ROOT, version, {"root": new_root}):
                return {"bundle": request_id, "revision": 1, "digest": d}

    async def revise(self, bundle, expected_revision, updates):
        copied = json.loads(_json(updates))
        key = self._bundle_key(bundle)
        while True:
            version, root = await self._ensure()
            rec = await self._get(root, key)
            if expected_revision != rec["revision"]:
                return "stale"
            if rec["published"]:
                return "published"
            updated = dict(rec)
            updated.update(revision=rec["revision"] + 1, updates=copied,
                           digest=digest(copied), approvals=[], ci=False)
            nr = await self._set(root, key, updated)
            if await self.store.cas(_ROOT, version, {"root": nr}):
                return "updated"

    async def approve(self, bundle, revision, reviewer):
        if reviewer not in ("alice", "bob", "carol"):
            return False
        key = self._bundle_key(bundle)
        while True:
            version, root = await self._ensure()
            rec = await self._get(root, key)
            if revision != rec["revision"] or rec["published"]:
                return False
            if reviewer in rec["approvals"]:
                return True
            updated = dict(rec)
            updated["approvals"] = sorted(rec["approvals"] + [reviewer])
            nr = await self._set(root, key, updated)
            if await self.store.cas(_ROOT, version, {"root": nr}):
                return True

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        key = self._bundle_key(bundle)
        while True:
            version, root = await self._ensure()
            rec = await self._get(root, key)
            if (revision != rec["revision"] or candidate_digest != rec["digest"]
                    or rec["published"]):
                return False
            result = bool(passed)
            if rec["ci"] is result:
                return True
            updated = dict(rec)
            updated["ci"] = result
            nr = await self._set(root, key, updated)
            if await self.store.cas(_ROOT, version, {"root": nr}):
                return True

    async def publish(self, bundle, revision, request_id):
        outcome_key = self._publish_key(request_id)
        bundle_key = self._bundle_key(bundle)
        while True:
            version, root = await self._ensure()
            prior = await self._get(root, outcome_key)
            if prior is not None:
                return prior
            rec = await self._get(root, bundle_key)
            if revision != rec["revision"]:
                outcome = "stale"
            elif rec["published"]:
                outcome = "published"
            elif len(set(rec["approvals"])) < 2 or not rec["ci"]:
                outcome = "not_ready"
            else:
                conflict = False
                for branch, update in rec["updates"].items():
                    if await self._get(root, "head:" + branch) != update["expected"]:
                        conflict = True
                        break
                outcome = "conflict" if conflict else "published"
            edits = {outcome_key: outcome}
            if outcome == "published" and not rec["published"] and revision == rec["revision"]:
                # The branch validations above and every update share the root CAS.
                for branch, update in rec["updates"].items():
                    edits["head:" + branch] = update["new"]
                published = dict(rec)
                published["published"] = True
                edits[bundle_key] = published
            nr = root
            for key, value in edits.items():
                nr = await self._set(nr, key, value)
            if await self.store.cas(_ROOT, version, {"root": nr}):
                return outcome

    async def read_heads(self, branches):
        _, root = await self._ensure()
        result = {}
        for branch in branches:
            result[branch] = await self._get(root, "head:" + branch)
        return result

    async def get_status(self, bundle):
        _, root = await self._ensure()
        rec = await self._get(root, self._bundle_key(bundle))
        return {"revision": rec["revision"], "updates": json.loads(_json(rec["updates"])),
                "digest": rec["digest"], "approvals": list(rec["approvals"]),
                "ci": rec["ci"], "published": rec["published"]}
