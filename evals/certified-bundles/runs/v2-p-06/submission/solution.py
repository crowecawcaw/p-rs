"""A linearizable, durable certified change bundle service."""
import copy
import json

from contract import digest

_ROOT = "certified-bundles:root"
_NODE_PREFIX = "certified-bundles:node:"


def _clone(value):
    return copy.deepcopy(value)


class Service:
    def __init__(self, store, initial_heads):
        self.store = store
        self._initial_heads = _clone(initial_heads)
        # This is only a key-space probe hint. Correctness comes from CAS; after a
        # restart or collision the probe simply advances to an unused immutable key.
        self._node_probe = 0

    async def _new_node(self, node):
        """Persist an immutable node at an unused key, returning its key."""
        while True:
            key = _NODE_PREFIX + str(self._node_probe)
            self._node_probe += 1
            if await self.store.cas(key, 0, node):
                return key

    async def _initialize(self):
        version, _ = await self.store.read(_ROOT)
        if version != 0:
            return
        genesis = {"parent": None, "event": {"kind": "init", "heads": self._initial_heads}}
        tip = await self._new_node(genesis)
        await self.store.cas(_ROOT, 0, {"tip": tip})

    async def _snapshot(self):
        await self._initialize()
        root_version, root = await self.store.read(_ROOT)
        # The root cannot be absent after initialization unless the store violates
        # its contract. Keep the version to condition the next root CAS.
        events = []
        tip = root["tip"]
        while tip is not None:
            _, node = await self.store.read(tip)
            events.append(node["event"])
            tip = node["parent"]
        state = {"heads": {}, "bundles": {}, "submits": {}, "publishes": {}}
        for event in reversed(events):
            kind = event["kind"]
            if kind == "init":
                state["heads"] = _clone(event["heads"])
            elif kind == "submit":
                bid = event["bundle"]
                if bid not in state["bundles"]:
                    b = {
                        "revision": 1,
                        "updates": _clone(event["updates"]),
                        "digest": event["digest"],
                        "approvals": [],
                        "ci": False,
                        "published": False,
                    }
                    state["bundles"][bid] = b
                    state["submits"][bid] = {
                        "bundle": bid, "revision": 1, "digest": event["digest"]
                    }
            elif kind == "revise":
                b = state["bundles"][event["bundle"]]
                b["revision"] = event["revision"]
                b["updates"] = _clone(event["updates"])
                b["digest"] = event["digest"]
                b["approvals"] = []
                b["ci"] = False
            elif kind == "approve":
                b = state["bundles"][event["bundle"]]
                if event["reviewer"] not in b["approvals"]:
                    b["approvals"].append(event["reviewer"])
            elif kind == "ci":
                state["bundles"][event["bundle"]]["ci"] = event["passed"]
            elif kind == "publish":
                state["publishes"][event["request_id"]] = event["outcome"]
                if event["outcome"] == "published":
                    b = state["bundles"][event["bundle"]]
                    if not b["published"]:
                        for branch, change in b["updates"].items():
                            state["heads"][branch] = change["new"]
                        b["published"] = True
        return root_version, root, state

    async def _append(self, root_version, root, event):
        node = {"parent": root["tip"], "event": event}
        tip = await self._new_node(node)
        return await self.store.cas(_ROOT, root_version, {"tip": tip})

    async def submit(self, request_id, updates):
        while True:
            version, root, state = await self._snapshot()
            if request_id in state["submits"]:
                return _clone(state["submits"][request_id])
            candidate_digest = digest(updates)
            event = {
                "kind": "submit", "bundle": request_id,
                "updates": _clone(updates), "digest": candidate_digest,
            }
            if await self._append(version, root, event):
                return {"bundle": request_id, "revision": 1, "digest": candidate_digest}

    async def revise(self, bundle, expected_revision, updates):
        while True:
            version, root, state = await self._snapshot()
            b = state["bundles"][bundle]
            if expected_revision != b["revision"]:
                return "stale"
            if b["published"]:
                return "published"
            new_revision = b["revision"] + 1
            event = {
                "kind": "revise", "bundle": bundle, "revision": new_revision,
                "updates": _clone(updates), "digest": digest(updates),
            }
            if await self._append(version, root, event):
                return "updated"

    async def approve(self, bundle, revision, reviewer):
        if reviewer not in ("alice", "bob", "carol"):
            return False
        while True:
            version, root, state = await self._snapshot()
            b = state["bundles"][bundle]
            if revision != b["revision"] or b["published"]:
                return False
            if reviewer in b["approvals"]:
                return True
            event = {"kind": "approve", "bundle": bundle, "reviewer": reviewer}
            if await self._append(version, root, event):
                return True

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        while True:
            version, root, state = await self._snapshot()
            b = state["bundles"][bundle]
            if (revision != b["revision"] or candidate_digest != b["digest"]
                    or b["published"]):
                return False
            event = {"kind": "ci", "bundle": bundle, "passed": bool(passed)}
            if await self._append(version, root, event):
                return True

    async def publish(self, bundle, revision, request_id):
        while True:
            version, root, state = await self._snapshot()
            if request_id in state["publishes"]:
                return state["publishes"][request_id]
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
            event = {
                "kind": "publish", "bundle": bundle,
                "request_id": request_id, "outcome": outcome,
            }
            if await self._append(version, root, event):
                return outcome

    async def read_heads(self, branches):
        _, _, state = await self._snapshot()
        return {branch: state["heads"][branch] for branch in branches}

    async def get_status(self, bundle):
        _, _, state = await self._snapshot()
        b = state["bundles"][bundle]
        return {
            "revision": b["revision"],
            "updates": _clone(b["updates"]),
            "digest": b["digest"],
            "approvals": sorted(b["approvals"]),
            "ci": b["ci"],
            "published": b["published"],
        }
