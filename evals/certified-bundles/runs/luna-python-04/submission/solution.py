"""Linearizable certified bundle service backed only by the supplied CAS store."""
import json
from contract import digest

_PAGE_CHARS = 6000
_ROOT = "certified-bundles/root"
_TICKER = "certified-bundles/ticket"
_PREFIX = "certified-bundles/page/"
_AUTHORIZED = {"alice", "bob", "carol"}


def _copy_json(value):
    return json.loads(json.dumps(value, separators=(",", ":"), ensure_ascii=True))


class Service:
    def __init__(self, store, initial_heads):
        self.store = store
        # Constructor intentionally performs no I/O. This copy is immutable input
        # material used only if the shared store has not yet been initialized.
        self._initial_heads = _copy_json(initial_heads)

    async def _new_ticket(self):
        while True:
            version, current = await self.store.read(_TICKER)
            current = 0 if current is None else current
            ticket = current + 1
            if await self.store.cas(_TICKER, version, ticket):
                return ticket

    async def _write_pages(self, ticket, state):
        encoded = json.dumps(state, sort_keys=True, separators=(",", ":"), ensure_ascii=True)
        pieces = [encoded[i:i + _PAGE_CHARS] for i in range(0, len(encoded), _PAGE_CHARS)]
        # Every ticket is uniquely allocated by CAS, so these page keys are
        # private to this attempted commit and are never overwritten.
        for index, piece in enumerate(pieces):
            ok = await self.store.cas(f"{_PREFIX}{ticket}/{index}", 0, {"data": piece})
            if not ok:
                # This can only indicate an unexpected store/key collision. Leave
                # the unreachable pages in place; the caller can retry at a new ticket.
                return None
        return len(pieces)

    async def _install(self, expected_root_version, state):
        ticket = await self._new_ticket()
        count = await self._write_pages(ticket, state)
        if count is None:
            return False
        return await self.store.cas(_ROOT, expected_root_version,
                                    {"ticket": ticket, "count": count})

    async def _snapshot(self):
        while True:
            root_version, root = await self.store.read(_ROOT)
            if root is None:
                initial = {"heads": _copy_json(self._initial_heads),
                           "bundles": {}, "publishes": {}}
                await self._install(root_version, initial)
                continue
            fragments = []
            for index in range(root["count"]):
                _, page = await self.store.read(f"{_PREFIX}{root['ticket']}/{index}")
                if page is None:
                    raise RuntimeError("committed state page is missing")
                fragments.append(page["data"])
            return root_version, json.loads("".join(fragments))

    async def _change(self, transition):
        """Run a deterministic state transition, retrying optimistic conflicts."""
        while True:
            version, state = await self._snapshot()
            changed, result = transition(state)
            if not changed:
                return result
            if await self._install(version, state):
                return result

    async def submit(self, request_id, updates):
        candidate = _copy_json(updates)
        candidate_digest = digest(candidate)

        def transition(state):
            old = state["bundles"].get(request_id)
            if old is not None:
                return False, _copy_json(old["submit_response"])
            response = {"bundle": request_id, "revision": 1, "digest": candidate_digest}
            state["bundles"][request_id] = {
                "revision": 1, "updates": candidate, "digest": candidate_digest,
                "approvals": [], "ci": False, "published": False,
                "submit_response": response,
            }
            return True, _copy_json(response)

        return await self._change(transition)

    async def revise(self, bundle, expected_revision, updates):
        candidate = _copy_json(updates)
        candidate_digest = digest(candidate)

        def transition(state):
            item = state["bundles"][bundle]
            if expected_revision != item["revision"]:
                return False, "stale"
            if item["published"]:
                return False, "published"
            item["revision"] += 1
            item["updates"] = candidate
            item["digest"] = candidate_digest
            item["approvals"] = []
            item["ci"] = False
            return True, "updated"

        return await self._change(transition)

    async def approve(self, bundle, revision, reviewer):
        def transition(state):
            item = state["bundles"][bundle]
            if reviewer not in _AUTHORIZED or revision != item["revision"] or item["published"]:
                return False, False
            if reviewer not in item["approvals"]:
                item["approvals"].append(reviewer)
                item["approvals"].sort()
                return True, True
            return False, True

        return await self._change(transition)

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        def transition(state):
            item = state["bundles"][bundle]
            if (revision != item["revision"] or candidate_digest != item["digest"]
                    or item["published"]):
                return False, False
            result = bool(passed)
            if item["ci"] == result:
                return False, True
            item["ci"] = result
            return True, True

        return await self._change(transition)

    async def publish(self, bundle, revision, request_id):
        def transition(state):
            if request_id in state["publishes"]:
                return False, state["publishes"][request_id]
            item = state["bundles"][bundle]
            if revision != item["revision"]:
                outcome = "stale"
            elif item["published"]:
                outcome = "published"
            elif len(set(item["approvals"]) & _AUTHORIZED) < 2 or not item["ci"]:
                outcome = "not_ready"
            elif any(state["heads"][branch] != change["expected"]
                     for branch, change in item["updates"].items()):
                outcome = "conflict"
            else:
                for branch, change in item["updates"].items():
                    state["heads"][branch] = change["new"]
                item["published"] = True
                outcome = "published"
            state["publishes"][request_id] = outcome
            return True, outcome

        return await self._change(transition)

    async def read_heads(self, branches):
        _, state = await self._snapshot()
        return {branch: state["heads"][branch] for branch in branches}

    async def get_status(self, bundle):
        _, state = await self._snapshot()
        item = state["bundles"][bundle]
        return {
            "revision": item["revision"],
            "updates": _copy_json(item["updates"]),
            "digest": item["digest"],
            "approvals": sorted(item["approvals"]),
            "ci": bool(item["ci"]),
            "published": bool(item["published"]),
        }
