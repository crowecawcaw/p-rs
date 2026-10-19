package io.github.crowecawcaw.prs;

import java.nio.file.Path;
import java.time.Duration;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Objects;

/** Immutable CLI configuration. Each recording creates independent application-observation state. */
public final class PrsObserver {
    private final List<Path> files;
    private final List<String> specs;
    private final String binary;
    private final Path artifacts;
    private final Duration timeout;
    private final boolean strict;
    private final boolean checkHot;

    private PrsObserver(Builder builder) {
        if (builder.files.isEmpty()) throw new IllegalArgumentException("specFiles must not be empty");
        if (builder.specs.isEmpty()) throw new IllegalArgumentException("specs must not be empty");
        files = builder.files.stream().map(p -> Objects.requireNonNull(p).toAbsolutePath().normalize()).toList();
        specs = builder.specs.stream().map(s -> nonempty(s, "spec")).distinct().toList();
        binary = nonempty(builder.binary, "binary");
        artifacts = builder.artifacts.toAbsolutePath().normalize();
        timeout = builder.timeout;
        if (timeout.isNegative() || timeout.isZero() || timeout.toMillis() < 1)
            throw new IllegalArgumentException("timeout must be at least one millisecond");
        strict = builder.strict;
        checkHot = builder.checkHot;
    }

    static String nonempty(String value, String name) {
        if (value == null || value.isBlank()) throw new IllegalArgumentException(name + " must not be blank");
        return value;
    }

    public static Builder builder() { return new Builder(); }
    public Recording recording(String label) { return new Recording(this, label, null); }
    public Recording recording(String label, CoverageCollector coverage) {
        Objects.requireNonNull(coverage).requireCompatible(this);
        return new Recording(this, label, coverage);
    }
    public CoverageCollector coverage(Path destination) { return new CoverageCollector(this, destination); }

    /** Used to reject collectors sharing a destination but using different CLI settings. */
    public String coverageCompatibilityKey() {
        return binary + "\n" + timeout + "\n" + artifacts;
    }

    List<String> command(Path log, Path output, Path report, boolean coverage) {
        var args = new ArrayList<>(List.of(binary, "observe", "--pfiles"));
        files.forEach(p -> args.add(p.toString()));
        args.add("--spec");
        args.addAll(specs);
        args.addAll(List.of("--logs", log.toString(), "--outputDir", output.toString(),
                "--json-report", report.toString()));
        if (strict) args.add("--strict");
        if (checkHot) args.add("--check-hot");
        if (coverage) args.add("--coverage");
        return args;
    }

    String binary() { return binary; }
    Path artifacts() { return artifacts; }
    Duration timeout() { return timeout; }
    boolean strict() { return strict; }
    List<String> specs() { return specs; }

    public static final class Builder {
        private final List<Path> files = new ArrayList<>();
        private final List<String> specs = new ArrayList<>();
        private String binary = System.getenv().getOrDefault("PRS_BIN", "prs");
        private Path artifacts = Path.of("target", "prs");
        private Duration timeout = Duration.ofSeconds(30);
        private boolean strict = true;
        private boolean checkHot;

        private Builder() {}
        public Builder specFiles(Path... values) { files.addAll(Arrays.asList(values)); return this; }
        public Builder specs(String... values) { specs.addAll(Arrays.asList(values)); return this; }
        /** Executable name on PATH, or an absolute path; never a shell command. */
        public Builder binary(String value) { binary = value; return this; }
        public Builder artifacts(Path value) { artifacts = Objects.requireNonNull(value); return this; }
        public Builder timeout(Duration value) { timeout = Objects.requireNonNull(value); return this; }
        public Builder strict(boolean value) { strict = value; return this; }
        public Builder checkHot(boolean value) { checkHot = value; return this; }
        public PrsObserver build() { return new PrsObserver(this); }
    }
}
