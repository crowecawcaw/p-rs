"""Durable, linearizable change bundle service over a CAS key/value store.

The commit key is a single versioned pointer into an immutable event chain.  All
business state changes become visible by swapping that pointer with one CAS.
"""
from __future__ import annotations

import copy
from contract import digest


class Service:
    _ROOT = "ccb:root"
    _ALLOC = "ccb:allocator"
    _EVENT = "ccb:event:"
    _BLOB = "ccb:blob:"

    def __init__(self, store, initial_heads):
        self.store = store
        self._initial_heads = copy.deepcopy(initial_heads)

    async def _root(self):
        """Return the initialized commit pointer (store version, event id)."""
        while True:
            version, value = await self.store.read(self._ROOT)
            if value is not None:
                return version, value["id"]
            if await self.store.cas(self._ROOT, version, {"id": 0}):
                return version + 1, 0

    async def _store_blob(self, value):
        """Persist arbitrary JSON payloads as linked, size-bounded chunks."""
        import json
        encoded = json.dumps(value, separators=(",", ":"), ensure_ascii=True)
        pieces = [encoded[i:i + 10000] for i in range(0, len(encoded), 10000)]
        next_id = 0
        for piece in reversed(pieces):
            chunk_id = await self._allocate()
            ok = await self.store.cas(self._BLOB + str(chunk_id), 0,
                                      {"text": piece, "next": next_id})
            if not ok:  # unique allocator IDs make this a store violation
                raise RuntimeError("duplicate durable blob ID")
            next_id = chunk_id
        return next_id

    async def _load_blob(self, chunk_id):
        import json
        pieces = []
        while chunk_id:
            _, chunk = await self.store.read(self._BLOB + str(chunk_id))
            pieces.append(chunk["text"])
            chunk_id = chunk["next"]
        return json.loads("".join(pieces))

    async def _state_at(self, event_id):
        """Replay one immutable committed prefix into private transient state."""
        state = {
            "heads": copy.deepcopy(self._initial_heads),
            "bundles": {},
            "publish_requests": {},
        }
        chain = []
        current = event_id
        while current:
            _, event = await self.store.read(self._EVENT + str(current))
            # Only committed event IDs are reachable from the root.  Records are
            # never deleted or modified after their unique allocation.
            chain.append(event)
            current = event["prev"]
        for event in reversed(chain):
            kind = event["kind"]
            if kind == "submit":
                state["bundles"][event["bundle"]] = {
                    "revision": 1,
                    "updates": await self._load_blob(event["updates_ref"]),
                    "digest": event["digest"],
                    "submit_digest": event["digest"],
                    "approvals": [],
                    "ci": False,
                    "published": False,
                }
            elif kind == "revise":
                bundle = state["bundles"][event["bundle"]]
                bundle["revision"] = event["revision"]
                bundle["updates"] = await self._load_blob(event["updates_ref"])
                bundle["digest"] = event["digest"]
                bundle["approvals"] = []
                bundle["ci"] = False
            elif kind == "approve":
                bundle = state["bundles"][event["bundle"]]
                if event["reviewer"] not in bundle["approvals"]:
                    bundle["approvals"].append(event["reviewer"])
            elif kind == "ci":
                state["bundles"][event["bundle"]]["ci"] = event["passed"]
            elif kind == "publish":
                state["publish_requests"][event["request_id"]] = event["outcome"]
                if event["outcome"] == "published" and event["apply"]:
                    published_bundle = state["bundles"][event["bundle"]]
                    for branch, values in published_bundle["updates"].items():
                        state["heads"][branch] = values["new"]
                    published_bundle["published"] = True
            else:  # pragma: no cover - records are written only by this class
                raise RuntimeError("unknown durable event")
        return state

    async def _allocate(self):
        """Allocate a globally unique immutable event key; gaps are harmless."""
        while True:
            version, value = await self.store.read(self._ALLOC)
            next_id = (value if value is not None else 0) + 1
            if await self.store.cas(self._ALLOC, version, next_id):
                return next_id

    async def _commit(self, root_version, previous_id, event):
        """Write an immutable candidate, then expose it with the root CAS."""
        event_id = await self._allocate()
        event["prev"] = previous_id
        key = self._EVENT + str(event_id)
        if not await self.store.cas(key, 0, event):
            # Allocation makes this unreachable unless a store violates its
            # contract; treating it as a lost race keeps the caller's loop safe.
            return False
        return await self.store.cas(self._ROOT, root_version, {"id": event_id})

    async def submit(self, request_id, updates):
        candidate = copy.deepcopy(updates)
        candidate_digest = digest(candidate)
        while True:
            rv, latest = await self._root()
            state = await self._state_at(latest)
            existing = state["bundles"].get(request_id)
            if existing is not None:
                return {"bundle": request_id, "revision": 1,
                        "digest": existing["submit_digest"]}
            updates_ref = await self._store_blob(candidate)
            event = {"kind": "submit", "bundle": request_id,
                     "updates_ref": updates_ref, "digest": candidate_digest}
            if await self._commit(rv, latest, event):
                return {"bundle": request_id, "revision": 1,
                        "digest": candidate_digest}

    async def revise(self, bundle, expected_revision, updates):
        candidate = copy.deepcopy(updates)
        candidate_digest = digest(candidate)
        while True:
            rv, latest = await self._root()
            state = await self._state_at(latest)
            current = state["bundles"][bundle]
            if expected_revision != current["revision"]:
                return "stale"
            if current["published"]:
                return "published"
            new_revision = current["revision"] + 1
            updates_ref = await self._store_blob(candidate)
            event = {"kind": "revise", "bundle": bundle,
                     "revision": new_revision, "updates_ref": updates_ref,
                     "digest": candidate_digest}
            if await self._commit(rv, latest, event):
                return "updated"

    async def approve(self, bundle, revision, reviewer):
        if reviewer not in ("alice", "bob", "carol"):
            return False
        while True:
            rv, latest = await self._root()
            state = await self._state_at(latest)
            current = state["bundles"][bundle]
            if revision != current["revision"] or current["published"]:
                return False
            if reviewer in current["approvals"]:
                return True
            event = {"kind": "approve", "bundle": bundle,
                     "reviewer": reviewer}
            if await self._commit(rv, latest, event):
                return True

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        while True:
            rv, latest = await self._root()
            state = await self._state_at(latest)
            current = state["bundles"][bundle]
            if (revision != current["revision"] or
                    candidate_digest != current["digest"] or
                    current["published"]):
                return False
            accepted = bool(passed)
            if current["ci"] == accepted:
                return True
            event = {"kind": "ci", "bundle": bundle, "passed": accepted}
            if await self._commit(rv, latest, event):
                return True

    async def publish(self, bundle, revision, request_id):
        while True:
            rv, latest = await self._root()
            state = await self._state_at(latest)
            if request_id in state["publish_requests"]:
                return state["publish_requests"][request_id]
            current = state["bundles"][bundle]
            if revision != current["revision"]:
                outcome = "stale"
            elif current["published"]:
                outcome = "published"
            elif len(set(current["approvals"]) & {"alice", "bob", "carol"}) < 2 or not current["ci"]:
                outcome = "not_ready"
            elif any(state["heads"][branch] != values["expected"]
                     for branch, values in current["updates"].items()):
                outcome = "conflict"
            else:
                outcome = "published"
            apply = outcome == "published" and not current["published"]
            event = {"kind": "publish", "bundle": bundle,
                     "request_id": request_id, "outcome": outcome,
                     "apply": apply}
            if await self._commit(rv, latest, event):
                return outcome

    async def read_heads(self, branches):
        _, latest = await self._root()
        state = await self._state_at(latest)
        return {branch: copy.deepcopy(state["heads"][branch])
                for branch in branches}

    async def get_status(self, bundle):
        _, latest = await self._root()
        state = await self._state_at(latest)
        current = state["bundles"][bundle]
        return {
            "revision": current["revision"],
            "updates": copy.deepcopy(current["updates"]),
            "digest": current["digest"],
            "approvals": sorted(current["approvals"]),
            "ci": current["ci"],
            "published": current["published"],
        }
