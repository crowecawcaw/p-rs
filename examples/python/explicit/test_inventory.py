import pytest

from prs import check_events, check_spec, expect_violation


def test_release_makes_stock_available_again(inventory_factory):
    with check_spec("InventoryCorrect") as events:
        inventory = inventory_factory()
        inventory.reserve("coffee", "order-1", 3)
        inventory.release("coffee", "order-1")
        inventory.reserve("coffee", "order-2", 3)
        assert inventory.available("coffee") == 0
    assert events.count("eReleased") == 1


def test_rejected_reservation_changes_nothing(inventory_factory):
    with check_spec("InventoryCorrect") as events:
        inventory = inventory_factory()
        inventory.reserve("coffee", "order-1", 2)
        with pytest.raises(ValueError, match="insufficient stock"):
            inventory.reserve("coffee", "order-2", 2)
        assert inventory.available("coffee") == 1
    assert events.count("eReserved") == 1


def test_products_have_independent_stock(inventory_factory):
    with check_spec("InventoryCorrect"):
        inventory = inventory_factory()
        # The same order can reserve different products; their keys separate them.
        inventory.reserve("coffee", "order-1", 3)
        inventory.reserve("tea", "order-1", 2)
        assert inventory.available("coffee") == inventory.available("tea") == 0


def test_spec_rejects_overselling():
    # A negative control for the spec, not an intentionally failing CI test.
    with expect_violation("InventoryCorrect", match="reservation exceeds available stock"):
        check_events([
            {"event": "eStocked", "key": "coffee", "units": 1},
            {"event": "eReserved", "key": "coffee", "order": "order-1", "units": 2},
        ], "InventoryCorrect")
