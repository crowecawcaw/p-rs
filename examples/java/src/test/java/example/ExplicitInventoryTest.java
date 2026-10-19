package example;

import java.util.Map;
import java.nio.file.Path;
import io.github.crowecawcaw.prs.PrsObserver;
import io.github.crowecawcaw.prs.ObservationFailure;
import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

class ExplicitInventoryTest {
    private static final PrsObserver OBSERVER = PrsObserver.builder()
            .specFiles(Path.of("specs/inventory.p")).specs("InventoryCorrect").build();
    @Test void reserveAndRelease() throws Exception {
        try (var recording = OBSERVER.recording("reserve and release")) {
            var inventory = new Inventory((sku, event, payload) -> recording.emit(event, sku, payload));
            inventory.addProduct("book", 5);
            inventory.reserve("book", "alice", 3);
            assertEquals(2, inventory.available("book"));
            assertThrows(IllegalArgumentException.class, () -> inventory.reserve("book", "bob", 3));
            inventory.release("book", "alice");
            assertEquals(5, inventory.available("book"));
        }
    }

    @Test void monitorRejectsOverselling() throws Exception {
        var recording = OBSERVER.recording("expected oversell violation");
        var inventory = new Inventory((sku, event, payload) -> recording.emit(event, sku, payload));
        inventory.addProduct("book", 1);
        // Simulate a broken commit boundary. The test passes only if P detects it.
        recording.emit("eReserved", "book", Map.of("order", "alice", "units", 2));
        var failure = assertThrows(ObservationFailure.class, recording::close);
        var report = failure.report().orElseThrow();
        assertEquals(2, report.exitCode());
        assertEquals("spec_violation", report.errors().get(0).kind());
        assertTrue(report.errors().get(0).message().contains("reservation exceeds available stock"));
    }
}
