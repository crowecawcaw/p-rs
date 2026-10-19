import pytest


def test_release_makes_stock_available_again(inventory):
    inventory.reserve("coffee", "order-1", 3)
    inventory.release("coffee", "order-1")
    inventory.reserve("coffee", "order-2", 3)
    assert inventory.available("coffee") == 0


def test_rejected_reservation_changes_nothing(inventory, prs_events):
    inventory.reserve("coffee", "order-1", 2)
    with pytest.raises(ValueError, match="insufficient stock"):
        inventory.reserve("coffee", "order-2", 2)
    assert inventory.available("coffee") == 1
    # prs_events is optional; use it to assert on instrumentation.
    assert prs_events.count("eStocked") == 2
    assert prs_events.count("eReserved") == 1


def test_products_have_independent_stock(inventory):
    inventory.reserve("coffee", "order-1", 3)
    inventory.reserve("tea", "order-1", 2)
    assert inventory.available("coffee") == inventory.available("tea") == 0
