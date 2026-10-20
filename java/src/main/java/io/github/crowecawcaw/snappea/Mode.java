package io.github.crowecawcaw.snappea;

/** Which tests the framework integration ({@code SnappeaExtension}) records. Set with {@code SNAPPEA_MODE} or {@code mode}. */
public enum Mode {
    /** Every test (the default). */
    AUTO,
    /** Only tests or classes annotated {@code @Snappea}. */
    MARKED,
    /** None: the extension is inert. */
    OFF;

    static Mode parse(String value, String where) {
        for (var mode : values()) if (mode.name().equalsIgnoreCase(value.trim())) return mode;
        throw SetupError.config(where + ": mode must be auto, marked or off, not '" + value + "'");
    }
}
