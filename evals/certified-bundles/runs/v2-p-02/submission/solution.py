"""Linearizable certified change bundles over a single-key CAS commit pointer."""
from copy import deepcopy
from contract import digest


_POINTER = "ccb:pointer"
_ALLOCATOR = "ccb:allocator"
_EVENT_PREFIX = "ccb:event:"
_REVIEWERS = {"alice", "bob", "carol"}


class Service:
    def __init__(self, store, initial_heads):
        self.store = store
        # Construction is local only; every call works from the durable pointer.
        self._initial_heads = deepcopy(initial_heads)

    async def _ensure_pointer(self):
        version, value = await self.store.read(_POINTER)
        if version == 0:
            await self.store.cas(_POINTER, 0, {"tail": None})

    async def _snapshot(self):
        await self._ensure_pointer()
        pointer_version, pointer = await self.store.read(_POINTER)
        tail = pointer["tail"]
        snapshot_tail = tail
        events = []
        while tail is not None:
            _, event = await self.store.read(_EVENT_PREFIX + str(tail))
            events.append(event)
            tail = event["prev"]
        state = {"heads": deepcopy(self._initial_heads), "bundles": {}, "publish": {}}
        for event in reversed(events):
            self._apply(state, event["kind"], event["data"])
        return state, pointer_version, snapshot_tail

    async def _new_event_id(self):
        while True:
            version, value = await self.store.read(_ALLOCATOR)
            n = 0 if version == 0 else value["next"]
            if await self.store.cas(_ALLOCATOR, version, {"next": n + 1}):
                return n

    @staticmethod
    def _apply(state, kind, data):
        """Apply one already committed command; return that command's response."""
        bundles = state["bundles"]
        if kind == "submit":
            bid = data["bundle"]
            if bid in bundles:
                return deepcopy(bundles[bid]["submit_response"])
            updates = deepcopy(data["updates"])
            d = digest(updates)
            response = {"bundle": bid, "revision": 1, "digest": d}
            bundles[bid] = {"revision": 1, "updates": updates, "digest": d,
                            "approvals": [], "ci": False, "published": False,
                            "submit_response": response}
            return deepcopy(response)
        if kind == "revise":
            b = bundles[data["bundle"]]
            if data["expected_revision"] != b["revision"]:
                return "stale"
            if b["published"]:
                return "published"
            b["revision"] += 1
            b["updates"] = deepcopy(data["updates"])
            b["digest"] = digest(b["updates"])
            b["approvals"] = []
            b["ci"] = False
            return "updated"
        if kind == "approve":
            if data["reviewer"] not in _REVIEWERS:
                return False
            b = bundles[data["bundle"]]
            if data["revision"] != b["revision"] or b["published"]:
                return False
            if data["reviewer"] not in b["approvals"]:
                b["approvals"].append(data["reviewer"])
                b["approvals"].sort()
            return True
        if kind == "ci":
            b = bundles[data["bundle"]]
            if (data["revision"] != b["revision"] or
                    data["candidate_digest"] != b["digest"] or b["published"]):
                return False
            b["ci"] = bool(data["passed"])
            return True
        if kind == "publish":
            rid = data["request_id"]
            if rid in state["publish"]:
                return state["publish"][rid]
            b = bundles[data["bundle"]]
            if data["revision"] != b["revision"]:
                result = "stale"
            elif b["published"]:
                result = "published"
            elif len(set(b["approvals"]) & _REVIEWERS) < 2 or not b["ci"]:
                result = "not_ready"
            elif any(state["heads"][branch] != update["expected"]
                     for branch, update in b["updates"].items()):
                result = "conflict"
            else:
                for branch, update in b["updates"].items():
                    state["heads"][branch] = update["new"]
                b["published"] = True
                result = "published"
            state["publish"][rid] = result
            return result
        raise AssertionError("unknown event kind")

    async def _command(self, kind, data):
        """Serialize a state transition at the pointer CAS; retry CAS races."""
        while True:
            state, pointer_version, snapshot_tail = await self._snapshot()
            # A trial copy lets us compute the response without changing the
            # snapshot used to prepare the immutable event.
            result = self._apply(deepcopy(state), kind, data)
            # Commands whose result is determined without changing durable state
            # can linearize at the pointer snapshot they just read.
            should_commit = True
            if kind == "approve":
                b = state["bundles"][data["bundle"]]
                should_commit = (data["reviewer"] in _REVIEWERS and
                                 data["revision"] == b["revision"] and
                                 not b["published"] and data["reviewer"] not in b["approvals"])
            elif kind == "ci":
                b = state["bundles"][data["bundle"]]
                should_commit = (data["revision"] == b["revision"] and
                                 data["candidate_digest"] == b["digest"] and
                                 not b["published"] and bool(data["passed"]) != b["ci"])
            elif kind == "revise":
                b = state["bundles"][data["bundle"]]
                should_commit = (data["expected_revision"] == b["revision"] and
                                 not b["published"])
            elif kind == "submit":
                should_commit = data["bundle"] not in state["bundles"]
            elif kind == "publish":
                # Even rejected outcomes must be durably memoized.
                should_commit = data["request_id"] not in state["publish"]
            if not should_commit:
                return deepcopy(result)

            # The event is immutable and unique. The pointer CAS is the single
            # commit point for the operation and rejects any intervening writer.
            eid = await self._new_event_id()
            event = {"prev": snapshot_tail, "kind": kind, "data": deepcopy(data)}
            await self.store.cas(_EVENT_PREFIX + str(eid), 0, event)
            if await self.store.cas(_POINTER, pointer_version, {"tail": eid}):
                return deepcopy(result)

    async def submit(self, request_id, updates):
        return await self._command("submit", {"bundle": request_id, "updates": deepcopy(updates)})

    async def revise(self, bundle, expected_revision, updates):
        return await self._command("revise", {"bundle": bundle, "expected_revision": expected_revision,
                                                "updates": deepcopy(updates)})

    async def approve(self, bundle, revision, reviewer):
        return await self._command("approve", {"bundle": bundle, "revision": revision, "reviewer": reviewer})

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        return await self._command("ci", {"bundle": bundle, "revision": revision,
                                            "candidate_digest": candidate_digest, "passed": bool(passed)})

    async def publish(self, bundle, revision, request_id):
        return await self._command("publish", {"bundle": bundle, "revision": revision,
                                                "request_id": request_id})

    async def read_heads(self, branches):
        state, _, _ = await self._snapshot()
        return {branch: deepcopy(state["heads"][branch]) for branch in branches}

    async def get_status(self, bundle):
        state, _, _ = await self._snapshot()
        b = state["bundles"][bundle]
        return {"revision": b["revision"], "updates": deepcopy(b["updates"]),
                "digest": b["digest"], "approvals": sorted(b["approvals"]),
                "ci": bool(b["ci"]), "published": bool(b["published"])}
