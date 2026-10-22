"""Public wire contract. Implement Service in solution.py; no third-party packages."""
import hashlib
import json


def digest(updates):
    """CI certifies both the expected bases and proposed commit IDs."""
    return hashlib.sha256(json.dumps(updates, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


class Service:
    def __init__(self, store, initial_heads):
        """initial_heads is identical across instances/restarts. No I/O in constructor."""
        self.store = store

    async def submit(self, request_id, updates):
        """Return {'bundle': request_id, 'revision': 1, 'digest': digest(updates)}."""
        raise NotImplementedError

    async def revise(self, bundle, expected_revision, updates):
        """Return 'updated', 'stale', or 'published'."""
        raise NotImplementedError

    async def approve(self, bundle, revision, reviewer):
        """Return bool: whether this approval was accepted."""
        raise NotImplementedError

    async def record_ci(self, bundle, revision, candidate_digest, passed):
        """Return bool: whether this result was accepted."""
        raise NotImplementedError

    async def publish(self, bundle, revision, request_id):
        """Return 'published', 'stale', 'not_ready', or 'conflict'."""
        raise NotImplementedError

    async def read_heads(self, branches):
        """Return {branch: commit_id}, from one consistent snapshot."""
        raise NotImplementedError

    async def get_status(self, bundle):
        """Return normalized status; fields specified in TASK.md."""
        raise NotImplementedError
