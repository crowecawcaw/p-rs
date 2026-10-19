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
            inventory.addProduct("coffee", 3);
            inventory.reserve("coffee", "order-1", 2);
            assertEquals(1, inventory.available("coffee"));
            assertThrows(IllegalArgumentException.class, () -> inventory.reserve("coffee", "order-2", 2));
            inventory.release("coffee", "order-1");
            assertEquals(3, inventory.available("coffee"));
        }
    }

    // A negative control: the spec must reject an overselling trace.
    @Test void monitorRejectsOverselling() throws Exception {
        var recording = OBSERVER.recording("expected oversell violation");
        var inventory = new Inventory((sku, event, payload) -> recording.emit(event, sku, payload));
        inventory.addProduct("coffee", 1);
        recording.emit("eReserved", "coffee", Map.of("order", "order-1", "units", 2));
        var failure = assertThrows(ObservationFailure.class, recording::close);
        var report = failure.report().orElseThrow();
        assertEquals(2, report.exitCode());
        assertEquals("spec_violation", report.errors().get(0).kind());
        assertTrue(report.errors().get(0).message().contains("reservation exceeds available stock"));
    }
}
