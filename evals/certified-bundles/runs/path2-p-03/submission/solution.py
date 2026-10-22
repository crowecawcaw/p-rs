"""Durable, linearizable change bundles over the store's single-key CAS API."""
import copy
import hashlib
import json

from contract import digest

_ROOT = "ccb:root"


def _canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def _node_id(node):
    return hashlib.sha256(_canonical(node).encode("utf-8")).hexdigest()


class _Tree:
    """Immutable path-compressed radix trie behind one root CAS pointer."""
    def __init__(self, store):
        self.store = store
        self.cache = {}

    @staticmethod
    def _path(key):
        return hashlib.sha256(key.encode("utf-8")).hexdigest()

    async def _load(self, node_id):
        if node_id is None:
            return None
        if node_id not in self.cache:
            _, node = await self.store.read("ccb:node:" + node_id)
            self.cache[node_id] = node
        return self.cache[node_id]

    async def get(self, root, key):
        path = self._path(key)
        node_id = root
        offset = 0
        while node_id is not None:
            node = await self._load(node_id)
            if "k" in node:
                return copy.deepcopy(node["v"]) if node["k"] == key else None
            prefix = node["p"]
            if not path.startswith(prefix, offset):
                return None
            offset += len(prefix)
            if offset >= len(path):
                return None
            node_id = node["c"].get(path[offset])
            offset += 1
        return None

    async def set_many(self, root, changes):
        async def insert(node_id, key, value, offset):
            path = self._path(key)
            old = await self._load(node_id)
            if old is None:
                return await self._store_node({"k": key, "v": copy.deepcopy(value)})
            if "k" in old:
                old_path = self._path(old["k"])
                if old["k"] == key:
                    return await self._store_node({"k": key, "v": copy.deepcopy(value)})
                common = 0
                while (offset + common < len(path)
                       and offset + common < len(old_path)
                       and path[offset + common] == old_path[offset + common]):
                    common += 1
                if offset + common >= len(path) or offset + common >= len(old_path):
                    raise ValueError("trie key hash collision")
                new_leaf = await self._store_node({"k": key, "v": copy.deepcopy(value)})
                children = {
                    old_path[offset + common]: node_id,
                    path[offset + common]: new_leaf,
                }
                return await self._store_node({"p": path[offset:offset + common], "c": children})

            prefix = old["p"]
            common = 0
            while (common < len(prefix) and offset + common < len(path)
                   and prefix[common] == path[offset + common]):
                common += 1
            if common == len(prefix):
                split = offset + common
                if split >= len(path):
                    raise ValueError("trie key hash collision")
                children = dict(old["c"])
                digit = path[split]
                children[digit] = await insert(children.get(digit), key, value, split + 1)
                return await self._store_node({"p": prefix, "c": children})

            # The new key diverges inside this compressed prefix. Keep the old
            # subtree under its first differing digit and add the new leaf.
            split = offset + common
            if split >= len(path):
                raise ValueError("trie key hash collision")
            old_children = dict(old["c"])
            shortened = await self._store_node({"p": prefix[common + 1:], "c": old_children})
            new_leaf = await self._store_node({"k": key, "v": copy.deepcopy(value)})
            children = {
                prefix[common]: shortened,
                path[split]: new_leaf,
            }
            return await self._store_node({"p": prefix[:common], "c": children})

        for key, value in changes.items():
            root = await insert(root, key, value, 0)
        return root

    async def _store_node(self, node):
        node_id = _node_id(node)
        if node_id not in self.cache:
            await self.store.cas("ccb:node:" + node_id, 0, node)
            self.cache[node_id] = node
        return node_id


class Service:
    def __init__(self, store, initial_heads):
        self.store = store
        self._initial_heads = copy.deepcopy(initial_heads)

    async def _ensure(self):
        version, root = await self.store.read(_ROOT)
        if version:
            return version, root
        tree = _Tree(self.store)
        changes = {"H:" + branch: head for branch, head in self._initial_heads.items()}
        root = await tree.set_many(None, changes) if changes else None
        if await self.store.cas(_ROOT, 0, root):
            return 1, root
        return await self.store.read(_ROOT)

    async def _read(self, keys):
        version, root = await self._ensure()
        tree = _Tree(self.store)
        values = {}
        for key in keys:
            values[key] = await tree.get(root, key)
        return version, root, values

    async def _commit(self, decide):
        """Run a state-dependent operation until its root-pointer CAS commits."""
        while True:
            version, root = await self._ensure()
            tree = _Tree(self.store)
            result, changes = await decide(tree, root)
            if not changes:
                return result
            new_root = await tree.set_many(root, changes)
            if await self.store.cas(_ROOT, version, new_root):
                return result

    async def submit(self, request_id, updates):
        updates = copy.deepcopy(updates)
        response = {"bundle": request_id, "revision": 1, "digest": digest(updates)}

        async def decide(tree, root):
            prior = await tree.get(root, "S:" + request_id)
            if prior is not None:
                return prior, {}
            state = {
                "revision": 1,
                "updates": updates,
                "digest": response["digest"],
                "approvals": [],
                "ci": False,
                "published": False,
            }
            return response, {"S:" + request_id: response, "B:" + request_id: state}
        return copy.deepcopy(await self._commit(decide))

    async def revise(self, bundle, expected_revision, updates):
        updates = copy.deepcopy(updates)

        async def decide(tree, root):
            state = await tree.get(root, "B:" + bundle)
            if expected_revision != state["revision"]:
                return "stale", {}
            if state["published"]:
                return "published", {}
            new_state = dict(state)
            new_state.update(revision=state["revision"] + 1, updates=updates,
                             digest=digest(updates), approvals=[], ci=False)
            return "updated", {"B:" + bundle: new_state}
        return await self._commit(decide)

    async def approve(self, bundle, revision, reviewer):
        if reviewer not in ("alice", "bob", "carol"):
            return False

        async def decide(tree, root):
            state = await tree.get(root, "B:" + bundle)
            if revision != state["revision"] or state["published"]:
                return False, {}
            approvals = set(state["approvals"])
            approvals.add(reviewer)
            new_state = dict(state)
            new_state["approvals"] = sorted(approvals)
            if new_state["approvals"] == state["approvals"]:
                return True, {}
            return True, {"B:" + bundle: new_state}
        return await self._commit(decide)

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        async def decide(tree, root):
            state = await tree.get(root, "B:" + bundle)
            if (revision != state["revision"] or candidate_digest != state["digest"]
                    or state["published"]):
                return False, {}
            new_state = dict(state)
            new_state["ci"] = bool(passed)
            if new_state["ci"] == state["ci"]:
                return True, {}
            return True, {"B:" + bundle: new_state}
        return await self._commit(decide)

    async def publish(self, bundle, revision, request_id):
        async def decide(tree, root):
            prior = await tree.get(root, "P:" + request_id)
            if prior is not None:
                return prior, {}
            state = await tree.get(root, "B:" + bundle)
            if revision != state["revision"]:
                outcome = "stale"
            elif state["published"]:
                outcome = "published"
            elif len(set(state["approvals"]) & {"alice", "bob", "carol"}) < 2 or not state["ci"]:
                outcome = "not_ready"
            else:
                head_keys = ["H:" + branch for branch in state["updates"]]
                heads = {key: await tree.get(root, key) for key in head_keys}
                if any(heads["H:" + branch] != item["expected"]
                       for branch, item in state["updates"].items()):
                    outcome = "conflict"
                else:
                    outcome = "published"
                    changes = {"P:" + request_id: outcome}
                    changes.update({"H:" + branch: item["new"]
                                    for branch, item in state["updates"].items()})
                    new_state = dict(state)
                    new_state["published"] = True
                    changes["B:" + bundle] = new_state
                    return outcome, changes
            return outcome, {"P:" + request_id: outcome}
        return await self._commit(decide)

    async def read_heads(self, branches):
        _, root = await self._ensure()
        tree = _Tree(self.store)
        result = {}
        for branch in branches:
            result[branch] = await tree.get(root, "H:" + branch)
        return copy.deepcopy(result)

    async def get_status(self, bundle):
        _, root = await self._ensure()
        state = await _Tree(self.store).get(root, "B:" + bundle)
        return {
            "revision": state["revision"],
            "updates": copy.deepcopy(state["updates"]),
            "digest": state["digest"],
            "approvals": sorted(state["approvals"]),
            "ci": bool(state["ci"]),
            "published": bool(state["published"]),
        }
