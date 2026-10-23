package io.github.crowecawcaw.snappea;

import java.util.ArrayList;
import java.util.List;

/** Immutable full report, retaining core errors, replay details, coverage and source metadata. */
public final class ObservationReport {
    public record ReplayEvent(String key, String timestamp, String event, String logLine) {}
    public record Error(String spec, String kind, String key, String message, String cause,
                        List<ReplayEvent> events) {
        public Error { events = List.copyOf(events); }
    }

    private final int exitCode;
    private final String json;
    private final List<Error> errors;

    /** {@code value} is a report {@code Recording} validated. */
    ObservationReport(Object value) {
        exitCode = Json.get(value, "exit_code") instanceof Integer code ? code : 0;
        json = Json.pretty(value);
        var found = new ArrayList<Error>();
        for (var spec : Json.items(Json.get(value, "specs"))) {
            for (var error : Json.items(Json.get(spec, "errors"))) {
                var events = new ArrayList<ReplayEvent>();
                for (var event : Json.items(Json.get(error, "events"))) {
                    events.add(new ReplayEvent(text(event, "key"), text(event, "timestamp"),
                            text(event, "event"), text(event, "log_line")));
                }
                found.add(new Error(text(spec, "spec"), text(error, "kind"), text(error, "key"),
                        text(error, "message"), text(error, "cause"), events));
            }
        }
        errors = List.copyOf(found);
    }

    private static String text(Object node, String field) {
        var value = Json.get(node, field);
        return value == null ? null : value instanceof String s ? s : Json.write(value);
    }

    /**
     * snappea's exit status: 0 no error, 1 a spec violation, 2 another error. Classify errors by
     * {@link Error#kind()}, not by this.
     */
    public int exitCode() { return exitCode; }
    public List<Error> errors() { return errors; }
    public String json() { return json; }
}
