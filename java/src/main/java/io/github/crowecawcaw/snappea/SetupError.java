package io.github.crowecawcaw.snappea;

import java.nio.file.Path;

/**
 * The check could not run or its input is unusable; not a spec violation. Assert on
 * {@link #kind()}, never on the message text.
 */
@SuppressWarnings("deprecation")
public final class SetupError extends ObservationFailure {
    private static final long serialVersionUID = 1L;
    /** No checker binary; the message says how to install one. */
    public static final String BINARY_MISSING = "binary_missing";
    /** Bad options or config file, missing spec files, P compile errors, unknown spec name. */
    public static final String CONFIG = "config";
    /** An event line or payload the core cannot read (including truncated lines). */
    public static final String PARSER = "parser";
    /** {@code strict} found a selected spec with no relevant event. */
    public static final String VALIDATION = "validation";
    /** An event name no loaded P file declares, under {@code unknownEvents = ERROR}. */
    public static final String UNKNOWN_EVENT = "unknown_event";
    /** Live input out of order. */
    public static final String OUT_OF_ORDER = "out_of_order";
    /** The checker exceeded the timeout. */
    public static final String TIMEOUT = "timeout";

    private final String kind;

    SetupError(String kind, String message, Path artifacts, ObservationReport report, Throwable cause) {
        super(message, artifacts, report, cause);
        this.kind = kind;
    }

    static SetupError config(String message) {
        return new SetupError(CONFIG, message, null, null, null);
    }

    /** One of the constants in this class, or another kind copied from the core report. */
    public String kind() { return kind; }
}
