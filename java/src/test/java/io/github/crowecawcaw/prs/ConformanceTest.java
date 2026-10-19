package io.github.crowecawcaw.prs;

import com.fasterxml.jackson.databind.JsonNode;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Instant;
import java.util.ArrayList;
import java.util.List;
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
        var testCase = Recording.JSON.readTree(dir.resolve("expect.json").toFile());
        for (var feature : testCase.path("requires")) {
            if (!FEATURES.contains(feature.asText())) {
                System.out.println("skip " + dir.getFileName() + ": requires " + feature.asText());
                org.junit.jupiter.api.Assumptions.abort("requires " + feature.asText());
            }
        }
        var options = testCase.path("options");
        var expect = testCase.path("expect");
        var builder = PrsObserver.builder().config(PrsConfig.NONE).specFiles(dir.resolve("spec.p"))
                .artifacts(temporary.resolve("artifacts"));
        if (options.has("specs")) builder.specs(strings(options.get("specs")));
        if (options.has("exclude")) builder.exclude(strings(options.get("exclude")));
        if (options.has("strict")) builder.strict(options.get("strict").asBoolean());
        if (options.has("check_hot")) builder.checkHot(options.get("check_hot").asBoolean());
        if (options.has("unknown_events"))
            builder.unknownEvents(UnknownEvents.valueOf(options.get("unknown_events").asText().toUpperCase()));
        var observer = builder.build();
        boolean wantsCoverage = expect.has("coverage");
        var coverage = observer.coverage(wantsCoverage ? temporary.resolve("coverage.json") : null);
        var recording = observer.recording(dir.getFileName().toString(), coverage);
        for (var line : Files.readAllLines(dir.resolve("events.jsonl"))) {
            if (line.isBlank()) continue;
            var event = Recording.JSON.readTree(line);
            var time = event.get("timestamp");
            var at = time.isNumber() ? Instant.ofEpochMilli(time.asLong()) : Instant.parse(time.asText());
            Object payload = event.has("payload") ? Recording.JSON.convertValue(event.get("payload"), Object.class) : null;
            recording.emitAt(at, event.path("event").asText(), event.path("key").asText(), payload);
        }
        var children = dir.resolve("children");
        if (Files.isDirectory(children)) {
            try (var files = Files.list(children)) {
                for (var file : files.sorted().toList()) Files.copy(file, recording.eventsDir().resolve(file.getFileName()));
            }
        }
        String outcome = expect.path("outcome").asText();
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
                assertEquals(expect.path("spec").asText(), violation.spec());
                if (expect.has("key")) assertEquals(expect.get("key").asText(), violation.key());
                assertTrue(violation.message().contains(expect.path("message").asText()), violation.message());
                assertEquals(expect.path("liveness").asBoolean(false), violation.liveness());
            }
            case "setup_error" -> {
                var error = assertInstanceOf(SetupError.class, thrown, () -> "expected a setup error, got " + thrown);
                assertEquals(expect.path("kind").asText(), error.kind(), error.getMessage());
                if (expect.has("message"))
                    assertTrue(error.getMessage().contains(expect.get("message").asText()), error.getMessage());
            }
            default -> fail("unknown outcome " + outcome);
        }
        if (wantsCoverage) {
            var report = Recording.JSON.readTree(recording.report().orElseThrow().json());
            var want = expect.get("coverage").fields();
            while (want.hasNext()) {
                var spec = want.next();
                JsonNode got = null;
                for (var each : report.path("specs")) if (each.path("spec").asText().equals(spec.getKey())) got = each.path("coverage");
                assertNotNull(got, "no coverage for " + spec.getKey());
                for (var part : List.of("states", "handlers"))
                    for (var count : List.of("covered", "total"))
                        if (spec.getValue().path(part).has(count))
                            assertEquals(spec.getValue().path(part).get(count).asInt(), got.path(part).path(count).asInt(),
                                    spec.getKey() + " " + part + "." + count);
            }
            coverage.close();
        }
    }

    private static String[] strings(JsonNode values) {
        var out = new ArrayList<String>();
        values.forEach(v -> out.add(v.asText()));
        return out.toArray(String[]::new);
    }
}
