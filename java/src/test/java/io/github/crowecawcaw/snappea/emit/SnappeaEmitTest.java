package io.github.crowecawcaw.snappea.emit;

import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Duration;
import java.time.Instant;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;
import static org.junit.jupiter.api.Assertions.*;

class SnappeaEmitTest {
    @TempDir Path temporary;

    enum Color { RED }
    record Reservation(String order, int units) {}

    // Recordings from other test classes in this JVM leave their router installed.
    @BeforeEach void noRecordings() { EmitHook.install(null); }

    @AfterEach void reset() { SnappeaEmit.close(); }

    @Test void offByDefaultAndNeverThrows() {
        assertFalse(SnappeaEmit.enabled());
        SnappeaEmit.emit("eAnything", "k", new Object()); // not encoded at all when off
    }

    @Test void payloadsEncodeAsJson() {
        var map = new LinkedHashMap<String, Object>();
        map.put("order", "o\"1\n");
        map.put("units", 2);
        map.put("tags", Set.of(Color.RED));
        map.put("pair", new int[] {1, 2});
        map.put("nothing", null);
        String line = EmitHook.line(Instant.parse("2026-01-01T00:00:00.000000001Z"), "eReserved", "p1", map, "a/B.java:3");
        assertTrue(line.startsWith("{\"key\":\"p1\",\"timestamp\":\"2026-01-01T00:00:00.000000001Z\",\"event\":\"eReserved\","
                + "\"payload\":{\"order\":\"o\\\"1\\n\",\"units\":2,\"tags\":[\"RED\"],\"pair\":[1,2],\"nothing\":null},"
                + "\"src\":\"a/B.java:3\",\"seq\":"), line);
        assertTrue(line.endsWith(",\"at\":true}"), line);
        assertTrue(EmitHook.line(null, "e", null, new Reservation("o", 1), null)
                .contains("\"key\":\"\",\"timestamp\":"));
        assertTrue(EmitHook.line(null, "e", "k", new Reservation("o", 1), null).contains("\"payload\":{\"order\":\"o\",\"units\":1}"));
        assertTrue(EmitHook.line(null, "e", "k", 7, null).contains("\"payload\":7,"));
        assertThrows(IllegalArgumentException.class, () -> EmitHook.line(null, "e", "k", new Object(), null));
        assertThrows(IllegalArgumentException.class, () -> EmitHook.line(null, "e", "k", Double.NaN, null));
        var cycle = new ArrayList<Object>();
        cycle.add(cycle);
        assertThrows(IllegalArgumentException.class, () -> EmitHook.line(null, "e", "k", cycle, null));
    }

    @Test void wallClockNeverDecreasesAndSeqIncreases() {
        String a = EmitHook.line(null, "e", "k", null, null);
        String b = EmitHook.line(null, "e", "k", null, null);
        assertTrue(field(a, "timestamp").compareTo(field(b, "timestamp")) <= 0);
        assertTrue(Long.parseLong(field(a, "seq")) < Long.parseLong(field(b, "seq")));
    }

    @Test void callSiteNamesTheCaller() {
        assertTrue(EmitHook.callSite().startsWith("io/github/crowecawcaw/snappea/emit/SnappeaEmitTest.java:"), EmitHook.callSite());
    }

    @Test void fileSinkWritesLinesAndCountsBadPayloads() throws Exception {
        var file = temporary.resolve("logs/events.jsonl");
        SnappeaEmit.setSink(Sink.file(file));
        assertTrue(SnappeaEmit.enabled());
        long dropped = SnappeaEmit.dropped();
        for (int i = 0; i < 100; i++) SnappeaEmit.emit("eStocked", "p" + i, Map.of("units", i));
        SnappeaEmit.emit("eStocked", "bad", new Object()); // dropped, not thrown
        assertTrue(SnappeaEmit.flush(Duration.ofSeconds(5)));
        List<String> lines = Files.readAllLines(file);
        assertEquals(100, lines.size());
        assertTrue(lines.get(0).contains("\"event\":\"eStocked\""));
        assertFalse(lines.get(0).contains("\"src\""), "production lines skip the stack walk");
        assertEquals(dropped + 1, SnappeaEmit.dropped());
    }

    @Test void callbackSinkSwallowsCallbackErrors() {
        var seen = new java.util.concurrent.CopyOnWriteArrayList<String>();
        SnappeaEmit.setSink(Sink.callback(seen::add));
        SnappeaEmit.emitAt(Instant.ofEpochSecond(0, 5), "eTick", "clock");
        assertTrue(SnappeaEmit.flush(Duration.ofSeconds(5)));
        assertEquals(1, seen.size());
        assertTrue(seen.get(0).contains("\"timestamp\":\"1970-01-01T00:00:00.000000005Z\""));
        assertFalse(seen.get(0).endsWith("\n"));
        long dropped = SnappeaEmit.dropped();
        SnappeaEmit.setSink(Sink.callback(line -> { throw new IllegalStateException("broken"); }));
        SnappeaEmit.emit("eTick", "clock");
        assertTrue(SnappeaEmit.flush(Duration.ofSeconds(5)));
        assertEquals(dropped + 1, SnappeaEmit.dropped());
    }

    @Test void writerSinkFlushesButDoesNotCloseTheWriter() {
        var out = new java.io.StringWriter() {
            boolean closed;
            @Override public void close() { closed = true; }
        };
        SnappeaEmit.setSink(Sink.writer(out));
        SnappeaEmit.emit("eTick", "clock");
        SnappeaEmit.close();
        assertTrue(out.toString().endsWith("}\n"), out.toString());
        assertFalse(out.closed);
        assertFalse(SnappeaEmit.enabled());
    }

    @Test void folderSinkWritesOneFilePerProcess() throws Exception {
        var sink = new Sinks.FolderSink(temporary.resolve("events"), "worker");
        sink.offer("{\"a\":1}");
        sink.offer("{\"a\":2}");
        try (var files = Files.list(temporary.resolve("events"))) {
            var all = files.toList();
            assertEquals(1, all.size());
            var name = all.get(0).getFileName().toString();
            assertTrue(name.matches("worker-" + ProcessHandle.current().pid() + "-[0-9a-f]+\\.jsonl"), name);
            assertEquals(List.of("{\"a\":1}", "{\"a\":2}"), Files.readAllLines(all.get(0)));
        }
    }

    private static String field(String line, String name) {
        int start = line.indexOf("\"" + name + "\":") + name.length() + 3;
        int end = start;
        while (end < line.length() && line.charAt(end) != ',' && line.charAt(end) != '}') end++;
        return line.substring(start, end).replace("\"", "");
    }
}
