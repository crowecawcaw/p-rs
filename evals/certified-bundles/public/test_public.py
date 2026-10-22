import unittest
from contract import digest
from runtime import Store
from solution import Service

HEADS = {'schema': 's0', 'service': 'v0', 'client': 'c0'}
UPDATES = {'schema': {'expected': 's0', 'new': 's1'}, 'service': {'expected': 'v0', 'new': 'v1'}}

class PublicTests(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self):
        self.store = Store()
        self.s = Service(self.store, HEADS.copy())

    async def ready(self, bid='b', updates=None):
        updates = updates or UPDATES
        result = await self.s.submit(bid, updates)
        self.assertEqual(result, {'bundle': bid, 'revision': 1, 'digest': digest(updates)})
        self.assertTrue(await self.s.approve(bid, 1, 'alice'))
        self.assertTrue(await self.s.approve(bid, 1, 'bob'))
        self.assertTrue(await self.s.record_ci(bid, 1, digest(updates), True))

    async def test_publish_and_restart(self):
        await self.ready()
        self.assertEqual(await self.s.publish('b', 1, 'p1'), 'published')
        other = Service(self.store, HEADS.copy())
        self.assertEqual(await other.read_heads(list(HEADS)), {'schema': 's1', 'service': 'v1', 'client': 'c0'})
        self.assertTrue((await other.get_status('b'))['published'])
        self.assertEqual(await other.publish('b', 1, 'p1'), 'published')

    async def test_revision_invalidates_evidence(self):
        await self.ready()
        self.assertEqual(await self.s.revise('b', 1, UPDATES), 'updated')
        status = await self.s.get_status('b')
        self.assertEqual(status['revision'], 2)
        self.assertEqual(status['approvals'], [])
        self.assertFalse(status['ci'])
        self.assertFalse(await self.s.approve('b', 1, 'carol'))
        self.assertEqual(await self.s.publish('b', 2, 'p2'), 'not_ready')

    async def test_rejected_publish_is_idempotent(self):
        await self.s.submit('b', UPDATES)
        self.assertEqual(await self.s.publish('b', 1, 'p1'), 'not_ready')
        await self.s.approve('b', 1, 'alice')
        await self.s.approve('b', 1, 'bob')
        await self.s.record_ci('b', 1, digest(UPDATES), True)
        self.assertEqual(await self.s.publish('b', 1, 'p1'), 'not_ready')
        self.assertEqual(await self.s.publish('b', 1, 'p2'), 'published')

    async def test_stale_base(self):
        await self.ready('a')
        await self.ready('b')
        self.assertEqual(await self.s.publish('a', 1, 'pa'), 'published')
        self.assertEqual(await self.s.publish('b', 1, 'pb'), 'conflict')

if __name__ == '__main__':
    unittest.main()
