"""A tiny lock server with leases, instrumented with `emit`."""

from collections import deque

from prs import emit


class LockServer:
    def __init__(self, lease: float = 10.0, clock=None, buggy_expiry: bool = False):
        self.lease = lease
        self.now = clock or (lambda: 0.0)
        self.buggy_expiry = buggy_expiry
        self.holder = {}  # lock -> (client, expires)
        self.queue = {}  # lock -> waiting clients

    def acquire(self, client: str, lock: str = "L1") -> bool:
        emit("eRequest", key=lock, client=client)
        held = self.holder.get(lock)
        if held and held[1] <= self.now():
            if not self.buggy_expiry:
                emit("eExit", key=lock, client=held[0])  # the lease ran out
            del self.holder[lock]
            held = None
        if held is None:
            self.holder[lock] = (client, self.now() + self.lease)
            emit("eGrant", key=lock, client=client)
            emit("eEnter", key=lock, client=client)
            return True
        self.queue.setdefault(lock, deque()).append(client)
        return False

    def release(self, client: str, lock: str = "L1") -> None:
        held = self.holder.get(lock)
        if not held or held[0] != client:
            return
        emit("eExit", key=lock, client=client)
        del self.holder[lock]
        waiting = self.queue.get(lock)
        if waiting:
            nxt = waiting.popleft()
            self.holder[lock] = (nxt, self.now() + self.lease)
            emit("eGrant", key=lock, client=nxt)
            emit("eEnter", key=lock, client=nxt)
