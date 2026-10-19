import pytest

from inventory_app import Inventory


@pytest.fixture
def inventory():
    # Function scope: fresh stock per test. The plugin records fixture setup too.
    service = Inventory()
    service.add_product("coffee", 3)
    service.add_product("tea", 2)
    return service
