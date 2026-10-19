import pytest

from inventory_app import Inventory


@pytest.fixture
def inventory_factory():
    # Call inside check_spec: seeding before recording starts would hide eStocked.
    def create():
        inventory = Inventory()
        inventory.add_product("coffee", 3)
        inventory.add_product("tea", 2)
        return inventory

    return create
