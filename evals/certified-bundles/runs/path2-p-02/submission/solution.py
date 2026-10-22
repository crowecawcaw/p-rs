"""Linearizable durable certified change bundle service.

A single durable root serializes commits. Immutable numbered journal entries hold
history; the root can carry one pending entry so another caller can finish an
interrupted commit.
"""
import copy

from contract import digest


class Service:
    ROOT = "certified-bundles/root"
    LOG = "certified-bundles/log/"
    REVIEWERS = {"alice", "bob", "carol"}

    def __init__(self, store, initial_heads):
        self.store = store
        self._initial_heads = copy.deepcopy(initial_heads)

    async def _root(self):
        version, root = await self.store.read(self.ROOT)
        if root is None:
            initial = {"seq": 0, "pending": None, "heads": self._initial_heads}
            if await self.store.cas(self.ROOT, version, initial):
                return await self.store.read(self.ROOT)
            return await self.store.read(self.ROOT)
        return version, root

    async def _finish_pending(self, version, root):
        """Help complete an already-reserved event; safe for any concurrent helper."""
        event = root["pending"]
        seq = root["seq"] + 1
        key = self.LOG + str(seq)
        lv, old = await self.store.read(key)
        if old is None:
            # Every helper writes the same durable reservation.
            await self.store.cas(key, lv, event)
        else:
            assert old == event
        # The root CAS is the commit point. A competing helper may have won it.
        current_version, current = await self.store.read(self.ROOT)
        if current.get("pending") == event and current.get("seq") == root["seq"]:
            await self.store.cas(self.ROOT, current_version,
                                 {"seq": seq, "pending": None, "heads": current["heads"]})

    async def _snapshot(self):
        """Return a consistent committed prefix, completing any interrupted writer."""
        while True:
            version, root = await self._root()
            if root["pending"] is not None:
                await self._finish_pending(version, root)
                continue
            state = {"heads": copy.deepcopy(root["heads"]), "bundles": {},
                     "submits": {}, "publish_requests": {}}
            for n in range(1, root["seq"] + 1):
                _, event = await self.store.read(self.LOG + str(n))
                self._apply(state, event)
            # The root read defines the snapshot even if a writer commits while replaying.
            return state, root

    @staticmethod
    def _apply(state, event):
        op = event["op"]
        if op == "submit":
            bundle = event["bundle"]
            state["bundles"][bundle] = {
                "revision": 1, "updates": copy.deepcopy(event["updates"]),
                "digest": event["digest"], "approvals": set(), "ci": False,
                "published": False,
            }
            state["submits"][bundle] = {
                "bundle": bundle, "revision": 1, "digest": event["digest"]}
        elif op == "revise":
            b = state["bundles"][event["bundle"]]
            b["revision"] += 1
            b["updates"] = copy.deepcopy(event["updates"])
            b["digest"] = event["digest"]
            b["approvals"] = set()
            b["ci"] = False
        elif op == "approve":
            state["bundles"][event["bundle"]]["approvals"].add(event["reviewer"])
        elif op == "ci":
            state["bundles"][event["bundle"]]["ci"] = event["passed"]
        elif op == "publish":
            state["publish_requests"][event["request_id"]] = event["outcome"]
            if event["apply"]:
                b = state["bundles"][event["bundle"]]
                for branch, pair in b["updates"].items():
                    state["heads"][branch] = pair["new"]
                b["published"] = True

    async def _append(self, event, expected_seq):
        """Serialize an operation by reserving it in root, then help it commit."""
        while True:
            version, root = await self._root()
            if root["pending"] is not None:
                await self._finish_pending(version, root)
                continue
            if root["seq"] != expected_seq:
                return False
            pending = {"seq": root["seq"], "pending": event, "heads": root["heads"]}
            if not await self.store.cas(self.ROOT, version, pending):
                continue
            # The reservation is durable. Complete it here, or a later caller will.
            v, current = await self.store.read(self.ROOT)
            if current["pending"] is not None:
                await self._finish_pending(v, current)
            return True

    async def submit(self, request_id, updates):
        while True:
            state, root = await self._snapshot()
            if request_id in state["submits"]:
                return copy.deepcopy(state["submits"][request_id])
            d = digest(updates)
            event = {"op": "submit", "bundle": request_id,
                     "updates": copy.deepcopy(updates), "digest": d}
            if not await self._append(event, root["seq"]):
                continue
            return {"bundle": request_id, "revision": 1, "digest": d}

    async def revise(self, bundle, expected_revision, updates):
        while True:
            state, root = await self._snapshot()
            b = state["bundles"][bundle]
            if expected_revision != b["revision"]:
                return "stale"
            if b["published"]:
                return "published"
            event = {"op": "revise", "bundle": bundle,
                     "updates": copy.deepcopy(updates), "digest": digest(updates)}
            if not await self._append(event, root["seq"]):
                continue
            return "updated"

    async def approve(self, bundle, revision, reviewer):
        if reviewer not in self.REVIEWERS:
            return False
        while True:
            state, root = await self._snapshot()
            b = state["bundles"][bundle]
            if revision != b["revision"] or b["published"]:
                return False
            event = {"op": "approve", "bundle": bundle, "reviewer": reviewer}
            if await self._append(event, root["seq"]):
                return True

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        while True:
            state, root = await self._snapshot()
            b = state["bundles"][bundle]
            if revision != b["revision"] or candidate_digest != b["digest"] or b["published"]:
                return False
            event = {"op": "ci", "bundle": bundle, "passed": bool(passed)}
            if await self._append(event, root["seq"]):
                return True

    async def publish(self, bundle, revision, request_id):
        while True:
            state, root = await self._snapshot()
            if request_id in state["publish_requests"]:
                return state["publish_requests"][request_id]
            b = state["bundles"][bundle]
            if revision != b["revision"]:
                outcome = "stale"
            elif b["published"]:
                outcome = "published"
            elif len(b["approvals"]) < 2 or not b["ci"]:
                outcome = "not_ready"
            elif any(state["heads"][branch] != pair["expected"]
                     for branch, pair in b["updates"].items()):
                outcome = "conflict"
            else:
                outcome = "published"
            event = {"op": "publish", "bundle": bundle, "request_id": request_id,
                     "outcome": outcome, "apply": outcome == "published" and not b["published"]}
            if await self._append(event, root["seq"]):
                return outcome

    async def read_heads(self, branches):
        state, _ = await self._snapshot()
        return {branch: state["heads"][branch] for branch in branches}

    async def get_status(self, bundle):
        state, _ = await self._snapshot()
        b = state["bundles"][bundle]
        return {"revision": b["revision"], "updates": copy.deepcopy(b["updates"]),
                "digest": b["digest"], "approvals": sorted(b["approvals"]),
                "ci": b["ci"], "published": b["published"]}
