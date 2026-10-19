"""A small in-memory service; all three examples test this same application."""

from prs import emit


class Inventory:
    def __init__(self):
        self._stock = {}
        self._reservations = {}

    def add_product(self, sku, units):
        if sku in self._stock or units < 0:
            raise ValueError("product must be new and stock nonnegative")
        self._stock[sku] = units
        emit("eStocked", key=sku, units=units)

    def available(self, sku):
        return self._stock[sku]

    def reserve(self, sku, order, units):
        if units <= 0:
            raise ValueError("reservation must be positive")
        if (sku, order) in self._reservations:
            raise ValueError("order already reserved this product")
        if units > self._stock[sku]:
            raise ValueError("insufficient stock")
        self._stock[sku] -= units
        self._reservations[sku, order] = units
        # Emit after the state change succeeds, at the same boundary a database
        # implementation would commit. Rejected requests emit no change event.
        emit("eReserved", key=sku, order=order, units=units)

    def release(self, sku, order):
        units = self._reservations.pop((sku, order))
        self._stock[sku] += units
        emit("eReleased", key=sku, order=order, units=units)
