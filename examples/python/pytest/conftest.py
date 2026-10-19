import pytest

from inventory_app import Inventory


@pytest.fixture
def inventory():
    # Function scope gives every test fresh stock AND a fresh event stream.
    # The plugin starts recording before this setup runs.
    service = Inventory()
    service.add_product("coffee", 3)
    service.add_product("tea", 2)
    return service
