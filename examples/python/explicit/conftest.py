import pytest

from inventory_app import Inventory


@pytest.fixture
def inventory_factory():
    # Construct AND seed inside check_spec so the initial-stock event is recorded.
    # A pre-seeded fixture would run before the explicit block starts recording.
    def create():
        inventory = Inventory()
        inventory.add_product("coffee", 3)
        inventory.add_product("tea", 2)
        return inventory

    return create
