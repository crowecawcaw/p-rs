package io.github.crowecawcaw.prs.emit;

import java.nio.file.Path;
import java.time.Duration;
import java.time.Instant;
import java.util.Objects;
import java.util.function.Consumer;

/**
 * Records that a P event happened, for P specs to check. Call it at the commit boundary of
 * application code: {@code PrsEmit.emit("eReserved", sku, Map.of("order", id, "units", 2))}.
 *
 * <p>Where events go, first match wins:
 * <ol>
 *   <li>a recording in this JVM (a test using {@code prs-test-core} or {@code prs-junit5});</li>
 *   <li>the folder in {@code PRS_EVENTS_DIR}, when a test started this process: one file
 *       per process, {@code <name>-<pid>-<random>.jsonl};</li>
 *   <li>the production sink set with {@link #toFile}, {@link #toStdout} or {@link #toConsumer}:
 *       JSON lines that {@code prs observe} reads;</li>
 *   <li>nowhere (the default): {@code emit} costs two volatile reads.</li>
 * </ol>
 *
 * <p>Outside tests, {@code emit} never throws and never waits for I/O: file and stdout sinks
 * write from a background thread, and an event that cannot be written (full queue, I/O
 * error, a payload JSON cannot hold) is dropped and counted in {@link #dropped()}.
 *
 * <p>Payloads: a {@code Map} or record for a P named tuple, a single value, or none. Enums
 * are sent by name; collections and arrays as lists.
 */
public final class PrsEmit {
    static final String EVENTS_DIR_ENV = "PRS_EVENTS_DIR";
    private static final Sinks.Sink ENV_SINK = Sinks.fromEnvironment(System.getenv(EVENTS_DIR_ENV));
    static volatile EmitHook.Router router;
    private static volatile Sinks.Sink sink = ENV_SINK;
    private static Sinks.Sink configured;

    private PrsEmit() {}

    /** Whether an event would go anywhere; use it to skip building an expensive payload. */
    public static boolean enabled() {
        return router != null || sink != null;
    }

    /** Records {@code event} on stream {@code key} now. {@code payload} may be null. */
    public static void emit(String event, String key, Object payload) {
        if (router == null && sink == null) return;
        deliver(null, event, key, payload);
    }

    /** Records an event without a payload. */
    public static void emit(String event, String key) {
        if (router == null && sink == null) return;
        deliver(null, event, key, null);
    }

    /** Records an event at a time the caller chose (an application timestamp or a logical clock). */
    public static void emitAt(Instant at, String event, String key, Object payload) {
        if (router == null && sink == null) return;
        deliver(Objects.requireNonNull(at, "at"), event, key, payload);
    }

    /** Records an event without a payload at a time the caller chose. */
    public static void emitAt(Instant at, String event, String key) {
        emitAt(at, event, key, null);
    }

    private static void deliver(Instant at, String event, String key, Object payload) {
        var hook = router;
        if (hook != null) {
            var target = hook.target();
            if (target != null) {
                // In a test: errors (a bad payload, a checked recording) fail the test.
                target.accept(EmitHook.line(at, event, key, payload, EmitHook.callSite()));
                return;
            }
        }
        var out = sink;
        if (out == null) return;
        String line;
        try {
            line = EmitHook.line(at, event, key, payload, out.wantsSource() ? EmitHook.callSite() : null);
        } catch (Throwable error) {
            Sinks.drop("cannot encode event " + event, error);
            return;
        }
        out.offer(line);
    }

    /** Production: append events to {@code file} (created if missing), one JSON line each. */
    public static void toFile(Path file) {
        configure(Sinks.file(Objects.requireNonNull(file, "file")));
    }

    /** Production: write events to standard output, one JSON line each. */
    public static void toStdout() {
        configure(Sinks.stdout());
    }

    /**
     * Production: hand each JSON line to {@code consumer}, on the emitting thread. Exceptions
     * from the consumer are caught and counted as dropped events; keep the consumer fast.
     */
    public static void toConsumer(Consumer<String> consumer) {
        configure(Sinks.consumer(Objects.requireNonNull(consumer, "consumer")));
    }

    /** Production: stop writing events (the default). */
    public static void off() {
        configure(null);
    }

    private static synchronized void configure(Sinks.Sink next) {
        var previous = configured;
        configured = next;
        // PRS_EVENTS_DIR means a test started this process: its folder wins.
        sink = ENV_SINK != null ? ENV_SINK : next;
        if (previous != null) previous.close();
    }

    /** Events dropped since start: a full queue, an I/O error, or a payload JSON cannot hold. */
    public static long dropped() {
        return Sinks.DROPPED.sum();
    }

    /** Waits up to {@code timeout} for queued events to be written; true when all were. */
    public static boolean flush(Duration timeout) {
        var out = sink;
        return out == null || out.flush(timeout);
    }
}
