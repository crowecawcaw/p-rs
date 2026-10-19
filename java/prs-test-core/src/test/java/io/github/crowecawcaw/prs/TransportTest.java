package io.github.crowecawcaw.prs;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Duration;
import java.util.List;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.condition.EnabledOnOs;
import org.junit.jupiter.api.condition.OS;
import org.junit.jupiter.api.io.TempDir;
import static org.junit.jupiter.api.Assertions.*;

class TransportTest {
    @TempDir Path temporary;

    private Path executable(String body) throws IOException {
        var path = temporary.resolve("fake prs");
        Files.writeString(path, "#!/bin/sh\n" + body);
        assertTrue(path.toFile().setExecutable(true));
        return path;
    }

    private Recording recording(Path executable) {
        return PrsObserver.builder().binary(executable.toString()).specFiles(Path.of("spec.p"))
                .specs("Spec").artifacts(temporary.resolve("artifacts")).build().recording("fake transport");
    }

    @Test @EnabledOnOs(OS.LINUX)
    void missingAndMalformedSuccessReportsFailClosed() throws Exception {
        for (String json : List.of("", "{}", "{\"exit_code\":0}", "{\"exit_code\":0,\"specs\":[]}",
                "{\"exit_code\":0,\"specs\":[{\"spec\":\"Spec\"}]}",
                "{\"exit_code\":0,\"specs\":[{\"spec\":\"Wrong\",\"errors\":[]}]}",
                "{\"exit_code\":0,\"specs\":[{\"spec\":\"Spec\",\"errors\":null}]}",
                "{\"exit_code\":\"0\",\"specs\":[{\"spec\":\"Spec\",\"errors\":[]}]}")) {
            var binary = executable("report=''\nwhile [ \"$#\" -gt 0 ]; do\n"
                    + "  if [ \"$1\" = '--json-report' ]; then shift; report=$1; fi\n  shift\ndone\n"
                    + "printf '%s' '" + json + "' > \"$report\"\nexit 0\n");
            var events = recording(binary);
            events.emit("event", "key");
            var failure = assertThrows(ObservationFailure.class, events::close, json);
            assertTrue(Files.exists(failure.artifacts().resolve("events.jsonl")));
        }
        var events = recording(executable("exit 0\n"));
        assertThrows(ObservationFailure.class, events::close);
    }

    @Test @EnabledOnOs(OS.LINUX)
    void invalidMetricStatusAndErrorTypesCannotMasqueradeAsSuccess() throws Exception {
        var valid = Recording.JSON.readTree("{\"exit_code\":0,\"specs\":[{\"spec\":\"Spec\","
                + "\"events_read\":1,\"partition_keys\":1,\"verified_keys\":1,\"errors\":[]}]}");
        for (String mutation : List.of("large status", "unknown status", "missing metric", "negative metric",
                "fractional metric", "too many verified", "unknown kind", "missing message", "process mismatch")) {
            var value = (com.fasterxml.jackson.databind.node.ObjectNode) valid.deepCopy();
            var spec = (com.fasterxml.jackson.databind.node.ObjectNode) value.path("specs").get(0);
            int exit = 0;
            switch (mutation) {
                case "large status" -> value.put("exit_code", 4_294_967_296L);
                case "unknown status" -> value.put("exit_code", 17);
                case "missing metric" -> spec.remove("events_read");
                case "negative metric" -> spec.put("events_read", -1);
                case "fractional metric" -> spec.put("events_read", 1.5);
                case "too many verified" -> spec.put("verified_keys", 2);
                case "unknown kind" -> spec.putArray("errors").addObject().put("kind", "typo").put("message", "bad");
                case "missing message" -> spec.putArray("errors").addObject().put("kind", "parser");
                case "process mismatch" -> exit = 2;
            }
            var json = value.toString();
            var binary = executable("report=''\nwhile [ \"$#\" -gt 0 ]; do\n"
                    + "  if [ \"$1\" = '--json-report' ]; then shift; report=$1; fi\n  shift\ndone\n"
                    + "printf '%s' '" + json + "' > \"$report\"\nexit " + exit + "\n");
            var events = recording(binary);
            events.emit("event", "key");
            var failure = assertThrows(ObservationFailure.class, events::close, mutation);
            assertTrue(failure.report().isEmpty(), "Invalid report must not be collected as coverage: " + mutation);
        }
    }

    @Test @EnabledOnOs(OS.LINUX)
    void timeoutTerminatesWrapperAndItsChildWithinBound() throws Exception {
        var pid = temporary.resolve("child.pid");
        var binary = executable("sleep 60 &\nchild=$!\nprintf '%s' \"$child\" > '" + pid + "'\nwait\n");
        long began = System.nanoTime();
        var error = assertThrows(IOException.class, () -> Processes.run(List.of(binary.toString()), null,
                temporary.resolve("console.txt"), Duration.ofMillis(200)));
        assertTrue(error.getMessage().contains("timeout"));
        assertTrue(Duration.ofNanos(System.nanoTime() - began).compareTo(Duration.ofSeconds(5)) < 0);
        long child = Long.parseLong(Files.readString(pid));
        // A killed child can briefly remain a zombie until init reaps it; a zombie
        // has no executable or open event stream and is already terminated.
        var stat = Path.of("/proc", Long.toString(child), "stat");
        assertTrue(ProcessHandle.of(child).isEmpty() || !ProcessHandle.of(child).orElseThrow().isAlive()
                || (Files.exists(stat) && Files.readString(stat).split(" ")[2].equals("Z")));
    }

    @Test @EnabledOnOs(OS.LINUX)
    void interruptionTerminatesProcessAndRestoresInterruptFlag() throws Exception {
        var binary = executable("sleep 60\n");
        var flag = new java.util.concurrent.atomic.AtomicBoolean();
        var failure = new java.util.concurrent.atomic.AtomicReference<Throwable>();
        var worker = new Thread(() -> {
            try {
                Processes.run(List.of(binary.toString()), null, temporary.resolve("interrupted.txt"), Duration.ofSeconds(30));
                failure.set(new AssertionError("process was not interrupted"));
            } catch (IOException expected) {
                flag.set(Thread.currentThread().isInterrupted());
            }
        });
        worker.start();
        worker.interrupt();
        worker.join(5000);
        assertFalse(worker.isAlive());
        assertNull(failure.get());
        assertTrue(flag.get());
    }
}
