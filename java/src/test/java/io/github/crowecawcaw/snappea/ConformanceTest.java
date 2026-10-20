package io.github.crowecawcaw.snappea;

import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Instant;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.stream.Stream;
import org.junit.jupiter.api.DynamicTest;
import org.junit.jupiter.api.TestFactory;
import org.junit.jupiter.api.io.TempDir;
import static org.junit.jupiter.api.Assertions.*;

/** Replays tests/conformance through the Java L1/L2 API (see tests/conformance/README.md). */
class ConformanceTest {
    /** Features of the corpus this helper supports; a case requiring anything else is skipped. */
    private static final Set<String> FEATURES = Set.of("events_dir", "unknown_events");
    private static final Path CORPUS = Path.of("../tests/conformance");
    @TempDir Path temporary;

    @TestFactory Stream<DynamicTest> corpus() throws Exception {
        List<Path> cases;
        try (var dirs = Files.list(CORPUS)) {
            cases = dirs.filter(Files::isDirectory).sorted().toList();
        }
        assertFalse(cases.isEmpty(), "no conformance cases in " + CORPUS.toAbsolutePath());
        return cases.stream().map(dir -> DynamicTest.dynamicTest(dir.getFileName().toString(), () -> replay(dir)));
    }

    private void replay(Path dir) throws Exception {
        var testCase = Json.read(dir.resolve("expect.json"));
        for (var feature : Json.items(Json.get(testCase, "requires"))) {
            if (!FEATURES.contains(feature)) {
                System.out.println("skip " + dir.getFileName() + ": requires " + feature);
                org.junit.jupiter.api.Assumptions.abort("requires " + feature);
            }
        }
        var options = Json.get(testCase, "options");
        var expect = Json.get(testCase, "expect");
        var builder = SnappeaObserver.builder().config(SnappeaConfig.NONE).specFiles(dir.resolve("spec.p"))
                .artifacts(temporary.resolve("artifacts"));
        if (Json.has(options, "specs")) builder.specs(strings(Json.get(options, "specs")));
        if (Json.has(options, "exclude")) builder.exclude(strings(Json.get(options, "exclude")));
        if (Json.has(options, "strict")) builder.strict((Boolean) Json.get(options, "strict"));
        if (Json.has(options, "check_hot")) builder.checkHot((Boolean) Json.get(options, "check_hot"));
        if (Json.has(options, "unknown_events"))
            builder.unknownEvents(UnknownEvents.valueOf(Json.text(Json.get(options, "unknown_events")).toUpperCase()));
        var observer = builder.build();
        boolean wantsCoverage = Json.has(expect, "coverage");
        var coverage = observer.coverage(wantsCoverage ? temporary.resolve("coverage.json") : null);
        var recording = observer.recording(dir.getFileName().toString(), coverage);
        for (var line : Files.readAllLines(dir.resolve("events.jsonl"))) {
            if (line.isBlank()) continue;
            var event = Json.parse(line);
            var time = Json.get(event, "timestamp");
            var at = time instanceof Number n ? Instant.ofEpochMilli(n.longValue()) : Instant.parse((String) time);
            recording.emitAt(at, Json.text(Json.get(event, "event")), Json.text(Json.get(event, "key")),
                    Json.get(event, "payload"));
        }
        var children = dir.resolve("children");
        if (Files.isDirectory(children)) {
            try (var files = Files.list(children)) {
                for (var file : files.sorted().toList()) Files.copy(file, recording.eventsDir().resolve(file.getFileName()));
            }
        }
        String outcome = Json.text(Json.get(expect, "outcome"));
        Throwable caught = null;
        try {
            recording.close();
        } catch (SpecViolation | SetupError error) {
            caught = error;
        }
        final Throwable thrown = caught;
        switch (outcome) {
            case "pass" -> assertNull(thrown, () -> "expected a pass, got " + thrown);
            case "violation" -> {
                var violation = assertInstanceOf(SpecViolation.class, thrown, () -> "expected a violation, got " + thrown);
                assertEquals(Json.get(expect, "spec"), violation.spec());
                if (Json.has(expect, "key")) assertEquals(Json.get(expect, "key"), violation.key());
                assertTrue(violation.message().contains(Json.text(Json.get(expect, "message"))), violation.message());
                assertEquals(Boolean.TRUE.equals(Json.get(expect, "liveness")), violation.liveness());
            }
            case "setup_error" -> {
                var error = assertInstanceOf(SetupError.class, thrown, () -> "expected a setup error, got " + thrown);
                assertEquals(Json.get(expect, "kind"), error.kind(), error.getMessage());
                if (Json.has(expect, "message"))
                    assertTrue(error.getMessage().contains(Json.text(Json.get(expect, "message"))), error.getMessage());
            }
            default -> fail("unknown outcome " + outcome);
        }
        if (wantsCoverage) {
            var report = Json.parse(recording.report().orElseThrow().json());
            for (var spec : ((Map<?, ?>) Json.get(expect, "coverage")).entrySet()) {
                Object got = null;
                for (var each : Json.items(Json.get(report, "specs")))
                    if (spec.getKey().equals(Json.get(each, "spec"))) got = Json.get(each, "coverage");
                assertNotNull(got, "no coverage for " + spec.getKey());
                for (var part : List.of("states", "handlers"))
                    for (var count : List.of("covered", "total"))
                        if (Json.has(Json.get(spec.getValue(), part), count))
                            assertEquals(Json.get(Json.get(spec.getValue(), part), count), Json.get(Json.get(got, part), count),
                                    spec.getKey() + " " + part + "." + count);
            }
            coverage.close();
        }
    }

    private static String[] strings(Object values) {
        var out = new ArrayList<String>();
        Json.items(values).forEach(v -> out.add((String) v));
        return out.toArray(String[]::new);
    }
}
