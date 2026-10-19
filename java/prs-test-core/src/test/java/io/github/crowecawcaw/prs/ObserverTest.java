package io.github.crowecawcaw.prs;

import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Duration;
import java.time.Instant;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.Executors;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;
import static org.junit.jupiter.api.Assertions.*;

class ObserverTest {
    @TempDir Path temporary;
    private Path spec() { return Path.of("src/test/resources/spec.p"); }
    private PrsObserver.Builder builder(String... specs) {
        return PrsObserver.builder().specFiles(spec()).specs(specs).artifacts(temporary.resolve("artifacts with spaces"));
    }

    @Test void independentRecordingsAndRelevantSpecSubsets() {
        var observer = builder("Counter", "Other").build();
        for (int i = 0; i < 2; i++) {
            try (var events = observer.recording("case " + i)) {
                events.emit("eAdd", "counter", 3);
                events.emit("eTake", "counter", 2);
                events.emit("eOther", "other");
            }
        }
    }

    @Test void strictRejectsEmptyAndIrrelevantButCanBeDisabled() {
        var empty = builder("Counter").build().recording("empty");
        var failure = assertThrows(ObservationFailure.class, empty::close);
        assertEquals("validation", failure.report().orElseThrow().errors().get(0).kind());
        assertTrue(Files.exists(failure.artifacts().resolve("report.json")));
        var irrelevant = builder("Counter", "Other").build().recording("missing other");
        irrelevant.emit("eAdd", "counter", 1);
        assertThrows(ObservationFailure.class, irrelevant::close);
        try (var ignored = builder("Counter").strict(false).build().recording("explicit empty opt out")) {}
    }

    @Test void structuredFailuresPreserveReplayAndSource() throws Exception {
        var recording = builder("Counter").build().recording("overspend");
        recording.emit("eAdd", "sku", 1);
        recording.emit("eTake", "sku", 2);
        var failure = assertThrows(ObservationFailure.class, recording::close);
        var report = failure.report().orElseThrow();
        assertEquals(2, report.exitCode());
        var error = report.errors().get(0);
        assertEquals("spec_violation", error.kind());
        assertEquals("sku", error.key());
        assertTrue(error.message().contains("counter below zero"));
        assertEquals("eTake", error.events().get(error.events().size() - 1).event());
        assertEquals("overspend", Recording.JSON.readTree(report.json()).path("source").path("label").asText());
        assertTrue(Files.exists(failure.artifacts().resolve("events.jsonl")));
        assertDoesNotThrow(recording::close);
        assertThrows(IllegalStateException.class, () -> recording.emit("eOther", "late"));
    }

    @Test void mutablePayloadIsSnapshottedAndEmissionErrorsStaySticky() {
        var payload = new HashMap<String, Object>(Map.of("units", 1));
        try (var events = builder("Snapshot").build().recording("snapshot")) {
            events.emit("eTuple", "sku", payload);
            payload.put("units", -1);
        }
        var events = builder("Counter").build().recording("bad payload");
        events.emit("eAdd", "sku", 1);
        assertThrows(ObservationFailure.class, () -> events.emit("eAdd", "sku", new Object()));
        var failure = assertThrows(ObservationFailure.class, events::close);
        assertNotNull(failure.getCause());
        assertTrue(failure.getMessage().contains("Cannot emit"));
    }

    @Test void tryWithResourcesPreservesBodyFailure() {
        var primary = assertThrows(IllegalStateException.class, () -> {
            try (var events = builder("Counter").build().recording("body and observation fail")) {
                events.emit("eTake", "sku", 1);
                throw new IllegalStateException("body failed");
            }
        });
        assertEquals("body failed", primary.getMessage());
        assertInstanceOf(ObservationFailure.class, primary.getSuppressed()[0]);
    }

    @Test void actualTimestampExplicitPrecisionAndConcurrentCapture() throws Exception {
        var recording = builder("Counter").build().recording("parallel emitters");
        var pool = Executors.newFixedThreadPool(4);
        try {
            var tasks = new java.util.ArrayList<java.util.concurrent.Callable<Void>>();
            for (int i = 0; i < 100; i++) tasks.add(() -> { recording.emit("eAdd", "sku", 1); return null; });
            for (var future : pool.invokeAll(tasks)) future.get();
        } finally { pool.shutdownNow(); }
        recording.emit("eTake", "sku", 101); // Retain the log for timestamp inspection.
        var failure = assertThrows(ObservationFailure.class, recording::close);
        var lines = Files.readAllLines(failure.artifacts().resolve("events.jsonl"));
        assertEquals(101, lines.size());
        assertTrue(Recording.JSON.readTree(lines.get(0)).path("timestamp").asLong() > 1_700_000_000_000L);
        var exact = builder("Counter").build().recording("timestamp");
        var stamp = Instant.parse("2026-10-19T00:00:00.123456789Z");
        exact.emitAt(stamp, "eTake", "sku", 1);
        var exactFailure = assertThrows(ObservationFailure.class, exact::close);
        assertEquals(stamp.toString(), Recording.JSON.readTree(Files.readString(
                exactFailure.artifacts().resolve("events.jsonl"))).path("timestamp").asText());
    }

    @Test void checkHotIsIndependentOfStrict() {
        try (var events = builder("Completion").build().recording("unfinished permitted")) {
            events.emit("eBegin", "flow");
        }
        var events = builder("Completion").strict(false).checkHot(true).build().recording("unfinished rejected");
        events.emit("eBegin", "flow");
        assertThrows(ObservationFailure.class, events::close);
    }

    @Test void parserAndMissingSpecFailuresNeverPass() {
        var wrongPayload = builder("Counter").build().recording("parser error");
        wrongPayload.emit("eAdd", "sku", "not an integer");
        var failure = assertThrows(ObservationFailure.class, wrongPayload::close);
        assertEquals("parser", failure.report().orElseThrow().errors().get(0).kind());
        var missing = builder("Typo").build().recording("missing spec");
        missing.emit("eOther", "sku");
        assertThrows(ObservationFailure.class, missing::close);
    }

    @Test void missingBinaryAndInvalidConfigurationFailClearly() {
        var events = builder("Counter").binary(temporary.resolve("does not exist").toString()).build().recording("binary");
        events.emit("eAdd", "sku", 1);
        var failure = assertThrows(ObservationFailure.class, events::close);
        assertTrue(failure.getMessage().contains("Cannot check recording binary"));
        assertThrows(IllegalArgumentException.class, () -> builder("Counter").timeout(Duration.ZERO).build());
        assertThrows(IllegalArgumentException.class, () -> PrsObserver.builder().specFiles(spec()).build());
    }

    @Test void suiteUnionIncludesFailuresAndRetainsDistinctSources() throws Exception {
        var observer = builder("Counter").build();
        var target = temporary.resolve("coverage/monitor.json");
        try (var coverage = observer.coverage(target)) {
            try (var events = observer.recording("successful", coverage)) { events.emit("eAdd", "sku", 2); }
            var failed = observer.recording("failed", coverage);
            failed.emit("eTake", "sku", 1);
            assertThrows(ObservationFailure.class, failed::close);
        }
        var merged = Recording.JSON.readTree(target.toFile());
        assertEquals("observed_monitor_suite", merged.path("scope").asText());
        assertEquals(2, merged.path("observations").size());
        assertTrue(merged.path("observations").toString().contains("spec_violation"));
        assertEquals(2, merged.path("specs").get(0).path("coverage").path("handlers").path("covered").asInt());
    }

    @Test void incompatibleSourceMergePreservesExistingDestination() throws Exception {
        var sourceCopy = temporary.resolve("changed.p");
        Files.writeString(sourceCopy, Files.readString(spec()) + "\n// different compilation sources\n");
        var first = builder("Counter").build();
        var changed = PrsObserver.builder().specFiles(sourceCopy).specs("Counter")
                .artifacts(temporary.resolve("artifacts with spaces")).build();
        var target = temporary.resolve("coverage.json");
        Files.writeString(target, "previous destination");
        var coverage = first.coverage(target);
        try (var events = first.recording("first", coverage)) { events.emit("eAdd", "sku", 1); }
        try (var events = changed.recording("changed", coverage)) { events.emit("eAdd", "sku", 1); }
        var failure = assertThrows(ObservationFailure.class, coverage::close);
        assertTrue(failure.getMessage().contains("fingerprint"));
        assertEquals("previous destination", Files.readString(target));
    }
}
