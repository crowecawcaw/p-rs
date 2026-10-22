"""A public deterministic storage runtime; grader changes schedules, not semantics."""
import asyncio
import copy
import json
import random
from collections import deque

class Crash(BaseException):
    pass

class Store:
    def __init__(self, seed=0, crash_at=None, hook=None):
        self._records = {}
        self._rng = random.Random(seed)
        self.steps = 0
        self.writes = 0
        self.crash_at = crash_at
        self.hook = hook
        self.trace = deque(maxlen=20000)

    async def _yield(self, event):
        self.steps += 1
        self.trace.append(event)
        if self.hook:
            await self.hook(event)
        for _ in range(self._rng.randrange(1, 4)):
            await asyncio.sleep(0)

    async def read(self, key):
        await self._yield(('read.before', key))
        result = copy.deepcopy(self._records.get(key, (0, None)))
        await self._yield(('read.after', key))
        return result

    async def cas(self, key, expected_version, value):
        if len(json.dumps(value, separators=(',', ':'), ensure_ascii=True).encode()) > 16384:
            raise ValueError('record exceeds 16384 bytes')
        value = copy.deepcopy(value)
        await self._yield(('cas.before', key))
        version = self._records.get(key, (0, None))[0]
        ok = version == expected_version
        if ok:
            self._records[key] = (version + 1, value)
            self.writes += 1
            if self.crash_at == self.writes:
                raise Crash('process died after durable write, before response')
        await self._yield(('cas.after', key, ok))
        return ok
