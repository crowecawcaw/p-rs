import copy
import hashlib
import json

from contract import digest


def _json(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def _clone(value):
    return copy.deepcopy(value)


class Service:
    """A linearizable change-bundle service backed only by Store.read/cas.

    The store's head key points to an immutable, parent-linked journal. A successful
    CAS of that one key commits the complete logical transition.
    """
    _HEAD = "change-bundles/head"
    _ROOT_PREFIX = "change-bundles/root/"
    _ENTRY_PREFIX = "change-bundles/entry/"
    _REVIEWERS = {"alice", "bob", "carol"}

    def __init__(self, store, initial_heads):
        self.store = store
        self._initial_heads = _clone(initial_heads)

    async def _snapshot(self):
        root_key = self._ROOT_PREFIX + hashlib.sha256(_json(self._initial_heads).encode()).hexdigest()
        version, root = await self.store.read(root_key)
        if version == 0:
            await self.store.cas(root_key, 0, {"root": self._initial_heads})
        hversion, pointer = await self.store.read(self._HEAD)
        if hversion == 0:
            await self.store.cas(self._HEAD, 0, root_key)
            hversion, pointer = await self.store.read(self._HEAD)

        # Records are immutable, so following this pointer remains a consistent
        # snapshot even if another client advances the head during the walk.
        events = []
        key = pointer
        while True:
            _, record = await self.store.read(key)
            if "root" in record:
                state = {"heads": _clone(record["root"]), "bundles": {},
                         "submits": {}, "publish_requests": {}}
                break
            events.append(record["event"])
            key = record["parent"]
        for event in reversed(events):
            self._apply(state, event)
        return hversion, pointer, state

    @staticmethod
    def _apply(state, event):
        kind = event["t"]
        if kind == "submit":
            state["bundles"][event["b"]] = {
                "revision": 1, "updates": _clone(event["u"]), "digest": event["d"],
                "approvals": [], "ci": False, "published": False,
            }
            state["submits"][event["b"]] = _clone(event["response"])
        elif kind == "revise":
            bundle = state["bundles"][event["b"]]
            bundle["revision"] += 1
            bundle["updates"] = _clone(event["u"])
            bundle["digest"] = event["d"]
            bundle["approvals"] = []
            bundle["ci"] = False
        elif kind == "approve":
            approvals = state["bundles"][event["b"]]["approvals"]
            if event["reviewer"] not in approvals:
                approvals.append(event["reviewer"])
        elif kind == "ci":
            state["bundles"][event["b"]]["ci"] = event["passed"]
        elif kind == "publish":
            state["publish_requests"][event["request"]] = event["outcome"]
            if event["outcome"] == "published" and event.get("apply"):
                state["heads"].update(event["heads"])
                state["bundles"][event["b"]]["published"] = True

    async def _commit(self, event, expected_version, parent):
        encoded = _json(event)
        key = self._ENTRY_PREFIX + hashlib.sha256((parent + "\n" + encoded).encode()).hexdigest()
        _, existing = await self.store.read(key)
        if existing is None:
            await self.store.cas(key, 0, {"parent": parent, "event": event})
        # If a concurrent writer committed this same deterministic entry, the
        # record is identical. The head CAS decides which caller wins the step.
        return await self.store.cas(self._HEAD, expected_version, key)

    async def submit(self, request_id, updates):
        updates = _clone(updates)
        while True:
            version, pointer, state = await self._snapshot()
            prior = state["submits"].get(request_id)
            if prior is not None:
                return _clone(prior)
            response = {"bundle": request_id, "revision": 1, "digest": digest(updates)}
            event = {"t": "submit", "b": request_id, "u": updates,
                     "d": response["digest"], "response": response}
            if await self._commit(event, version, pointer):
                return _clone(response)

    async def revise(self, bundle, expected_revision, updates):
        updates = _clone(updates)
        while True:
            version, pointer, state = await self._snapshot()
            current = state["bundles"][bundle]
            if expected_revision != current["revision"]:
                return "stale"
            if current["published"]:
                return "published"
            event = {"t": "revise", "b": bundle, "u": updates, "d": digest(updates)}
            if await self._commit(event, version, pointer):
                return "updated"

    async def approve(self, bundle, revision, reviewer):
        if reviewer not in self._REVIEWERS:
            return False
        while True:
            version, pointer, state = await self._snapshot()
            current = state["bundles"][bundle]
            if revision != current["revision"] or current["published"]:
                return False
            event = {"t": "approve", "b": bundle, "reviewer": reviewer}
            if await self._commit(event, version, pointer):
                return True

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        while True:
            version, pointer, state = await self._snapshot()
            current = state["bundles"][bundle]
            if (revision != current["revision"] or current["published"] or
                    candidate_digest != current["digest"]):
                return False
            event = {"t": "ci", "b": bundle, "passed": bool(passed)}
            if await self._commit(event, version, pointer):
                return True

    async def publish(self, bundle, revision, request_id):
        while True:
            version, pointer, state = await self._snapshot()
            memo = state["publish_requests"].get(request_id)
            if memo is not None:
                return memo
            current = state["bundles"][bundle]
            apply_heads = False
            heads = {}
            if revision != current["revision"]:
                outcome = "stale"
            elif current["published"]:
                outcome = "published"
            elif len(set(current["approvals"]) & self._REVIEWERS) < 2 or not current["ci"]:
                outcome = "not_ready"
            elif any(state["heads"][branch] != item["expected"]
                     for branch, item in current["updates"].items()):
                outcome = "conflict"
            else:
                outcome = "published"
                apply_heads = True
                heads = {branch: item["new"] for branch, item in current["updates"].items()}
            event = {"t": "publish", "b": bundle, "request": request_id,
                     "outcome": outcome, "apply": apply_heads, "heads": heads}
            if await self._commit(event, version, pointer):
                return outcome

    async def read_heads(self, branches):
        _, _, state = await self._snapshot()
        return {branch: state["heads"][branch] for branch in branches}

    async def get_status(self, bundle):
        _, _, state = await self._snapshot()
        current = state["bundles"][bundle]
        return {
            "revision": current["revision"],
            "updates": _clone(current["updates"]),
            "digest": current["digest"],
            "approvals": sorted(current["approvals"]),
            "ci": current["ci"],
            "published": current["published"],
        }
