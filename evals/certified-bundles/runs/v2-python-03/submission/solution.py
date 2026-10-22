"""Linearizable certified change bundles over a CAS-only durable store."""
import copy
import hashlib
import json

from contract import digest


_ROOT = "certified-bundles/root"
_EVENT_PREFIX = "certified-bundles/event/"
_REVIEWERS = {"alice", "bob", "carol"}


def _json(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def _event_id(event):
    return hashlib.sha256(_json(event).encode("utf-8")).hexdigest()


class Service:
    def __init__(self, store, initial_heads):
        self.store = store
        self._initial_heads = copy.deepcopy(initial_heads)

    async def _ensure_root(self):
        """Lazily install a shared base event and root pointer."""
        while True:
            version, pointer = await self.store.read(_ROOT)
            if version:
                return version, pointer

            base = {"op": "base", "heads": self._initial_heads}
            base_id = _event_id(base)
            key = _EVENT_PREFIX + base_id
            if not await self.store.cas(key, 0, base):
                # The content-addressed record may have been installed by a
                # concurrent initializer. Confirm it before pointing at it.
                _, found = await self.store.read(key)
                if found != base:
                    continue
            if await self.store.cas(_ROOT, 0, base_id):
                return await self.store.read(_ROOT)
            # Another instance initialized the root; use its pointer.

    async def _load(self):
        """Read and replay one immutable event-log snapshot."""
        root_version, pointer = await self._ensure_root()
        chain = []
        current = pointer
        while current:
            _, event = await self.store.read(_EVENT_PREFIX + current)
            # A committed root can only point to a durable event. The store's
            # contract guarantees durability, so a missing event is corruption.
            if event is None:
                raise RuntimeError("durable event referenced by root is missing")
            chain.append(event)
            current = event.get("prev")

        state = None
        for event in reversed(chain):
            op = event["op"]
            if op == "base":
                state = {"heads": copy.deepcopy(event["heads"]),
                         "bundles": {}, "outcomes": {}}
            elif op == "submit":
                state["bundles"][event["bundle"]] = {
                    "revision": 1,
                    "updates": copy.deepcopy(event["updates"]),
                    "digest": event["digest"],
                    "approvals": [], "ci": False, "published": False,
                }
            elif op == "revise":
                bundle = state["bundles"][event["bundle"]]
                bundle["revision"] = event["revision"]
                bundle["updates"] = copy.deepcopy(event["updates"])
                bundle["digest"] = event["digest"]
                bundle["approvals"] = []
                bundle["ci"] = False
            elif op == "approve":
                approvals = state["bundles"][event["bundle"]]["approvals"]
                if event["reviewer"] not in approvals:
                    approvals.append(event["reviewer"])
            elif op == "ci":
                state["bundles"][event["bundle"]]["ci"] = event["passed"]
            elif op == "publish":
                state["outcomes"][event["request_id"]] = {
                    "bundle": event["bundle"],
                    "revision": event["revision"],
                    "result": event["result"],
                }
                if event["result"] == "published":
                    state["bundles"][event["bundle"]]["published"] = True
                    state["heads"].update(event["heads"])
            else:
                raise RuntimeError("unknown durable event")
        if state is None:
            raise RuntimeError("event log has no base")
        return root_version, pointer, state

    async def _store_event(self, event):
        event_id = _event_id(event)
        key = _EVENT_PREFIX + event_id
        if not await self.store.cas(key, 0, event):
            _, found = await self.store.read(key)
            if found != event:
                raise RuntimeError("content-addressed event collision")
        return event_id

    async def _append(self, root_version, pointer, event):
        event["prev"] = pointer
        event_id = await self._store_event(event)
        return await self.store.cas(_ROOT, root_version, event_id)

    async def submit(self, request_id, updates):
        updates = copy.deepcopy(updates)
        while True:
            root_version, pointer, state = await self._load()
            existing = state["bundles"].get(request_id)
            if existing is not None:
                return {"bundle": request_id, "revision": 1,
                        "digest": existing["digest"]}
            value_digest = digest(updates)
            event = {"op": "submit", "bundle": request_id,
                     "updates": updates, "digest": value_digest}
            if await self._append(root_version, pointer, event):
                return {"bundle": request_id, "revision": 1,
                        "digest": value_digest}

    async def revise(self, bundle, expected_revision, updates):
        updates = copy.deepcopy(updates)
        while True:
            root_version, pointer, state = await self._load()
            current = state["bundles"][bundle]
            if current["revision"] != expected_revision:
                return "stale"
            if current["published"]:
                return "published"
            next_revision = current["revision"] + 1
            event = {"op": "revise", "bundle": bundle,
                     "revision": next_revision, "updates": updates,
                     "digest": digest(updates)}
            if await self._append(root_version, pointer, event):
                return "updated"

    async def approve(self, bundle, revision, reviewer):
        if reviewer not in _REVIEWERS:
            return False
        while True:
            root_version, pointer, state = await self._load()
            current = state["bundles"][bundle]
            if current["revision"] != revision or current["published"]:
                return False
            if reviewer in current["approvals"]:
                return True
            event = {"op": "approve", "bundle": bundle,
                     "reviewer": reviewer}
            if await self._append(root_version, pointer, event):
                return True

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        while True:
            root_version, pointer, state = await self._load()
            current = state["bundles"][bundle]
            if (current["revision"] != revision or current["published"]
                    or candidate_digest != current["digest"]):
                return False
            event = {"op": "ci", "bundle": bundle,
                     "passed": bool(passed)}
            if await self._append(root_version, pointer, event):
                return True

    async def publish(self, bundle, revision, request_id):
        while True:
            root_version, pointer, state = await self._load()
            prior = state["outcomes"].get(request_id)
            if prior is not None:
                return prior["result"]

            current = state["bundles"][bundle]
            if current["revision"] != revision:
                result = "stale"
            elif current["published"]:
                result = "published"
            elif len(set(current["approvals"]) & _REVIEWERS) < 2 or not current["ci"]:
                result = "not_ready"
            elif any(state["heads"][branch] != update["expected"]
                     for branch, update in current["updates"].items()):
                result = "conflict"
            else:
                result = "published"

            changed_heads = {}
            if result == "published" and not current["published"]:
                changed_heads = {branch: update["new"]
                                 for branch, update in current["updates"].items()}
            event = {"op": "publish", "bundle": bundle,
                     "revision": revision, "request_id": request_id,
                     "result": result, "heads": changed_heads}
            if await self._append(root_version, pointer, event):
                return result

    async def read_heads(self, branches):
        _, _, state = await self._load()
        return {branch: state["heads"][branch] for branch in branches}

    async def get_status(self, bundle):
        _, _, state = await self._load()
        current = state["bundles"][bundle]
        return {
            "revision": current["revision"],
            "updates": copy.deepcopy(current["updates"]),
            "digest": current["digest"],
            "approvals": sorted(current["approvals"]),
            "ci": current["ci"],
            "published": current["published"],
        }
