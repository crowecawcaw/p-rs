package example;

import io.github.crowecawcaw.snappea.SnappeaObserver;
import io.github.crowecawcaw.snappea.Recording;
import io.github.crowecawcaw.snappea.emit.SnappeaEmit;
import java.util.Map;
import java.util.concurrent.TimeUnit;
import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

class ExplicitInventoryTest {
    // Spec files and names come from snappea.properties.
    private static final SnappeaObserver OBSERVER = SnappeaObserver.builder().build();

    @Test void reserveAndRelease() {
        try (var recording = OBSERVER.recording("reserve and release")) {
            var inventory = new Inventory();
            inventory.addProduct("coffee", 3);
            inventory.reserve("coffee", "order-1", 2);
            assertEquals(1, inventory.available("coffee"));
            assertThrows(IllegalArgumentException.class, () -> inventory.reserve("coffee", "order-2", 2));
            inventory.release("coffee", "order-1");
            assertEquals(3, inventory.available("coffee"));
        }
    }

    // A negative control: the spec must reject an overselling trace.
    @Test void monitorRejectsOverselling() {
        var violation = Recording.expectViolation("InventoryCorrect", "exceeds available stock", () -> {
            try (var recording = OBSERVER.recording("expected oversell violation")) {
                new Inventory().addProduct("coffee", 1);
                SnappeaEmit.emit("eReserved", "coffee", Map.of("order", "order-1", "units", 2));
            }
        });
        assertEquals("coffee", violation.key());
    }

    // Events from another process: the worker inherits SNAPPEA_EVENTS_DIR and writes there.
    @Test void workerProcess() throws Exception {
        try (var recording = OBSERVER.recording("worker process")) {
            var java = ProcessHandle.current().info().command().orElse("java");
            var worker = new ProcessBuilder(java, "-cp", System.getProperty("java.class.path"),
                    Worker.class.getName(), "tea", "2", "order-7", "2").inheritIO();
            worker.environment().putAll(recording.environment());
            var process = worker.start();
            assertTrue(process.waitFor(60, TimeUnit.SECONDS)); // join children before the check
            assertEquals(0, process.exitValue());
        }
    }
}
