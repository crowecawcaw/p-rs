"""Linearizable change-bundle service backed only by Store.read/Store.cas."""
import copy
import hashlib
import json

from contract import digest


_ROOT = "change-bundles:root"


def _json(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def _event_key(event):
    encoded = _json(event).encode("utf-8")
    return "change-bundles:event:" + hashlib.sha256(encoded).hexdigest()


class Service:
    def __init__(self, store, initial_heads):
        self.store = store
        # Constructors are synchronous. Keep only a private copy of the common
        # initial value; durable initialization happens in the first operation.
        self._initial_heads = copy.deepcopy(initial_heads)

    async def _snapshot(self):
        """Return a root version and the state represented by its immutable log."""
        version, root = await self.store.read(_ROOT)
        if root is None:
            genesis = {"prev": None, "kind": "genesis", "heads": self._initial_heads}
            key = _event_key(genesis)
            await self.store.cas(key, 0, genesis)
            # A competing initializer writes the same genesis and root value.
            await self.store.cas(_ROOT, version, {"tail": key})
            version, root = await self.store.read(_ROOT)
            while root is None:
                # Only needed if the competing root CAS did not complete yet.
                genesis = {"prev": None, "kind": "genesis", "heads": self._initial_heads}
                key = _event_key(genesis)
                await self.store.cas(key, 0, genesis)
                await self.store.cas(_ROOT, version, {"tail": key})
                version, root = await self.store.read(_ROOT)

        # Follow immutable event links from the root, then replay oldest first.
        events = []
        key = root["tail"]
        while key is not None:
            _, event = await self.store.read(key)
            if event is None:  # A root is only published after its event is durable.
                raise RuntimeError("committed change-bundle event is missing")
            events.append(event)
            key = event["prev"]

        state = {"heads": {}, "bundles": {}, "submits": {}, "publishes": {}}
        for event in reversed(events):
            kind = event["kind"]
            if kind == "genesis":
                state["heads"] = copy.deepcopy(event["heads"])
            elif kind == "submit":
                bid = event["bundle"]
                state["bundles"][bid] = {
                    "revision": 1,
                    "updates": copy.deepcopy(event["updates"]),
                    "digest": event["digest"],
                    "approvals": [],
                    "ci": False,
                    "published": False,
                }
                state["submits"][bid] = {
                    "bundle": bid,
                    "revision": 1,
                    "digest": event["digest"],
                }
            elif kind == "revise":
                b = state["bundles"][event["bundle"]]
                b["revision"] = event["revision"]
                b["updates"] = copy.deepcopy(event["updates"])
                b["digest"] = event["digest"]
                b["approvals"] = []
                b["ci"] = False
            elif kind == "approve":
                b = state["bundles"][event["bundle"]]
                if event["reviewer"] not in b["approvals"]:
                    b["approvals"].append(event["reviewer"])
                    b["approvals"].sort()
            elif kind == "ci":
                state["bundles"][event["bundle"]]["ci"] = event["passed"]
            elif kind == "publish":
                state["publishes"][event["request_id"]] = event["result"]
                if event["applied"]:
                    b = state["bundles"][event["bundle"]]
                    for branch, change in b["updates"].items():
                        state["heads"][branch] = change["new"]
                    b["published"] = True
        return version, root, state

    async def _append(self, version, root, kind, **fields):
        event = {"prev": root["tail"], "kind": kind, **copy.deepcopy(fields)}
        key = _event_key(event)
        # A deterministic event key makes an orphan from a lost response
        # reusable. A failed CAS means the identical immutable event is already
        # present (SHA-256 collision aside).
        await self.store.cas(key, 0, event)
        return await self.store.cas(_ROOT, version, {"tail": key})

    async def submit(self, request_id, updates):
        while True:
            version, root, state = await self._snapshot()
            prior = state["submits"].get(request_id)
            if prior is not None:
                return copy.deepcopy(prior)
            candidate = copy.deepcopy(updates)
            d = digest(candidate)
            if await self._append(version, root, "submit", bundle=request_id,
                                  updates=candidate, digest=d):
                return {"bundle": request_id, "revision": 1, "digest": d}

    async def revise(self, bundle, expected_revision, updates):
        while True:
            version, root, state = await self._snapshot()
            b = state["bundles"][bundle]
            if expected_revision != b["revision"]:
                return "stale"
            if b["published"]:
                return "published"
            candidate = copy.deepcopy(updates)
            d = digest(candidate)
            new_revision = b["revision"] + 1
            if await self._append(version, root, "revise", bundle=bundle,
                                  revision=new_revision, updates=candidate, digest=d):
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
            if await self._append(version, root, "approve", bundle=bundle,
                                  revision=revision, reviewer=reviewer):
                return True

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        while True:
            version, root, state = await self._snapshot()
            b = state["bundles"][bundle]
            if (revision != b["revision"] or b["published"] or
                    candidate_digest != b["digest"]):
                return False
            result = bool(passed)
            if b["ci"] == result:
                return True
            if await self._append(version, root, "ci", bundle=bundle,
                                  revision=revision, passed=result):
                return True

    async def publish(self, bundle, revision, request_id):
        while True:
            version, root, state = await self._snapshot()
            prior = state["publishes"].get(request_id)
            if prior is not None:
                return prior
            b = state["bundles"][bundle]
            if revision != b["revision"]:
                result, applied = "stale", False
            elif b["published"]:
                result, applied = "published", False
            elif len(b["approvals"]) < 2 or not b["ci"]:
                result, applied = "not_ready", False
            elif any(state["heads"][branch] != change["expected"]
                     for branch, change in b["updates"].items()):
                result, applied = "conflict", False
            else:
                result, applied = "published", True
            if await self._append(version, root, "publish", bundle=bundle,
                                  revision=revision, request_id=request_id,
                                  result=result, applied=applied):
                return result

    async def read_heads(self, branches):
        _, _, state = await self._snapshot()
        return {branch: state["heads"][branch] for branch in branches}

    async def get_status(self, bundle):
        _, _, state = await self._snapshot()
        b = state["bundles"][bundle]
        return {
            "revision": b["revision"],
            "updates": copy.deepcopy(b["updates"]),
            "digest": b["digest"],
            "approvals": sorted(b["approvals"]),
            "ci": b["ci"],
            "published": b["published"],
        }
