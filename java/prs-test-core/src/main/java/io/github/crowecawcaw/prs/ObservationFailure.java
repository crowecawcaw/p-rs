package io.github.crowecawcaw.prs;

import java.nio.file.Path;
import java.util.Optional;

/** An observation, recording, transport, or coverage error. Failed artifacts are retained. */
public final class ObservationFailure extends AssertionError {
    private final Path artifacts;
    private final ObservationReport report;

    ObservationFailure(String message, Path artifacts, ObservationReport report, Throwable cause) {
        super(message + "\nArtifacts: " + artifacts);
        this.artifacts = artifacts;
        this.report = report;
        if (cause != null) initCause(cause);
    }

    public Path artifacts() { return artifacts; }
    public Optional<ObservationReport> report() { return Optional.ofNullable(report); }
}
