"""Linearizable certified change bundles over a single durable CAS pointer."""
import copy
import hashlib
import json

from contract import digest

_ROOT = "certified-change-bundles/root"
_REVIEWERS = {"alice", "bob", "carol"}


def _json(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def _clone(value):
    # The public contract admits JSON values only.
    return json.loads(_json(value))


class Service:
    def __init__(self, store, initial_heads):
        self.store = store
        self._initial_heads = _clone(initial_heads)

    async def _snapshot(self):
        version, root = await self.store.read(_ROOT)
        if version == 0:
            initial = {"tail": None, "heads": self._initial_heads}
            if await self.store.cas(_ROOT, 0, initial):
                version, root = await self.store.read(_ROOT)
            else:
                version, root = await self.store.read(_ROOT)
        tail = root["tail"]
        reverse = []
        while tail is not None:
            _, node = await self.store.read(tail)
            if node is None:
                # A tail is only made visible after its immutable node is durable.
                continue
            reverse.append(node["event"])
            tail = node["parent"]
        state = {
            "heads": _clone(root["heads"]),
            "bundles": {},
            "submits": {},
            "publishes": {},
        }
        for event in reversed(reverse):
            self._apply(state, event)
        return version, root, state

    @staticmethod
    def _apply(state, e):
        op = e["op"]
        if op == "submit":
            b = e["bundle"]
            state["submits"].setdefault(b, {"updates": e["updates"], "response": e["response"]})
            if b not in state["bundles"]:
                state["bundles"][b] = {
                    "revision": 1,
                    "updates": e["updates"],
                    "digest": e["response"]["digest"],
                    "approvals": [],
                    "ci": False,
                    "published": False,
                }
        elif op == "revise":
            b = state["bundles"][e["bundle"]]
            if b["revision"] == e["revision"] and not b["published"]:
                b["revision"] += 1
                b["updates"] = e["updates"]
                b["digest"] = e["digest"]
                b["approvals"] = []
                b["ci"] = False
        elif op == "approve":
            b = state["bundles"][e["bundle"]]
            if b["revision"] == e["revision"] and not b["published"] and e["reviewer"] not in b["approvals"]:
                b["approvals"].append(e["reviewer"])
                b["approvals"].sort()
        elif op == "ci":
            b = state["bundles"][e["bundle"]]
            if b["revision"] == e["revision"] and not b["published"]:
                b["ci"] = e["passed"]
        elif op == "publish":
            # The outcome and any head changes become visible in the same root CAS.
            state["publishes"].setdefault(e["request_id"], e["outcome"])
            if e["outcome"] == "published" and e["applied"]:
                b = state["bundles"][e["bundle"]]
                if not b["published"]:
                    for branch, update in b["updates"].items():
                        state["heads"][branch] = update["new"]
                    b["published"] = True

    async def _append(self, version, root, event):
        # Content-addressed immutable log cells make abandoned pre-CAS writes harmless.
        parent = root["tail"]
        payload = {"parent": parent, "event": event}
        encoded = _json(payload)
        base = hashlib.sha256(encoded.encode("utf-8")).hexdigest()
        suffix = 0
        while True:
            key = "certified-change-bundles/node/" + base + ("/" + str(suffix) if suffix else "")
            node_version, node = await self.store.read(key)
            if node_version == 0:
                if await self.store.cas(key, 0, payload):
                    break
                continue
            if node == payload:
                break
            suffix += 1
        new_root = {"tail": key, "heads": root["heads"]}
        return await self.store.cas(_ROOT, version, new_root)

    async def submit(self, request_id, updates):
        updates = _clone(updates)
        while True:
            version, root, state = await self._snapshot()
            if request_id in state["submits"]:
                return _clone(state["submits"][request_id]["response"])
            d = digest(updates)
            response = {"bundle": request_id, "revision": 1, "digest": d}
            event = {"op": "submit", "bundle": request_id, "updates": updates, "response": response}
            if await self._append(version, root, event):
                return _clone(response)

    async def revise(self, bundle, expected_revision, updates):
        updates = _clone(updates)
        while True:
            version, root, state = await self._snapshot()
            b = state["bundles"][bundle]
            if b["revision"] != expected_revision:
                return "stale"
            if b["published"]:
                return "published"
            event = {"op": "revise", "bundle": bundle, "revision": expected_revision,
                     "updates": updates, "digest": digest(updates)}
            if await self._append(version, root, event):
                return "updated"

    async def approve(self, bundle, revision, reviewer):
        if reviewer not in _REVIEWERS:
            return False
        while True:
            version, root, state = await self._snapshot()
            b = state["bundles"][bundle]
            if b["revision"] != revision or b["published"]:
                return False
            event = {"op": "approve", "bundle": bundle, "revision": revision, "reviewer": reviewer}
            if await self._append(version, root, event):
                return True

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        while True:
            version, root, state = await self._snapshot()
            b = state["bundles"][bundle]
            if b["revision"] != revision or b["published"] or candidate_digest != b["digest"]:
                return False
            event = {"op": "ci", "bundle": bundle, "revision": revision, "passed": bool(passed)}
            if await self._append(version, root, event):
                return True

    async def publish(self, bundle, revision, request_id):
        while True:
            version, root, state = await self._snapshot()
            if request_id in state["publishes"]:
                return state["publishes"][request_id]
            b = state["bundles"][bundle]
            applied = False
            if b["revision"] != revision:
                outcome = "stale"
            elif b["published"]:
                outcome = "published"
            elif len(set(b["approvals"]) & _REVIEWERS) < 2 or not b["ci"]:
                outcome = "not_ready"
            elif any(state["heads"][branch] != update["expected"] for branch, update in b["updates"].items()):
                outcome = "conflict"
            else:
                outcome = "published"
                applied = True
            event = {"op": "publish", "bundle": bundle, "revision": revision,
                     "request_id": request_id, "outcome": outcome, "applied": applied}
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
