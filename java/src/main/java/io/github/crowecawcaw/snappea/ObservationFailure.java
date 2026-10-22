package io.github.crowecawcaw.snappea;

import java.nio.file.Path;
import java.util.Optional;
import org.opentest4j.AssertionFailedError;

/**
 * A failed check: {@link SpecViolation} or {@link SetupError}. Failed artifacts are kept.
 *
 * @deprecated Catch {@link SpecViolation} or {@link SetupError}. Kept so code written for
 *     0.1 keeps compiling; it will be removed in the next minor release.
 */
@Deprecated(since = "0.2")
public class ObservationFailure extends AssertionFailedError {
    private static final long serialVersionUID = 1L;
    private final transient Path artifacts;
    private final transient ObservationReport report;

    ObservationFailure(String message, Path artifacts, ObservationReport report, Throwable cause) {
        super(message, cause);
        this.artifacts = artifacts;
        this.report = report;
    }

    /** The folder holding the events, snappea's console output and its report; null when none was made. */
    public Path artifacts() { return artifacts; }

    /** The parsed JSON report, when snappea wrote a valid one. */
    public Optional<ObservationReport> report() { return Optional.ofNullable(report); }
}
