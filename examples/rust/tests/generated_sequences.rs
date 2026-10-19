mod support;

use inventory_integration_example::Error;
use proptest::prelude::*;
use support::{inventory, model};

#[derive(Clone, Debug)]
enum Command {
    Reserve { sku: usize, order: u8, units: i64 },
    Release { sku: usize, order: u8 },
}

fn command() -> impl Strategy<Value = Command> {
    prop_oneof![
        (0usize..2, 0u8..6, -1i64..7).prop_map(|(sku, order, units)| Command::Reserve { sku, order, units }),
        (0usize..2, 0u8..6).prop_map(|(sku, order)| Command::Release { sku, order }),
    ]
}

proptest! {
    // A bounded CI budget, with shrinking and persisted regression seeds enabled.
    #![proptest_config(ProptestConfig { cases: 64, max_shrink_iters: 2048, failure_persistence: Some(Box::new(proptest::test_runner::FileFailurePersistence::WithSource("proptest-regressions"))), ..ProptestConfig::default() })]

    #[test]
    fn generated_stateful_operations(commands in prop::collection::vec(command(), 1..41)) {
        // This scope runs afresh for EVERY generated case AND shrink attempt,
        // not once for the enclosing #[test]. Never reuse inventory or recorder.
        let recorder = model().recording();
        let mut inventory = inventory(recorder.sink());
        inventory.stock("coffee", 3).unwrap();
        inventory.stock("tea", 2).unwrap();
        for command in &commands {
            let result = match command {
                Command::Reserve { sku, order, units } =>
                    inventory.reserve(["coffee", "tea"][*sku], &format!("order-{order}"), *units),
                Command::Release { sku, order } =>
                    inventory.release(["coffee", "tea"][*sku], &format!("order-{order}")),
            };
            // Rejection is normal input, not a discarded generated case. Unexpected
            // errors are failures. P independently reconstructs accepted operations.
            prop_assert!(matches!(result, Ok(()) | Err(Error::InvalidQuantity)
                | Err(Error::DuplicateOrder) | Err(Error::InsufficientStock) | Err(Error::UnknownOrder)),
                "unexpected result for {:?}: {:?}", command, result);
        }
        // Return a proptest failure, so a P violation shrinks the COMMAND sequence.
        let checked = recorder.check();
        prop_assert!(checked.is_ok(), "{}", checked.unwrap_err());
        for sku in ["coffee", "tea"] {
            prop_assert!(inventory.available(sku).unwrap() >= 0);
        }
    }
}
