package example;

import java.util.Map;
import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

class ExplicitInventoryTest {
    @Test void reserveAndRelease() throws Exception {
        try (var recording = new Recording()) {
            var inventory = new Inventory(recording);
            inventory.addProduct("book", 5);
            inventory.reserve("book", "alice", 3);
            assertEquals(2, inventory.available("book"));
            assertThrows(IllegalArgumentException.class, () -> inventory.reserve("book", "bob", 3));
            inventory.release("book", "alice");
            assertEquals(5, inventory.available("book"));
        }
    }

    @Test void monitorRejectsOverselling() throws Exception {
        var recording = new Recording();
        var inventory = new Inventory(recording);
        inventory.addProduct("book", 1);
        // Simulate a broken commit boundary. The test passes only if P detects it.
        recording.emit("book", "eReserved", Map.of("order", "alice", "units", 2));
        var failure = assertThrows(AssertionError.class, recording::close);
        assertTrue(failure.getMessage().contains("reservation exceeds available stock"), failure::getMessage);
        // prs maps PObserve's exit 2 (also retained in the report) to its BUG exit 1.
        assertTrue(failure.getMessage().contains("prs exit 1"), failure::getMessage);
        assertTrue(failure.getMessage().contains("\"exit_code\": 2"), failure::getMessage);
        assertTrue(failure.getMessage().contains("\"kind\": \"spec_violation\""), failure::getMessage);
    }
}
