# Java: inventory reservations

Three JUnit test classes check one [inventory service](src/main/java/example/Inventory.java)
against the [P spec](specs/inventory.p) with the [Java helpers](../../java/README.md),
which run the `prs observe` CLI. See the
[shared overview](../README.md#how-the-checks-work) for what the checks do and why.

## Run

From the repository root, with JDK 17+, Maven 3.9.9 and Rust:

```sh
cargo build --locked -p prs-cli
export PRS_BIN="$PWD/target/debug/prs"
mvn --batch-mode --no-transfer-progress -f java/pom.xml install
mvn --batch-mode --no-transfer-progress -f examples/java/pom.xml test
```

The helpers aren't on Maven Central, so the first Maven command installs them
locally. From `examples/java`, run one class with
`mvn test -Dtest=GeneratedInventoryTest`. The
[CI workflow](../../.github/workflows/examples-java.yml) runs them with Java 17.

`Inventory` takes an injected `Inventory.Events` interface, adapted with a lambda:

```java
var inventory = new Inventory(
    (sku, event, payload) -> recording.emit(event, sku, payload));
```

## 1. Explicit checks in selected tests

[ExplicitInventoryTest](src/test/java/example/ExplicitInventoryTest.java) opens a
`PrsObserver` recording in try-with-resources before building the inventory; closing
it checks. The negative control asserts the structured `spec_violation` error.

## 2. Automatic checks with a JUnit extension

[AutomaticInventoryTest](src/test/java/example/AutomaticInventoryTest.java) registers
`PrsExtension`. The injected `Recording` opens before `@BeforeEach` stocks products
and checks after `@AfterEach` releases the fixture's reservation, so both are
recorded. [Coverage](../README.md#coverage) is written to
`target/coverage/model.json`.

## 3. Generated sequences with jqwik

[GeneratedInventoryTest](src/test/java/example/GeneratedInventoryTest.java) generates
60 sequences of 1–40 reserve/release operations, including invalid ones, and checks
them against a reference ledger as well as the spec.

**Gotcha: Jupiter callbacks don't wrap jqwik tries.** Create the inventory, ledger
and recording inside the `@Property` method so every try and shrink attempt is
independent. Coverage goes to a `CoverageCollector` closed in `@AfterContainer`,
written to `target/coverage/generated.json`.

jqwik prints its seed and smallest failing list, and keeps replay state in
`.jqwik-database`. Add `seed = "<reported seed>"` to `@Property` to reproduce one.

## Adapt to your project

Emit from the same domain interface your production telemetry or outbox uses. Set
spec files and names in `PrsObserver`/`PrsExtension`, and `PRS_BIN` to your `prs`
build. `prs-test-core` works without Jupiter. Failed recordings keep events, reports
and replay logs under `target/prs`. To combine the two suite reports:

```sh
"$PRS_BIN" coverage merge-observe target/coverage/model.json target/coverage/generated.json \
  --json-report target/coverage/suite.json
```

See the [helper documentation](../../java/README.md) for collector lifetimes,
parallel tests and forked JVMs.
