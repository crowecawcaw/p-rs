mod support;

use inventory_integration_example::Error;
use support::with_inventory;

#[test]
fn cancellation_restores_stock() {
    with_inventory(|inventory| {
        inventory.reserve("coffee", "order-1", 2).unwrap();
        inventory.release("coffee", "order-1").unwrap();
        assert_eq!(inventory.available("coffee"), Some(3));
    });
}

#[test]
fn products_have_independent_monitor_state() {
    with_inventory(|inventory| {
        // The same order ID is valid for different product partition keys.
        inventory.reserve("coffee", "order-1", 3).unwrap();
        inventory.reserve("tea", "order-1", 2).unwrap();
        assert_eq!(inventory.available("coffee"), Some(0));
        assert_eq!(inventory.available("tea"), Some(0));
        assert_eq!(inventory.reserve("tea", "order-2", 1), Err(Error::InsufficientStock));
    });
}
