package io.github.crowecawcaw.prs;

/** What the check does with an event name no loaded P file declares (a typo, stale instrumentation). */
public enum UnknownEvents {
    /** Drop it, as upstream PObserve does. */
    IGNORE,
    /** Drop it and print a warning with the closest declared name. */
    WARN,
    /** Fail the check with a {@link SetupError} of kind {@code unknown_event} (the default). */
    ERROR;

    String option() { return name().toLowerCase(java.util.Locale.ROOT); }

    static UnknownEvents parse(String value, String where) {
        for (var policy : values()) if (policy.option().equals(value.trim().toLowerCase(java.util.Locale.ROOT))) return policy;
        throw SetupError.config(where + ": unknown_events must be ignore, warn or error, not '" + value + "'");
    }
}
