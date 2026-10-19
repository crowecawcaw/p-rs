# P model: inventory reservations

A P model of the inventory service that the [language examples](../README.md)
implement, checked against the same [InventoryCorrect spec](PSpec/inventory.p).

* [`PSrc/Inventory.p`](PSrc/Inventory.p): the service for one product, and orders that
  reserve stock and may cancel. The service `announce`s the spec's events after each
  stock change, where the applications call `emit`.
* [`PTst/TestDriver.p`](PTst/TestDriver.p): three orders of 1 or 2 units compete for 3
  units of stock.
* [`PTst/TestScript.p`](PTst/TestScript.p): one test case per design.

The model has two designs for a reservation:

* **Atomic** (`tcAtomicReserve`): one request checks stock and reserves it. This is
  what the applications do.
* **Check then reserve** (`tcCheckThenReserve`): an order reads the available stock,
  then commits a reservation the service no longer checks. Two orders can both see 3
  units and reserve 2 each.

## Run

From the repository root:

```sh
cargo build -p prs-cli --bin prs
cd examples/model
../../target/debug/prs check tcAtomicReserve --exhaustive
../../target/debug/prs check tcCheckThenReserve
```

The first explores every interleaving and data choice and finds no bug:

```text
..... Found 0 bugs.
..... Search finished: every behaviour of the test was explored.
```

The second finds the oversell within a few schedules, and exits with 1:

```text
<ErrorLog> Assertion Failed: PSpec/inventory.p:27:7 reservation exceeds available stock
```

The trace in `PCheckerOutput/tcCheckThenReserve/BugFinding/` shows both orders reading
3 units before either commits. `prs check` with no test case checks both, so it exits
with 1 too.

The spec is the same file the language examples check. A unit test of the Python
service runs one ordering per test; the model explores every ordering of concurrent
orders, which tests of an implementation do not reach. The
[CI workflow](../../.github/workflows/model-example.yml) runs both test cases and
checks that every example's `specs/inventory.p` links here.
