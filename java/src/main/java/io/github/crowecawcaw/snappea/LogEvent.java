package io.github.crowecawcaw.snappea;

import java.time.Instant;
import java.util.Objects;

/**
 * One event parsed from an existing log by {@link Recording#addLog(java.nio.file.Path, LogEvent.Parser)}.
 * The time is required so that the log merges with other events by time.
 */
public record LogEvent(Instant at, String event, String key, Object payload) {
    public LogEvent {
        Objects.requireNonNull(at, "at");
        Objects.requireNonNull(event, "event");
    }

    /** Turns one log line into an event, or null to skip the line. */
    @FunctionalInterface
    public interface Parser {
        LogEvent parse(String line) throws Exception;
    }
}
