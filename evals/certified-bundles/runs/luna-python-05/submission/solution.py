"""Linearizable durable bundle service backed only by the supplied KV store."""
import copy
import hashlib
import json

from contract import digest

_ROOT = "certified-bundles/root"
_EVENT_PREFIX = "certified-bundles/event/"


def _json(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def _clone(value):
    return copy.deepcopy(value)


class Service:
    def __init__(self, store, initial_heads):
        self.store = store
        # The constructor does no store I/O. All instances receive identical heads.
        self._initial_heads = _clone(initial_heads)

    async def _init_if_needed(self):
        while True:
            version, root = await self.store.read(_ROOT)
            if version:
                return
            event = {"op": "init", "heads": self._initial_heads}
            ident = hashlib.sha256(_json(event).encode("utf-8")).hexdigest()
            await self.store.cas(_EVENT_PREFIX + ident, 0, event)
            if await self.store.cas(_ROOT, 0, ident):
                return

    async def _snapshot(self):
        await self._init_if_needed()
        _, root = await self.store.read(_ROOT)
        events = []
        while root is not None:
            _, event = await self.store.read(_EVENT_PREFIX + root)
            events.append(event)
            root = event.get("prev")
        state = {"heads": {}, "bundles": {}, "outcomes": {}}
        for event in reversed(events):
            op = event["op"]
            if op == "init":
                state["heads"] = _clone(event["heads"])
            elif op == "submit":
                bid = event["bundle"]
                updates = _clone(event["updates"])
                state["bundles"][bid] = {
                    "revision": 1,
                    "updates": updates,
                    "digest": event["digest"],
                    "approvals": set(),
                    "ci": False,
                    "published": False,
                }
            elif op == "revise":
                b = state["bundles"][event["bundle"]]
                b["revision"] = event["revision"]
                b["updates"] = _clone(event["updates"])
                b["digest"] = event["digest"]
                b["approvals"] = set()
                b["ci"] = False
            elif op == "approve":
                state["bundles"][event["bundle"]]["approvals"].add(event["reviewer"])
            elif op == "ci":
                state["bundles"][event["bundle"]]["ci"] = event["passed"]
            elif op == "publish":
                state["outcomes"][event["request_id"]] = event["result"]
                if event["result"] == "published":
                    b = state["bundles"][event["bundle"]]
                    if event["apply"]:
                        for branch, head in event["heads"].items():
                            state["heads"][branch] = head
                        b["published"] = True
        return state

    async def _append(self, previous_root, event):
        record = _clone(event)
        record["prev"] = previous_root
        ident = hashlib.sha256(_json(record).encode("utf-8")).hexdigest()
        # Event keys are content addressed and immutable. A false CAS means the
        # identical event already exists (normally after an interrupted retry).
        await self.store.cas(_EVENT_PREFIX + ident, 0, record)
        return ident

    async def _commit(self, root_version, old_root, event):
        new_root = await self._append(old_root, event)
        return await self.store.cas(_ROOT, root_version, new_root)

    async def _read_root(self):
        await self._init_if_needed()
        return await self.store.read(_ROOT)

    async def submit(self, request_id, updates):
        updates = _clone(updates)
        d = digest(updates)
        while True:
            version, root = await self._read_root()
            state = await self._snapshot_from(root)
            if request_id in state["bundles"]:
                b = state["bundles"][request_id]
                return {"bundle": request_id, "revision": 1, "digest": b["initial_digest"]}
            event = {"op": "submit", "bundle": request_id, "updates": updates, "digest": d}
            if await self._commit(version, root, event):
                return {"bundle": request_id, "revision": 1, "digest": d}

    async def _snapshot_from(self, root):
        events = []
        while root is not None:
            _, event = await self.store.read(_EVENT_PREFIX + root)
            events.append(event)
            root = event.get("prev")
        state = {"heads": {}, "bundles": {}, "outcomes": {}}
        for event in reversed(events):
            op = event["op"]
            if op == "init":
                state["heads"] = _clone(event["heads"])
            elif op == "submit":
                bid = event["bundle"]
                state["bundles"][bid] = {
                    "revision": 1, "updates": _clone(event["updates"]),
                    "digest": event["digest"], "initial_digest": event["digest"],
                    "approvals": set(), "ci": False, "published": False,
                }
            elif op == "revise":
                b = state["bundles"][event["bundle"]]
                b["revision"] = event["revision"]
                b["updates"] = _clone(event["updates"])
                b["digest"] = event["digest"]
                b["approvals"] = set()
                b["ci"] = False
            elif op == "approve":
                state["bundles"][event["bundle"]]["approvals"].add(event["reviewer"])
            elif op == "ci":
                state["bundles"][event["bundle"]]["ci"] = event["passed"]
            elif op == "publish":
                state["outcomes"][event["request_id"]] = event["result"]
                if event["result"] == "published":
                    b = state["bundles"][event["bundle"]]
                    if event["apply"]:
                        state["heads"].update(event["heads"])
                        b["published"] = True
        return state

    async def revise(self, bundle, expected_revision, updates):
        updates = _clone(updates)
        d = digest(updates)
        while True:
            version, root = await self._read_root()
            state = await self._snapshot_from(root)
            b = state["bundles"][bundle]
            if expected_revision != b["revision"]:
                return "stale"
            if b["published"]:
                return "published"
            event = {"op": "revise", "bundle": bundle,
                     "revision": b["revision"] + 1, "updates": updates, "digest": d}
            if await self._commit(version, root, event):
                return "updated"

    async def approve(self, bundle, revision, reviewer):
        if reviewer not in {"alice", "bob", "carol"}:
            return False
        while True:
            version, root = await self._read_root()
            state = await self._snapshot_from(root)
            b = state["bundles"][bundle]
            if revision != b["revision"] or b["published"]:
                return False
            if reviewer in b["approvals"]:
                return True
            event = {"op": "approve", "bundle": bundle, "reviewer": reviewer}
            if await self._commit(version, root, event):
                return True

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        passed = bool(passed)
        while True:
            version, root = await self._read_root()
            state = await self._snapshot_from(root)
            b = state["bundles"][bundle]
            if revision != b["revision"] or candidate_digest != b["digest"] or b["published"]:
                return False
            event = {"op": "ci", "bundle": bundle, "passed": passed}
            if await self._commit(version, root, event):
                return True

    async def publish(self, bundle, revision, request_id):
        while True:
            version, root = await self._read_root()
            state = await self._snapshot_from(root)
            if request_id in state["outcomes"]:
                return state["outcomes"][request_id]
            b = state["bundles"][bundle]
            if revision != b["revision"]:
                result = "stale"
                apply = False
                changed_heads = {}
            elif b["published"]:
                result = "published"
                apply = False
                changed_heads = {}
            elif len(b["approvals"]) < 2 or not b["ci"]:
                result = "not_ready"
                apply = False
                changed_heads = {}
            elif any(state["heads"][branch] != update["expected"]
                     for branch, update in b["updates"].items()):
                result = "conflict"
                apply = False
                changed_heads = {}
            else:
                result = "published"
                apply = True
                changed_heads = {branch: update["new"]
                                 for branch, update in b["updates"].items()}
            event = {"op": "publish", "bundle": bundle, "request_id": request_id,
                     "result": result, "apply": apply, "heads": changed_heads}
            if await self._commit(version, root, event):
                return result

    async def read_heads(self, branches):
        state = await self._snapshot()
        return {branch: state["heads"][branch] for branch in branches}

    async def get_status(self, bundle):
        state = await self._snapshot()
        b = state["bundles"][bundle]
        return {"revision": b["revision"], "updates": _clone(b["updates"]),
                "digest": b["digest"], "approvals": sorted(b["approvals"]),
                "ci": b["ci"], "published": b["published"]}
