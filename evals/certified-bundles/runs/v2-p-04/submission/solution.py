"""Linearizable change bundle service over a single CAS commit pointer."""
import copy

from contract import digest


_ROOT = "change-bundles/root"
_LOG = "change-bundles/log/"


class Service:
    def __init__(self, store, initial_heads):
        self.store = store
        # This is immutable caller-supplied construction data, not shared state.
        self._initial_heads = copy.deepcopy(initial_heads)

    async def _load(self, seq):
        state = {
            "heads": copy.deepcopy(self._initial_heads),
            "bundles": {},
            "submits": {},
            "publishes": {},
        }
        for n in range(1, seq + 1):
            _, entry = await self.store.read(_LOG + str(n))
            if entry is None:
                # A visible root always has a complete prefix. This can only happen
                # if storage has violated its contract.
                raise RuntimeError("commit pointer references a missing log entry")
            kind = entry["kind"]
            if kind == "submit":
                state["bundles"][entry["id"]] = entry["value"]
                state["submits"][entry["id"]] = entry["response"]
            elif kind == "revise":
                b = state["bundles"][entry["id"]]
                b.update(entry["value"])
            elif kind == "approve":
                state["bundles"][entry["id"]]["approvals"].append(entry["reviewer"])
            elif kind == "ci":
                state["bundles"][entry["id"]]["ci"] = entry["passed"]
            elif kind == "publish":
                state["publishes"][entry["request_id"]] = entry["outcome"]
                if entry["outcome"] == "published":
                    state["bundles"][entry["id"]]["published"] = True
                    for branch, head in entry["heads"].items():
                        state["heads"][branch] = head
            else:
                raise RuntimeError("unknown change-bundle log entry")
        return state

    async def _snapshot(self):
        version, root = await self.store.read(_ROOT)
        if root is None:
            # Initial heads are identical across instances. A competing initializer
            # may win this CAS; either way the following read finds the root.
            await self.store.cas(_ROOT, version, {"seq": 0})
            version, root = await self.store.read(_ROOT)
        return version, root["seq"], await self._load(root["seq"])

    async def _help_next(self, root_version, seq):
        """Complete a durable reservation left by a caller that stopped."""
        _, entry = await self.store.read(_LOG + str(seq + 1))
        if entry is not None:
            await self.store.cas(_ROOT, root_version, {"seq": seq + 1})
            return True
        return False

    async def _commit(self, make_entry):
        """Retry a state transition until its immutable log slot is committed."""
        while True:
            root_version, seq, state = await self._snapshot()
            entry, result, changed = make_entry(state)
            if not changed:
                return result
            slot = _LOG + str(seq + 1)
            won = await self.store.cas(slot, 0, entry)
            if not won:
                # A concurrent operation reserved the next sequence. Helping it
                # advances the pointer; then this operation recomputes on new state.
                await self._help_next(root_version, seq)
                continue
            advanced = await self.store.cas(_ROOT, root_version, {"seq": seq + 1})
            if advanced:
                return result
            # Another helper may have advanced our reservation. If so the operation
            # is committed; its result remains the one calculated above.
            _, latest = await self.store.read(_ROOT)
            if latest is not None and latest["seq"] >= seq + 1:
                return result
            # An unexpected failed pointer CAS leaves the reservation for a helper.
            await self._help_next(root_version, seq)

    async def submit(self, request_id, updates):
        copied = copy.deepcopy(updates)
        d = digest(copied)
        response = {"bundle": request_id, "revision": 1, "digest": d}

        def transition(state):
            if request_id in state["submits"]:
                return None, copy.deepcopy(state["submits"][request_id]), False
            value = {
                "revision": 1,
                "updates": copied,
                "digest": d,
                "approvals": [],
                "ci": False,
                "published": False,
            }
            return ({"kind": "submit", "id": request_id, "value": value,
                     "response": response}, response, True)

        return await self._commit(transition)

    async def revise(self, bundle, expected_revision, updates):
        copied = copy.deepcopy(updates)
        d = digest(copied)

        def transition(state):
            b = state["bundles"][bundle]
            if expected_revision != b["revision"]:
                return None, "stale", False
            if b["published"]:
                return None, "published", False
            value = {"revision": b["revision"] + 1, "updates": copied,
                     "digest": d, "approvals": [], "ci": False}
            return ({"kind": "revise", "id": bundle, "value": value}, "updated", True)

        return await self._commit(transition)

    async def approve(self, bundle, revision, reviewer):
        if reviewer not in ("alice", "bob", "carol"):
            return False

        def transition(state):
            b = state["bundles"][bundle]
            if revision != b["revision"] or b["published"]:
                return None, False, False
            if reviewer in b["approvals"]:
                return None, True, False
            return ({"kind": "approve", "id": bundle, "reviewer": reviewer}, True, True)

        return await self._commit(transition)

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        def transition(state):
            b = state["bundles"][bundle]
            if revision != b["revision"] or candidate_digest != b["digest"] or b["published"]:
                return None, False, False
            value = bool(passed)
            if b["ci"] == value:
                return None, True, False
            return ({"kind": "ci", "id": bundle, "passed": value}, True, True)

        return await self._commit(transition)

    async def publish(self, bundle, revision, request_id):
        def transition(state):
            if request_id in state["publishes"]:
                return None, state["publishes"][request_id], False
            b = state["bundles"][bundle]
            if revision != b["revision"]:
                outcome = "stale"
            elif b["published"]:
                outcome = "published"
            elif len(set(b["approvals"]) & {"alice", "bob", "carol"}) < 2 or not b["ci"]:
                outcome = "not_ready"
            elif any(state["heads"][branch] != item["expected"]
                     for branch, item in b["updates"].items()):
                outcome = "conflict"
            else:
                outcome = "published"
            heads = ({branch: item["new"] for branch, item in b["updates"].items()}
                     if outcome == "published" and not b["published"] else {})
            entry = {"kind": "publish", "id": bundle, "request_id": request_id,
                     "outcome": outcome, "heads": heads}
            return entry, outcome, True

        return await self._commit(transition)

    async def read_heads(self, branches):
        _, _, state = await self._snapshot()
        return {branch: copy.deepcopy(state["heads"][branch]) for branch in branches}

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
