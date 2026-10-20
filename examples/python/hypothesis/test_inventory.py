"""Generate inventory histories, checking each history with its own P monitor."""

import sys

import pytest
from hypothesis import settings, strategies as st
from hypothesis.stateful import RuleBasedStateMachine, initialize, invariant, precondition, rule

from inventory_app import Inventory
from prs import Recorder


class InventoryHistories(RuleBasedStateMachine):
    def __init__(self):
        super().__init__()
        # One machine per generated history and shrink attempt, so one recorder each.
        self.events = Recorder(["InventoryCorrect"]).start()
        self.initial_stock = {"coffee": 3, "tea": 2}
        self.reservations = {}
        self.next_order = 0

    @initialize()
    def seed_inventory(self):
        # Seed inside the recording; teardown still runs if this fails.
        self.inventory = Inventory()
        for sku, units in self.initial_stock.items():
            self.inventory.add_product(sku, units)

    @precondition(lambda self: any(self.inventory.available(sku) for sku in self.initial_stock))
    @rule(data=st.data())
    def reserve(self, data):
        available_skus = [sku for sku in self.initial_stock if self.inventory.available(sku) > 0]
        sku = data.draw(st.sampled_from(available_skus), label="sku")
        units = data.draw(st.integers(1, self.inventory.available(sku)), label="units")
        order = "order-{}".format(self.next_order)
        self.next_order += 1
        self.inventory.reserve(sku, order, units)
        self.reservations[sku, order] = units

    @precondition(lambda self: bool(self.reservations))
    @rule(data=st.data())
    def release(self, data):
        sku, order = data.draw(st.sampled_from(sorted(self.reservations)), label="reservation")
        self.inventory.release(sku, order)
        del self.reservations[sku, order]

    @rule(sku=st.sampled_from(["coffee", "tea"]))
    def reject_overselling(self, sku):
        before = self.events.count("eReserved")
        with pytest.raises(ValueError, match="insufficient stock"):
            self.inventory.reserve(sku, "oversized-order", self.inventory.available(sku) + 1)
        assert self.events.count("eReserved") == before

    @precondition(lambda self: bool(self.reservations))
    @rule(data=st.data())
    def reject_duplicate_order(self, data):
        sku, order = data.draw(st.sampled_from(sorted(self.reservations)), label="reservation")
        before = self.events.count("eReserved")
        with pytest.raises(ValueError, match="order already reserved"):
            self.inventory.reserve(sku, order, 1)
        assert self.events.count("eReserved") == before

    @invariant()
    def returned_stock_matches_successful_operations(self):
        # P sees only emitted events; this also checks the service's own state.
        for sku, initial in self.initial_stock.items():
            reserved = sum(units for (product, _), units in self.reservations.items() if product == sku)
            assert self.inventory.available(sku) == initial - reserved

    def teardown(self):
        # Check before the next history. SpecViolation is an AssertionError,
        # so Hypothesis shrinks it like any other failure. Teardown also runs
        # after a failed step: skip the check then, so that failure is reported.
        self.events.stop()
        if sys.exc_info()[1] is None:
            self.events.check()


TestInventoryHistories = InventoryHistories.TestCase
TestInventoryHistories.settings = settings(
    max_examples=30,
    stateful_step_count=25,
    deadline=None,  # each history starts the checker process
)
