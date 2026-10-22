"""Durable, linearizable certified change bundle service."""
import hashlib
import json

from contract import digest


def _json(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def _copy(value):
    # The durable API already copies, but this also separates inputs/outputs from
    # local replay state and from other fields in a response.
    return json.loads(_json(value))


class Service:
    _ROOT = "ccb:root"
    _REVIEWERS = {"alice", "bob", "carol"}

    def __init__(self, store, initial_heads):
        self.store = store
        self._initial_heads = _copy(initial_heads)
        self._base = hashlib.sha256(_json(["base", self._initial_heads]).encode()).hexdigest()

    async def _ensure_root(self):
        # The base node is immutable and named by its content. Competing lazy
        # initializers may both write it; only one root CAS can initialize state.
        base_key = "ccb:node:" + self._base
        value = {"base": True, "heads": self._initial_heads}
        version, old = await self.store.read(base_key)
        if version == 0:
            await self.store.cas(base_key, 0, value)
        version, root = await self.store.read(self._ROOT)
        if version == 0:
            await self.store.cas(self._ROOT, 0, {"head": self._base})

    async def _load(self):
        await self._ensure_root()
        root_version, root = await self.store.read(self._ROOT)
        head = root["head"]
        chain = []
        while head != self._base:
            _, node = await self.store.read("ccb:node:" + head)
            if "event" in node:
                event = node["event"]
            else:
                encoded = "".join([
                    (await self.store.read("ccb:chunk:" + chunk_id))[1]["data"]
                    for chunk_id in node["chunks"]
                ])
                event = json.loads(encoded)
            chain.append(event)
            head = node["parent"]
        state = {"heads": _copy(self._initial_heads), "bundles": {}, "publishes": {}}
        for event in reversed(chain):
            kind = event["kind"]
            if kind == "submit":
                state["bundles"][event["bundle"]] = {
                    "revision": 1,
                    "updates": event["updates"],
                    "digest": event["digest"],
                    "submitted_digest": event["digest"],
                    "approvals": [],
                    "ci": False,
                    "published": False,
                }
            elif kind == "revise":
                bundle = state["bundles"][event["bundle"]]
                bundle["revision"] += 1
                bundle["updates"] = event["updates"]
                bundle["digest"] = event["digest"]
                bundle["approvals"] = []
                bundle["ci"] = False
            elif kind == "approve":
                approvals = state["bundles"][event["bundle"]]["approvals"]
                if event["reviewer"] not in approvals:
                    approvals.append(event["reviewer"])
            elif kind == "ci":
                state["bundles"][event["bundle"]]["ci"] = event["passed"]
            elif kind == "publish":
                state["publishes"][event["request_id"]] = event["result"]
                if event["result"] == "published" and event["advance"]:
                    bundle = state["bundles"][event["bundle"]]
                    for branch, update in bundle["updates"].items():
                        state["heads"][branch] = update["new"]
                    bundle["published"] = True
        return root_version, root["head"], state

    async def _commit(self, version, parent, event):
        # Persist the immutable event before publishing its pointer. A crash before
        # the pointer CAS leaves only an unreachable node. A crash after it is
        # recovered by replaying from the durable root.
        encoded_event = _json(event)
        node_material = _json([parent, encoded_event])
        node_id = hashlib.sha256(node_material.encode()).hexdigest()
        key = "ccb:node:" + node_id
        if len(encoded_event.encode("utf-8")) <= 8000:
            value = {"parent": parent, "event": event}
        else:
            pieces = [encoded_event[i:i + 8000] for i in range(0, len(encoded_event), 8000)]
            chunk_ids = []
            for piece in pieces:
                chunk_id = hashlib.sha256(piece.encode()).hexdigest()
                chunk_key = "ccb:chunk:" + chunk_id
                chunk_version, _ = await self.store.read(chunk_key)
                if chunk_version == 0:
                    await self.store.cas(chunk_key, 0, {"data": piece})
                chunk_ids.append(chunk_id)
            value = {"parent": parent, "chunks": chunk_ids}
        node_version, _ = await self.store.read(key)
        if node_version == 0:
            await self.store.cas(key, 0, value)
        return await self.store.cas(self._ROOT, version, {"head": node_id})

    async def submit(self, request_id, updates):
        updates = _copy(updates)
        submitted_digest = digest(updates)
        while True:
            version, parent, state = await self._load()
            existing = state["bundles"].get(request_id)
            if existing is not None:
                return {"bundle": request_id, "revision": 1, "digest": self._submitted_digest(request_id, state)}
            event = {"kind": "submit", "bundle": request_id, "updates": updates, "digest": submitted_digest}
            if await self._commit(version, parent, event):
                return {"bundle": request_id, "revision": 1, "digest": submitted_digest}

    @staticmethod
    def _submitted_digest(bundle, state):
        # The first revision's response remains stable after later edits. The
        # original updates are retained in the first submit event, so replay adds
        # this private field below when needed.
        return state["bundles"][bundle].get("submitted_digest", state["bundles"][bundle]["digest"])

    async def revise(self, bundle, expected_revision, updates):
        updates = _copy(updates)
        new_digest = digest(updates)
        while True:
            version, parent, state = await self._load()
            current = state["bundles"][bundle]
            if expected_revision != current["revision"]:
                return "stale"
            if current["published"]:
                return "published"
            event = {"kind": "revise", "bundle": bundle, "updates": updates, "digest": new_digest}
            if await self._commit(version, parent, event):
                return "updated"

    async def approve(self, bundle, revision, reviewer):
        if reviewer not in self._REVIEWERS:
            return False
        while True:
            version, parent, state = await self._load()
            current = state["bundles"][bundle]
            if revision != current["revision"] or current["published"]:
                return False
            if reviewer in current["approvals"]:
                return True
            event = {"kind": "approve", "bundle": bundle, "reviewer": reviewer}
            if await self._commit(version, parent, event):
                return True

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        while True:
            version, parent, state = await self._load()
            current = state["bundles"][bundle]
            if revision != current["revision"] or candidate_digest != current["digest"] or current["published"]:
                return False
            passed = bool(passed)
            if current["ci"] == passed:
                return True
            event = {"kind": "ci", "bundle": bundle, "passed": passed}
            if await self._commit(version, parent, event):
                return True

    async def publish(self, bundle, revision, request_id):
        while True:
            version, parent, state = await self._load()
            if request_id in state["publishes"]:
                return state["publishes"][request_id]
            current = state["bundles"][bundle]
            if revision != current["revision"]:
                result, advance = "stale", False
            elif current["published"]:
                result, advance = "published", False
            elif len(set(current["approvals"]) & self._REVIEWERS) < 2 or not current["ci"]:
                result, advance = "not_ready", False
            elif any(state["heads"][branch] != update["expected"] for branch, update in current["updates"].items()):
                result, advance = "conflict", False
            else:
                result, advance = "published", True
            event = {
                "kind": "publish", "bundle": bundle,
                "request_id": request_id, "result": result, "advance": advance,
            }
            if await self._commit(version, parent, event):
                return result

    async def read_heads(self, branches):
        _, _, state = await self._load()
        return {branch: state["heads"][branch] for branch in branches}

    async def get_status(self, bundle):
        _, _, state = await self._load()
        current = state["bundles"][bundle]
        return {
            "revision": current["revision"],
            "updates": _copy(current["updates"]),
            "digest": current["digest"],
            "approvals": sorted(current["approvals"]),
            "ci": current["ci"],
            "published": current["published"],
        }
