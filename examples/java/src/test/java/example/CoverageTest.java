package example;

import java.util.Map;
import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

class CoverageTest {
    @Test void failingObservationRetainsAttemptedHandler() throws Exception {
        // Local aggregate deliberately stays separate from the JUnit extension's
        // root resource: this adapter test must not inflate example suite coverage.
        var coverage = new Coverage();
        var recording = new Recording(coverage);
        recording.emit("book", "eReserved", Map.of("order", "alice", "units", 1));
        var failure = assertThrows(AssertionError.class, recording::close);
        assertTrue(failure.getMessage().contains("prs exit 1"), failure::getMessage);
        assertTrue(failure.getMessage().contains("\"exit_code\": 2"), failure::getMessage);
        // The same attempted handler in another observation counts only once.
        var second = new Recording(coverage);
        second.emit("book", "eReserved", Map.of("order", "bob", "units", 1));
        assertThrows(AssertionError.class, second::close);
        var report = coverage.write(java.nio.file.Files.createTempDirectory("coverage-test-").resolve("model.json"));
        var handlers = report.path("specs").get(0).path("coverage").path("handlers");
        assertEquals(1, handlers.path("covered").asInt());
        assertEquals(2, report.path("observations").size());
        assertEquals("InventoryCorrect.Unstocked", handlers.path("visited").get(0).path("state").asText());
        assertEquals("eReserved", handlers.path("visited").get(0).path("event").asText());
    }
    @Test void reportWriteFailureIsNotHidden() throws Exception {
        var directory = java.nio.file.Files.createTempDirectory("coverage-unwritable-");
        assertThrows(AssertionError.class, () -> new Coverage().write(directory));
    }
}
