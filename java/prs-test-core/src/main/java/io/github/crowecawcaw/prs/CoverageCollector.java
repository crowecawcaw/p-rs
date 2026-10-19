package io.github.crowecawcaw.prs;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.ArrayList;
import java.util.List;

/** Thread-safe suite scope. The core CLI owns validation, fingerprint checks and set union. */
public final class CoverageCollector implements AutoCloseable {
    private final PrsObserver observer;
    private final Path destination;
    private final Path directory;
    private final List<Path> reports = new ArrayList<>();
    private boolean closed;

    CoverageCollector(PrsObserver observer, Path destination) {
        this.observer = observer;
        this.destination = destination.toAbsolutePath().normalize();
        Path created = observer.artifacts();
        try {
            Files.createDirectories(created);
            created = Files.createTempDirectory(created, "coverage-");
            directory = created;
        } catch (IOException error) {
            throw new ObservationFailure("Cannot create coverage scope", created, null, error);
        }
    }

    public Path destination() { return destination; }

    void requireCompatible(PrsObserver candidate) {
        if (!observer.coverageCompatibilityKey().equals(candidate.coverageCompatibilityKey()))
            throw new IllegalArgumentException("Coverage collector has incompatible binary, timeout or artifact settings");
    }

    synchronized void add(ObservationReport report) {
        if (closed) throw new ObservationFailure("Coverage scope was closed before recording", directory, report, null);
        try {
            var path = directory.resolve("report-" + reports.size() + ".json");
            Files.writeString(path, report.json());
            reports.add(path);
        } catch (IOException error) {
            throw new ObservationFailure("Cannot retain coverage report", directory, report, error);
        }
    }

    @Override public synchronized void close() {
        if (closed) return;
        closed = true;
        var input = directory.resolve("observations.json");
        var console = directory.resolve("console.txt");
        Path temporary = null;
        try {
            // Stream immutable reports from disk instead of retaining every test's
            // replay history in heap. --stdin avoids OS argument-length limits.
            try (var writer = Files.newBufferedWriter(input)) {
                writer.write("[");
                for (int i = 0; i < reports.size(); i++) {
                    if (i > 0) writer.write(",");
                    try (var reader = Files.newBufferedReader(reports.get(i))) { reader.transferTo(writer); }
                }
                writer.write("]");
            }
            Files.createDirectories(destination.getParent());
            temporary = Files.createTempFile(destination.getParent(), "prs-coverage-", ".json");
            int exit = Processes.run(List.of(observer.binary(), "coverage", "merge-observe", "--stdin",
                    "--json-report", temporary.toString()), input, console, observer.timeout());
            if (exit != 0) throw new IOException("Coverage merge exit " + exit);
            var merged = Recording.JSON.readTree(temporary.toFile());
            if (merged == null || !merged.path("scope").asText().equals("observed_monitor_suite")
                    || merged.path("schema_version").asInt(-1) != 1 || !merged.path("specs").isArray()
                    || !merged.path("observations").isArray()) throw new IOException("Malformed merged coverage report");
            try {
                Files.move(temporary, destination, StandardCopyOption.REPLACE_EXISTING, StandardCopyOption.ATOMIC_MOVE);
            } catch (java.nio.file.AtomicMoveNotSupportedException unsupported) {
                Files.move(temporary, destination, StandardCopyOption.REPLACE_EXISTING);
            }
            System.out.print(Processes.tail(console));
            System.out.println("Observed monitor coverage JSON: " + destination);
            Recording.deleteTree(directory);
        } catch (IOException | RuntimeException error) {
            throw new ObservationFailure("Cannot merge observed monitor coverage\n" + Processes.tail(console),
                    directory, null, error);
        } finally {
            if (temporary != null) {
                try { Files.deleteIfExists(temporary); } catch (IOException ignored) { /* retain original failure */ }
            }
        }
    }
}
