package io.github.crowecawcaw.prs.emit;

import java.time.Duration;
import java.time.Instant;
import java.util.Objects;

/**
 * Records that a P event happened, for P specs to check. Call it at the commit boundary of
 * application code: {@code PrsEmit.emit("eReserved", sku, Map.of("order", id, "units", 2))}.
 *
 * <p>Where events go, first match wins:
 * <ol>
 *   <li>a recording in this JVM (a test using the p-rs test helpers);</li>
 *   <li>the folder in {@code PRS_EVENTS_DIR}, when a test started this process: one file
 *       per process, {@code <name>-<pid>-<random>.jsonl};</li>
 *   <li>the production sink installed with {@link #setSink}: JSON lines that
 *       {@code prs observe} reads;</li>
 *   <li>nowhere (the default): {@code emit} costs two volatile reads.</li>
 * </ol>
 *
 * <p>Outside tests, {@code emit} never throws and never waits for I/O: sinks write from a
 * background thread, and an event that cannot be written (full queue, I/O
 * error, a payload JSON cannot hold) is dropped and counted in {@link #dropped()}.
 *
 * <p>Payloads: a {@code Map} or record for a P named tuple, a single value, or none. Enums
 * are sent by name; collections and arrays as lists.
 */
public final class PrsEmit {
    static final String EVENTS_DIR_ENV = "PRS_EVENTS_DIR";
    private static final Sinks.Out ENV_SINK = Sinks.fromEnvironment(System.getenv(EVENTS_DIR_ENV));
    static volatile EmitHook.Router router;
    private static volatile Sinks.Out sink = ENV_SINK;
    private static Sinks.Out configured;

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

    /**
     * Production: sends events to {@code sink} ({@link Sink#file}, {@link Sink#stdout},
     * {@link Sink#writer} or {@link Sink#callback}), replacing and closing any earlier sink;
     * {@code null} turns production emitting off. In a process a test started
     * ({@code PRS_EVENTS_DIR} set) it does nothing: events go to the test.
     */
    public static synchronized void setSink(Sink sink) {
        var previous = configured;
        // PRS_EVENTS_DIR means a test started this process: its folder wins.
        configured = sink == null || ENV_SINK != null ? null : sink.open.get();
        PrsEmit.sink = ENV_SINK != null ? ENV_SINK : configured;
        if (previous != null) previous.close();
    }

    /** Writes the queued events and turns production emitting off. Call it at shutdown. */
    public static void close() {
        setSink(null);
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
