package io.github.crowecawcaw.prs;

import java.nio.file.Path;
import java.util.Optional;
import org.opentest4j.AssertionFailedError;

/** A failed check: {@link SpecViolation} or {@link SetupError}. Failed artifacts are kept. */
abstract class CheckFailure extends AssertionFailedError {
    private static final long serialVersionUID = 1L;
    private final transient Path artifacts;
    private final transient ObservationReport report;

    CheckFailure(String message, Path artifacts, ObservationReport report, Throwable cause) {
        super(message, cause);
        this.artifacts = artifacts;
        this.report = report;
    }

    /** The folder holding the events, prs's console output and its report; null when none was made. */
    public Path artifacts() { return artifacts; }

    /** The parsed JSON report, when prs wrote a valid one. */
    public Optional<ObservationReport> report() { return Optional.ofNullable(report); }
}
