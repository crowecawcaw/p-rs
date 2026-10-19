package example;

import java.util.HashMap;
import java.util.List;
import java.util.Map;
import net.jqwik.api.*;
import net.jqwik.api.lifecycle.AfterContainer;
import java.nio.file.Path;
import io.github.crowecawcaw.prs.PrsObserver;
import io.github.crowecawcaw.prs.CoverageCollector;
import static org.junit.jupiter.api.Assertions.*;

class GeneratedInventoryTest {
    private static final PrsObserver OBSERVER = PrsObserver.builder()
            .specFiles(Path.of("specs/inventory.p")).specs("InventoryCorrect").build();
    private static final CoverageCollector COVERAGE = OBSERVER.coverage(Path.of("target/coverage/generated.json"));

    @AfterContainer static void writeCoverage() { COVERAGE.close(); }

    record Operation(boolean release, String sku, String order, int units) {}

    @Provide Arbitrary<List<Operation>> sequences() {
        var operation = Combinators.combine(Arbitraries.of(true, false), Arbitraries.of("coffee", "tea"),
                Arbitraries.of("order-1", "order-2", "order-3"), Arbitraries.integers().between(-1, 7))
                .as(Operation::new);
        return operation.list().ofMinSize(1).ofMaxSize(40);
    }

    @Property(tries = 60) void operationSequences(@ForAll("sequences") List<Operation> operations) throws Exception {
        // Runs once per try and shrink attempt; Jupiter fixtures do not wrap jqwik tries.
        try (var recording = OBSERVER.recording("jqwik inventory sequence", COVERAGE)) {
            var inventory = new Inventory((sku, event, payload) -> recording.emit(event, sku, payload));
            inventory.addProduct("coffee", 3);
            inventory.addProduct("tea", 2);
            var available = new HashMap<>(Map.of("coffee", 3, "tea", 2));
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
