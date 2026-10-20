package io.github.crowecawcaw.snappea.emit;

import java.io.OutputStream;
import java.io.Writer;
import java.nio.file.Path;
import java.util.Objects;
import java.util.function.Consumer;
import java.util.function.Supplier;

/**
 * A production destination for events, installed with {@link SnappeaEmit#setSink}: JSON lines
 * that {@code snappea observe -l} reads, written by a background thread. Nothing starts until
 * the sink is installed.
 *
 * <pre>{@code
 * SnappeaEmit.setSink(Sink.file(Path.of("/var/log/app/events/inventory.jsonl")));
 * }</pre>
 */
public final class Sink {
    final Supplier<Sinks.Out> open;

    private Sink(Supplier<Sinks.Out> open) {
        this.open = open;
    }

    /** Appends events to {@code file}, creating it and its folder. */
    public static Sink file(Path file) {
        Objects.requireNonNull(file, "file");
        return new Sink(() -> Sinks.file(file));
    }

    /** Writes events to standard output. */
    public static Sink stdout() {
        return new Sink(Sinks::stdout);
    }

    /** Writes events to {@code out}. Closing the sink flushes {@code out} but does not close it. */
    public static Sink writer(Writer out) {
        Objects.requireNonNull(out, "out");
        return new Sink(() -> Sinks.writer(out));
    }

    /** Writes events to {@code out} as UTF-8. Closing the sink flushes {@code out} but does not close it. */
    public static Sink writer(OutputStream out) {
        Objects.requireNonNull(out, "out");
        return new Sink(() -> Sinks.writer(out));
    }

    /**
     * Hands each JSON line (without the newline) to {@code callback}, on the sink's background
     * thread. An exception from the callback drops that event.
     */
    public static Sink callback(Consumer<String> callback) {
        Objects.requireNonNull(callback, "callback");
        return new Sink(() -> Sinks.callback(callback));
    }
}
