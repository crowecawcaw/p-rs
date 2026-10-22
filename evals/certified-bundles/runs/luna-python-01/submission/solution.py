import hashlib
import json

from contract import digest

_ROOT = "certified-change-bundles/root"


def _enc(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def _hash(value):
    return hashlib.sha256(_enc(value).encode("utf-8")).hexdigest()


class Service:
    def __init__(self, store, initial_heads):
        self.store = store
        self._initial_heads = dict(initial_heads)

    async def _write_immutable(self, prefix, value):
        raw = _enc(value)
        if len(raw.encode("utf-8")) > 16384:
            raise ValueError("record too large")
        ident = hashlib.sha256(raw.encode("utf-8")).hexdigest()
        key = prefix + ident
        version, old = await self.store.read(key)
        if version == 0:
            await self.store.cas(key, 0, value)
        return ident

    async def _snapshot(self, state):
        # Pack independent entity records into bounded immutable pages.
        entries = sorted(state["items"].items())
        pages, current = [], []
        for entry in entries:
            candidate = current + [entry]
            if current and len(_enc(candidate).encode("utf-8")) > 12000:
                pages.append(current)
                current = [entry]
            else:
                current = candidate
        if current:
            pages.append(current)
        page_ids = []
        for page in pages:
            page_ids.append(await self._write_immutable("certified-change-bundles/page/", page))
        descriptor = {"heads": state["heads"], "pages": page_ids}
        ident = await self._write_immutable("certified-change-bundles/snapshot/", descriptor)
        return ident

    async def _load_snapshot(self, ident):
        _, desc = await self.store.read("certified-change-bundles/snapshot/" + ident)
        items = {}
        for page_id in desc["pages"]:
            _, page = await self.store.read("certified-change-bundles/page/" + page_id)
            items.update(page)
        return {"heads": desc["heads"], "items": items}

    async def _ensure(self):
        version, root = await self.store.read(_ROOT)
        if version == 0:
            state = {"heads": self._initial_heads, "items": {}}
            ident = await self._snapshot(state)
            await self.store.cas(_ROOT, 0, {"snapshot": ident})

    async def _read_state(self):
        while True:
            await self._ensure()
            _, root = await self.store.read(_ROOT)
            return await self._load_snapshot(root["snapshot"])

    async def _transact(self, mutate):
        while True:
            await self._ensure()
            version, root = await self.store.read(_ROOT)
            state = await self._load_snapshot(root["snapshot"])
            result, changed = mutate(state)
            if not changed:
                return result
            ident = await self._snapshot(state)
            if await self.store.cas(_ROOT, version, {"snapshot": ident}):
                return result

    async def submit(self, request_id, updates):
        original = json.loads(_enc(updates))
        dg = digest(original)
        def op(state):
            key = "b:" + request_id
            if key in state["items"]:
                b = state["items"][key]
                return ({"bundle": request_id, "revision": 1, "digest": b["submit_digest"]}, False)
            state["items"][key] = {
                "revision": 1, "updates": original, "digest": dg,
                "approvals": [], "ci": False, "published": False,
                "submit_updates": original, "submit_digest": dg,
            }
            return ({"bundle": request_id, "revision": 1, "digest": dg}, True)
        return await self._transact(op)

    async def revise(self, bundle, expected_revision, updates):
        updates = json.loads(_enc(updates))
        def op(state):
            b = state["items"]["b:" + bundle]
            if expected_revision != b["revision"]: return ("stale", False)
            if b["published"]: return ("published", False)
            b["revision"] += 1
            b["updates"] = updates
            b["digest"] = digest(updates)
            b["approvals"] = []
            b["ci"] = False
            return ("updated", True)
        return await self._transact(op)

    async def approve(self, bundle, revision, reviewer):
        allowed = reviewer in ("alice", "bob", "carol")
        def op(state):
            b = state["items"]["b:" + bundle]
            if not allowed or revision != b["revision"] or b["published"]:
                return (False, False)
            if reviewer not in b["approvals"]:
                b["approvals"].append(reviewer)
                b["approvals"].sort()
                return (True, True)
            return (True, False)
        return await self._transact(op)

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        def op(state):
            b = state["items"]["b:" + bundle]
            if revision != b["revision"] or candidate_digest != b["digest"] or b["published"]:
                return (False, False)
            val = bool(passed)
            if b["ci"] == val: return (True, False)
            b["ci"] = val
            return (True, True)
        return await self._transact(op)

    async def publish(self, bundle, revision, request_id):
        def op(state):
            memo_key = "p:" + request_id
            if memo_key in state["items"]:
                return (state["items"][memo_key]["outcome"], False)
            b = state["items"]["b:" + bundle]
            if revision != b["revision"]: outcome = "stale"
            elif b["published"]: outcome = "published"
            elif len(set(b["approvals"]) & {"alice", "bob", "carol"}) < 2 or not b["ci"]: outcome = "not_ready"
            elif any(state["heads"][branch] != change["expected"] for branch, change in b["updates"].items()): outcome = "conflict"
            else:
                for branch, change in b["updates"].items():
                    state["heads"][branch] = change["new"]
                b["published"] = True
                outcome = "published"
            state["items"][memo_key] = {"outcome": outcome, "bundle": bundle, "revision": revision}
            return (outcome, True)
        return await self._transact(op)

    async def read_heads(self, branches):
        state = await self._read_state()
        return {branch: state["heads"][branch] for branch in branches}

    async def get_status(self, bundle):
        state = await self._read_state()
        b = state["items"]["b:" + bundle]
        return {"revision": b["revision"], "updates": json.loads(_enc(b["updates"])),
                "digest": b["digest"], "approvals": sorted(b["approvals"]),
                "ci": b["ci"], "published": b["published"]}
