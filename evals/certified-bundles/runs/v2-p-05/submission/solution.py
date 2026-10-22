"""Linearizable, durable certified bundle service over a CAS-only store."""
import hashlib
import json
from contract import digest

_ROOT = "ccb/root"
_PREFIX = "ccb/event/"


def _json(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def _event_key(event):
    return _PREFIX + hashlib.sha256(_json(event).encode("utf-8")).hexdigest()


class Service:
    def __init__(self, store, initial_heads):
        self.store = store
        # Copy caller-owned structures. Branches and initial heads are immutable by contract.
        self.initial_heads = dict(initial_heads)

    async def _snapshot(self):
        """Read the root and replay its immutable event chain into an isolated state."""
        version, root = await self.store.read(_ROOT)
        if root is None:
            init = {"kind": "init", "heads": self.initial_heads}
            key = _event_key(init)
            ok, _ = await self._put_event(key, init)
            # Another initializer may have won. CAS only establishes a root if still absent.
            await self.store.cas(_ROOT, 0, {"tail": key})
            version, root = await self.store.read(_ROOT)
        events = []
        key = root["tail"]
        while key is not None:
            _, event = await self.store.read(key)
            events.append(event)
            key = event.get("prev")
        state = {"heads": {}, "bundles": {}, "submits": {}, "publishes": {}}
        for event in reversed(events):
            self._apply(state, event)
        return version, root, state

    async def _put_event(self, key, event):
        if await self.store.cas(key, 0, event):
            return True, event
        _, existing = await self.store.read(key)
        # SHA-256 names content. The check also makes corruption fail loudly.
        if existing != event:
            raise RuntimeError("event key collision or invalid durable record")
        return False, existing

    @staticmethod
    def _apply(state, event):
        kind = event["kind"]
        if kind == "init":
            state["heads"] = dict(event["heads"])
        elif kind == "submit":
            bid, rid = event["bundle"], event["request_id"]
            if rid not in state["submits"]:
                response = {"bundle": bid, "revision": 1, "digest": event["digest"]}
                state["submits"][rid] = response
                state["bundles"][bid] = {
                    "revision": 1, "updates": event["updates"],
                    "digest": event["digest"], "approvals": [],
                    "ci": False, "published": False,
                }
        elif kind == "revise":
            b = state["bundles"][event["bundle"]]
            b.update(revision=event["revision"], updates=event["updates"],
                     digest=event["digest"], approvals=[], ci=False)
        elif kind == "approve":
            b = state["bundles"][event["bundle"]]
            if event["reviewer"] not in b["approvals"]:
                b["approvals"].append(event["reviewer"])
        elif kind == "ci":
            state["bundles"][event["bundle"]]["ci"] = event["passed"]
        elif kind == "publish":
            state["publishes"][event["request_id"]] = event["outcome"]
            if event["outcome"] == "published" and event["applied"]:
                state["heads"].update(event["heads"])
                state["bundles"][event["bundle"]]["published"] = True

    async def _commit(self, version, root, state, event):
        event = dict(event)
        event["prev"] = root["tail"]
        key = _event_key(event)
        await self._put_event(key, event)
        if await self.store.cas(_ROOT, version, {"tail": key}):
            self._apply(state, event)
            return True
        return False

    async def _transition(self, make_event, result_fn):
        while True:
            version, root, state = await self._snapshot()
            event, result = make_event(state)
            if event is None:
                return result
            if await self._commit(version, root, state, event):
                return result

    async def submit(self, request_id, updates):
        update_copy = json.loads(_json(updates))
        d = digest(update_copy)
        def plan(state):
            if request_id in state["submits"]:
                return None, dict(state["submits"][request_id])
            event = {"kind": "submit", "request_id": request_id,
                     "bundle": request_id, "updates": update_copy, "digest": d}
            return event, {"bundle": request_id, "revision": 1, "digest": d}
        return await self._transition(plan, None)

    async def revise(self, bundle, expected_revision, updates):
        update_copy = json.loads(_json(updates))
        d = digest(update_copy)
        def plan(state):
            b = state["bundles"][bundle]
            if b["revision"] != expected_revision:
                return None, "stale"
            if b["published"]:
                return None, "published"
            return ({"kind": "revise", "bundle": bundle,
                     "revision": expected_revision + 1, "updates": update_copy,
                     "digest": d}, "updated")
        return await self._transition(plan, None)

    async def approve(self, bundle, revision, reviewer):
        if reviewer not in ("alice", "bob", "carol"):
            return False
        def plan(state):
            b = state["bundles"][bundle]
            if b["revision"] != revision or b["published"]:
                return None, False
            if reviewer in b["approvals"]:
                return None, True
            return ({"kind": "approve", "bundle": bundle, "reviewer": reviewer}, True)
        return await self._transition(plan, None)

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        passed = bool(passed)
        def plan(state):
            b = state["bundles"][bundle]
            if b["revision"] != revision or b["digest"] != candidate_digest or b["published"]:
                return None, False
            if b["ci"] == passed:
                return None, True
            return ({"kind": "ci", "bundle": bundle, "passed": passed}, True)
        return await self._transition(plan, None)

    async def publish(self, bundle, revision, request_id):
        def plan(state):
            if request_id in state["publishes"]:
                return None, state["publishes"][request_id]
            b = state["bundles"][bundle]
            if revision != b["revision"]:
                outcome = "stale"
            elif b["published"]:
                outcome = "published"
            elif len(set(b["approvals"]) & {"alice", "bob", "carol"}) < 2 or not b["ci"]:
                outcome = "not_ready"
            elif any(state["heads"][branch] != change["expected"]
                     for branch, change in b["updates"].items()):
                outcome = "conflict"
            else:
                outcome = "published"
            applied = outcome == "published" and not b["published"]
            changed_heads = ({branch: change["new"]
                              for branch, change in b["updates"].items()}
                             if applied else {})
            event = {"kind": "publish", "request_id": request_id,
                     "bundle": bundle, "outcome": outcome,
                     "applied": applied, "heads": changed_heads}
            return event, outcome
        return await self._transition(plan, None)

    async def read_heads(self, branches):
        _, _, state = await self._snapshot()
        return {branch: state["heads"][branch] for branch in branches}

    async def get_status(self, bundle):
        _, _, state = await self._snapshot()
        b = state["bundles"][bundle]
        return {"revision": b["revision"], "updates": json.loads(_json(b["updates"])),
                "digest": b["digest"], "approvals": sorted(b["approvals"]),
                "ci": b["ci"], "published": b["published"]}
