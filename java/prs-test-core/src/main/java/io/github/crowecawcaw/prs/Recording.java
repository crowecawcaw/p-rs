package io.github.crowecawcaw.prs;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import java.io.BufferedWriter;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Instant;
import java.util.IdentityHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;

/** One independent observation. Close after fixtures and application cleanup have finished. */
public final class Recording implements AutoCloseable {
    static final ObjectMapper JSON = new ObjectMapper();
    private final PrsObserver observer;
    private final CoverageCollector coverage;
    private final String label;
    private final Path directory;
    private final BufferedWriter writer;
    private Throwable emissionError;
    private ObservationReport report;
    private boolean closed;

    Recording(PrsObserver observer, String label, CoverageCollector coverage) {
        this.observer = observer;
        this.label = PrsObserver.nonempty(label, "label");
        this.coverage = coverage;
        Path created = observer.artifacts();
        try {
            Files.createDirectories(created);
            created = Files.createTempDirectory(created, "recording-");
            directory = created;
            writer = Files.newBufferedWriter(directory.resolve("events.jsonl"));
        } catch (IOException error) {
            throw new ObservationFailure("Cannot start recording " + label, created, null, error);
        }
    }

    /** Timestamped inside the same lock that serializes and appends the event. */
    public synchronized void emit(String event, String key, Object payload) {
        append(event, key, payload, System.currentTimeMillis());
    }

    public void emit(String event, String key) { emit(event, key, null); }

    /** Preserve an application's timestamp, including sub-millisecond precision. */
    public synchronized void emitAt(Instant timestamp, String event, String key, Object payload) {
        append(event, key, payload, timestamp == null ? null : timestamp.toString());
    }

    public void emitAt(Instant timestamp, String event, String key) { emitAt(timestamp, event, key, null); }

    private void append(String event, String key, Object payload, Object timestamp) {
        if (closed) throw new IllegalStateException("Recording is closed: " + label);
        if (emissionError != null) throw emissionFailure();
        try {
            PrsObserver.nonempty(event, "event");
            PrsObserver.nonempty(key, "key");
            if (timestamp == null) throw new IllegalArgumentException("timestamp must not be null");
            validateJson(payload, new IdentityHashMap<>(), 0);
            var value = JSON.createObjectNode();
            value.put("event", event);
            value.put("key", key);
            value.set("timestamp", JSON.valueToTree(timestamp));
            value.set("payload", JSON.valueToTree(payload));
            // Serialize the entire snapshot before appending; a serialization error
            // cannot leave half a line. No caller-owned mutable payload is retained.
            String line = JSON.writeValueAsString(value);
            writer.write(line);
            writer.newLine();
        } catch (IOException | RuntimeException error) {
            emissionError = error;
            throw emissionFailure();
        }
    }

    private ObservationFailure emissionFailure() {
        return new ObservationFailure("Cannot emit event for " + label, directory, report, emissionError);
    }

    private static void validateJson(Object value, IdentityHashMap<Object, Boolean> active, int depth) {
        if (depth > 256) throw new IllegalArgumentException("JSON payload nesting exceeds 256");
        if (value == null || value instanceof String || value instanceof Boolean) return;
        if (value instanceof Byte || value instanceof Short || value instanceof Integer || value instanceof Long
                || value instanceof java.math.BigInteger || value instanceof java.math.BigDecimal) return;
        if (value instanceof Double d && Double.isFinite(d)) return;
        if (value instanceof Float f && Float.isFinite(f)) return;
        if (active.put(value, Boolean.TRUE) != null) throw new IllegalArgumentException("Cyclic JSON payload");
        try {
            if (value instanceof Map<?, ?> map) {
                for (var entry : map.entrySet()) {
                    if (!(entry.getKey() instanceof String)) throw new IllegalArgumentException("JSON keys must be strings");
                    validateJson(entry.getValue(), active, depth + 1);
                }
            } else if (value instanceof List<?> list) {
                for (var entry : list) validateJson(entry, active, depth + 1);
            } else {
                throw new IllegalArgumentException("Expected JSON-compatible payload, got " + value.getClass().getName());
            }
        } finally {
            active.remove(value);
        }
    }

    public Path artifacts() { return directory; }
    public synchronized Optional<ObservationReport> report() { return Optional.ofNullable(report); }

    /** Idempotent; an emission error is sticky even if its immediate exception was caught. */
    @Override public synchronized void close() {
        if (closed) return;
        closed = true;
        try {
            writer.close();
        } catch (IOException error) {
            if (emissionError == null) emissionError = error;
            else emissionError.addSuppressed(error);
        }
        var reportPath = directory.resolve("report.json");
        var console = directory.resolve("console.txt");
        ObservationFailure observationError = null;
        try {
            int exit = Processes.run(observer.command(directory.resolve("events.jsonl"),
                    directory.resolve("observe"), reportPath, coverage != null), null, console, observer.timeout());
            JsonNode value = JSON.readTree(reportPath.toFile());
            validateReport(value, exit);
            var full = (com.fasterxml.jackson.databind.node.ObjectNode) value;
            var source = full.putObject("source");
            source.put("label", label);
            source.put("recording_id", directory.getFileName().toString());
            if (emissionError != null) {
                source.put("recording_error", emissionError.toString());
                full.put("input_complete", false);
                full.put("stop_reason", "recording_error");
            }
            JSON.writerWithDefaultPrettyPrinter().writeValue(reportPath.toFile(), full);
            report = new ObservationReport(full);
            if (exit != 0 || report.exitCode() != 0 || !report.errors().isEmpty()) {
                observationError = new ObservationFailure("Observation failed for " + label + ": prs exit " + exit
                        + "\n" + Processes.tail(console) + "\n" + report.json(), directory, report, null);
            }
        } catch (IOException | RuntimeException error) {
            observationError = new ObservationFailure("Cannot check recording " + label + "\n" + Processes.tail(console),
                    directory, report, error);
        }
        // Coverage retains failed prefixes too. Aggregation errors must not replace a
        // monitor/transport failure; recording errors likewise remain the primary cause.
        if (coverage != null && report != null) {
            try { coverage.add(report); }
            catch (ObservationFailure error) {
                if (observationError == null) observationError = error;
                else observationError.addSuppressed(error);
            }
        }
        if (emissionError != null) {
            var failure = emissionFailure();
            if (observationError != null) failure.addSuppressed(observationError);
            throw failure;
        }
        if (observationError != null) throw observationError;
        try {
            deleteTree(directory);
        } catch (IOException error) {
            throw new ObservationFailure("Cannot remove successful recording " + label, directory, report, error);
        }
    }

    private void validateReport(JsonNode value, int processExit) throws IOException {
        if (value == null || !value.isObject() || !value.path("exit_code").isIntegralNumber()
                || !value.path("exit_code").canConvertToInt()
                || !java.util.Set.of(0, 2, 3, 4).contains(value.path("exit_code").intValue())
                || !value.path("specs").isArray()) throw new IOException("Malformed observation report");
        boolean hasErrors = false;
        var names = new java.util.HashSet<String>();
        for (var spec : value.path("specs")) {
            if (!spec.path("spec").isTextual() || !names.add(spec.path("spec").asText())
                    || !spec.path("errors").isArray()) throw new IOException("Malformed spec report");
            for (String metric : java.util.List.of("events_read", "partition_keys", "verified_keys")) {
                var number = spec.path(metric);
                if (!number.isIntegralNumber() || !number.canConvertToLong() || number.longValue() < 0)
                    throw new IOException("Malformed observation metric: " + metric);
            }
            if (spec.path("verified_keys").longValue() > spec.path("partition_keys").longValue())
                throw new IOException("verified_keys exceeds partition_keys");
            for (var error : spec.path("errors")) {
                hasErrors = true;
                if (!error.path("kind").isTextual() || !java.util.Set.of("spec_violation", "parser",
                        "out_of_order", "unknown", "validation").contains(error.path("kind").asText())
                        || !error.path("message").isTextual())
                    throw new IOException("Malformed observation error");
                for (String field : java.util.List.of("key", "cause")) {
                    var optional = error.get(field);
                    if (optional == null || (!optional.isNull() && !optional.isTextual()))
                        throw new IOException("Malformed observation error " + field);
                }
                if (!error.path("events").isArray()) throw new IOException("Malformed error replay events");
                for (var replay : error.path("events")) {
                    for (String field : java.util.List.of("key", "timestamp", "event", "log_line")) {
                        if (!replay.path(field).isTextual()) throw new IOException("Malformed replay " + field);
                    }
                }
            }
        }
        if (!names.equals(new java.util.HashSet<>(observer.specs())))
            throw new IOException("Report spec set differs from requested specifications");
        boolean success = value.path("exit_code").intValue() == 0;
        if (success == hasErrors || success != (processExit == 0))
            throw new IOException("Inconsistent process status, report status and errors");
    }

    static void deleteTree(Path directory) throws IOException {
        try (var paths = Files.walk(directory)) {
            for (var path : paths.sorted(java.util.Comparator.reverseOrder()).toList()) Files.delete(path);
        }
    }
}
