package io.github.crowecawcaw.prs;

import com.fasterxml.jackson.databind.JsonNode;
import java.util.ArrayList;
import java.util.List;

/** Immutable full report, retaining core errors, replay details, coverage and source metadata. */
public final class ObservationReport {
    public record ReplayEvent(String key, String timestamp, String event, String logLine) {}
    public record Error(String spec, String kind, String key, String message, String cause,
                        List<ReplayEvent> events) {
        public Error { events = List.copyOf(events); }
    }

    private final JsonNode value;
    private final List<Error> errors;

    ObservationReport(JsonNode value) {
        this.value = value.deepCopy();
        var found = new ArrayList<Error>();
        for (var spec : value.path("specs")) {
            for (var error : spec.path("errors")) {
                var events = new ArrayList<ReplayEvent>();
                for (var event : error.path("events")) {
                    events.add(new ReplayEvent(text(event, "key"), text(event, "timestamp"),
                            text(event, "event"), text(event, "log_line")));
                }
                found.add(new Error(text(spec, "spec"), text(error, "kind"), text(error, "key"),
                        text(error, "message"), text(error, "cause"), events));
            }
        }
        errors = List.copyOf(found);
    }

    private static String text(JsonNode node, String field) {
        var value = node.get(field);
        return value == null || value.isNull() ? null : value.asText();
    }

    public int exitCode() { return value.path("exit_code").intValue(); }
    public List<Error> errors() { return errors; }
    public String json() { return value.toPrettyString(); }
    JsonNode value() { return value.deepCopy(); }
}
