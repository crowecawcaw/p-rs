package example;

import java.util.HashMap;
import java.util.List;
import java.util.Map;
import net.jqwik.api.*;
import static org.junit.jupiter.api.Assertions.*;

class GeneratedInventoryTest {
    record Operation(boolean release, String sku, String order, int units) {}

    @Provide Arbitrary<List<Operation>> sequences() {
        var operation = Combinators.combine(Arbitraries.of(true, false), Arbitraries.of("book", "pen"),
                Arbitraries.of("alice", "bob", "carol"), Arbitraries.integers().between(-1, 7))
                .as(Operation::new);
        return operation.list().ofMinSize(1).ofMaxSize(40);
    }

    @Property(tries = 60) void operationSequences(@ForAll("sequences") List<Operation> operations) throws Exception {
        // jqwik invokes this method for EVERY candidate and shrink attempt. Jupiter's
        // @BeforeEach/extension does not bracket a jqwik property invocation.
        try (var recording = new Recording()) {
            var inventory = new Inventory(recording);
            inventory.addProduct("book", 5);
            inventory.addProduct("pen", 3);
            var available = new HashMap<>(Map.of("book", 5, "pen", 3));
            var reservations = new HashMap<String, Integer>();
            for (var op : operations) {
                String key = op.sku() + ":" + op.order();
                if (op.release()) {
                    Integer units = reservations.get(key);
                    if (units == null) {
                        assertThrows(IllegalArgumentException.class, () -> inventory.release(op.sku(), op.order()));
                    } else {
                        inventory.release(op.sku(), op.order());
                        reservations.remove(key);
                        available.merge(op.sku(), units, Integer::sum);
                    }
                } else if (op.units() <= 0 || reservations.containsKey(key) || op.units() > available.get(op.sku())) {
                    assertThrows(IllegalArgumentException.class, () -> inventory.reserve(op.sku(), op.order(), op.units()));
                } else {
                    inventory.reserve(op.sku(), op.order(), op.units());
                    reservations.put(key, op.units());
                    available.merge(op.sku(), -op.units(), Integer::sum);
                }
                for (var sku : available.keySet()) assertEquals(available.get(sku).intValue(), inventory.available(sku));
            }
        }
    }
}
