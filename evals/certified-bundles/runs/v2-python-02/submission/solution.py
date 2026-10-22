"""Linearizable, durable certified change bundle service."""
import copy
import hashlib
import json

from contract import digest


def _encode(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def _ref(value):
    return hashlib.sha256(_encode(value).encode("utf-8")).hexdigest()


class Service:
    _ROOT = "ccb:root:v1"
    _SHARDS = 128
    _REVIEWERS = {"alice", "bob", "carol"}

    def __init__(self, store, initial_heads):
        self.store = store
        self._initial = copy.deepcopy(initial_heads)

    @staticmethod
    def _logical(kind, name):
        return kind + ":" + name

    def _shard(self, logical_key):
        n = int.from_bytes(hashlib.sha256(logical_key.encode("utf-8")).digest()[:4], "big")
        return n % self._SHARDS

    async def _write_immutable(self, value):
        ref = _ref(value)
        key = "ccb:data:" + ref
        version, old = await self.store.read(key)
        if version == 0:
            if await self.store.cas(key, 0, value):
                return ref
            _, old = await self.store.read(key)
        if old != value:
            raise RuntimeError("content address collision")
        return ref

    async def _data(self, ref):
        if ref is None:
            return None
        _, value = await self.store.read("ccb:data:" + ref)
        return value

    async def _root(self):
        while True:
            version, manifest = await self.store.read(self._ROOT)
            if manifest is not None:
                return version, manifest
            heads_ref = await self._write_immutable(self._initial)
            manifest = {"heads": heads_ref, "buckets": [None] * self._SHARDS}
            if await self.store.cas(self._ROOT, version, manifest):
                return version + 1, manifest

    async def _get(self, manifest, logical_key):
        if logical_key.startswith("head:"):
            heads = await self._data(manifest["heads"])
            return heads.get(logical_key[5:])
        index = self._shard(logical_key)
        bucket = await self._data(manifest["buckets"][index])
        if bucket is None:
            return None
        return bucket.get(logical_key)

    async def _commit(self, version, manifest, changes):
        # Coalesce edits so a publish writes each immutable shard only once.
        changed = {}
        heads = None
        for logical_key, value in changes:
            if logical_key.startswith("head:"):
                if heads is None:
                    heads = await self._data(manifest["heads"])
                heads[logical_key[5:]] = copy.deepcopy(value)
            else:
                index = self._shard(logical_key)
                if index not in changed:
                    bucket = await self._data(manifest["buckets"][index])
                    changed[index] = {} if bucket is None else bucket
                changed[index][logical_key] = copy.deepcopy(value)
        proposed = {"heads": manifest["heads"], "buckets": list(manifest["buckets"])}
        if heads is not None:
            proposed["heads"] = await self._write_immutable(heads)
        for index, bucket in changed.items():
            proposed["buckets"][index] = await self._write_immutable(bucket)
        return await self.store.cas(self._ROOT, version, proposed)

    async def submit(self, request_id, updates):
        updates = copy.deepcopy(updates)
        response = {"bundle": request_id, "revision": 1, "digest": digest(updates)}
        record = {"revision": 1, "updates": updates, "digest": response["digest"],
                  "approvals": [], "ci": False, "published": False}
        bundle_key = self._logical("bundle", request_id)
        memo_key = self._logical("submit", request_id)
        while True:
            version, manifest = await self._root()
            memo = await self._get(manifest, memo_key)
            if memo is not None:
                return copy.deepcopy(memo)
            if await self._commit(version, manifest, [(bundle_key, record), (memo_key, response)]):
                return copy.deepcopy(response)

    async def revise(self, bundle, expected_revision, updates):
        updates = copy.deepcopy(updates)
        key = self._logical("bundle", bundle)
        while True:
            version, manifest = await self._root()
            record = await self._get(manifest, key)
            if expected_revision != record["revision"]:
                return "stale"
            if record["published"]:
                return "published"
            revised = {"revision": record["revision"] + 1, "updates": updates,
                       "digest": digest(updates), "approvals": [], "ci": False,
                       "published": False}
            if await self._commit(version, manifest, [(key, revised)]):
                return "updated"

    async def approve(self, bundle, revision, reviewer):
        if reviewer not in self._REVIEWERS:
            return False
        key = self._logical("bundle", bundle)
        while True:
            version, manifest = await self._root()
            record = await self._get(manifest, key)
            if revision != record["revision"] or record["published"]:
                return False
            approvals = sorted(set(record["approvals"]) | {reviewer})
            if approvals == record["approvals"]:
                return True
            updated = dict(record)
            updated["approvals"] = approvals
            if await self._commit(version, manifest, [(key, updated)]):
                return True

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        key = self._logical("bundle", bundle)
        while True:
            version, manifest = await self._root()
            record = await self._get(manifest, key)
            if (revision != record["revision"] or record["published"]
                    or candidate_digest != record["digest"]):
                return False
            updated = dict(record)
            updated["ci"] = bool(passed)
            if updated == record:
                return True
            if await self._commit(version, manifest, [(key, updated)]):
                return True

    async def publish(self, bundle, revision, request_id):
        outcome_key = self._logical("publish", request_id)
        bundle_key = self._logical("bundle", bundle)
        while True:
            version, manifest = await self._root()
            prior = await self._get(manifest, outcome_key)
            if prior is not None:
                return prior
            record = await self._get(manifest, bundle_key)
            if revision != record["revision"]:
                outcome = "stale"
            elif record["published"]:
                outcome = "published"
            elif len(set(record["approvals"]) & self._REVIEWERS) < 2 or not record["ci"]:
                outcome = "not_ready"
            else:
                heads = await self._data(manifest["heads"])
                conflict = any(heads[branch] != update["expected"]
                               for branch, update in record["updates"].items())
                if conflict:
                    outcome = "conflict"
                else:
                    outcome = "published"
                    changes = [(self._logical("head", branch), update["new"])
                               for branch, update in record["updates"].items()]
                    new_record = dict(record)
                    new_record["published"] = True
                    changes.extend([(bundle_key, new_record), (outcome_key, outcome)])
                    if await self._commit(version, manifest, changes):
                        return outcome
                    continue
            if await self._commit(version, manifest, [(outcome_key, outcome)]):
                return outcome

    async def read_heads(self, branches):
        _, manifest = await self._root()
        heads = await self._data(manifest["heads"])
        return {branch: copy.deepcopy(heads[branch]) for branch in branches}

    async def get_status(self, bundle):
        _, manifest = await self._root()
        record = await self._get(manifest, self._logical("bundle", bundle))
        return {"revision": record["revision"], "updates": copy.deepcopy(record["updates"]),
                "digest": record["digest"], "approvals": sorted(record["approvals"]),
                "ci": bool(record["ci"]), "published": bool(record["published"])}
