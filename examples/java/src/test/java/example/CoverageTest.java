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
        var report = coverage.report();
        var handlers = report.path("specs").get(0).path("coverage").path("handlers");
        assertEquals(1, handlers.path("covered").asInt());
        assertEquals("InventoryCorrect.Unstocked", handlers.path("visited").get(0).path("state").asText());
        assertEquals("eReserved", handlers.path("visited").get(0).path("event").asText());
    }
}
