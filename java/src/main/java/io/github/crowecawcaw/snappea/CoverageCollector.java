package io.github.crowecawcaw.snappea;

import java.io.IOException;
import java.nio.channels.FileChannel;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.nio.file.StandardOpenOption;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;

/**
 * Suite coverage: collects each check's coverage and, when closed, merges it into the
 * destination with {@code snappea coverage merge-observe}. An existing destination is merged
 * into, under an exclusive file lock, so several JVMs (Maven or Gradle forks) can share one
 * path; delete a stale file before a run. Thread-safe.
 */
public final class CoverageCollector implements AutoCloseable {
    private static final Map<Path, Object> JVM_LOCKS = new ConcurrentHashMap<>();
    private final SnappeaObserver observer;
    private final Path destination;
    private final Path directory;
    private final List<Path> reports = new ArrayList<>();
    private boolean closed;

    CoverageCollector(SnappeaObserver observer, Path destination) {
        this.observer = observer;
        this.destination = destination == null ? null : destination.toAbsolutePath().normalize();
        if (this.destination == null) {
            directory = null;
            return;
        }
        Path created = observer.artifacts();
        try {
            Files.createDirectories(created);
            created = Files.createTempDirectory(created, "coverage-");
            directory = created;
        } catch (IOException error) {
            throw new SetupError(SetupError.CONFIG, "Cannot create coverage scope: " + error, created, null, error);
        }
    }

    /** The coverage JSON written on close; null for an inert collector (no destination configured). */
    public Path destination() { return destination; }

    boolean enabled() { return destination != null; }

    void requireCompatible(SnappeaObserver candidate) {
        if (!observer.coverageCompatibilityKey().equals(candidate.coverageCompatibilityKey()))
            throw new IllegalArgumentException("Coverage collector has incompatible binary, timeout or artifact settings");
    }

    synchronized void add(ObservationReport report) {
        if (destination == null) return;
        if (closed) throw new SetupError(SetupError.CONFIG, "Coverage scope was closed before recording", directory, report, null);
        try {
            var path = directory.resolve("report-" + reports.size() + ".json");
            Files.writeString(path, report.json());
            reports.add(path);
        } catch (IOException error) {
            throw new SetupError(SetupError.CONFIG, "Cannot keep coverage report: " + error, directory, report, error);
        }
    }

    /** Merges into the destination and prints the summary. Idempotent. */
    @Override public synchronized void close() {
        if (closed || destination == null) return;
        closed = true;
        var input = directory.resolve("observations.json");
        var console = directory.resolve("console.txt");
        Path temporary = null;
        try {
            Files.createDirectories(destination.getParent());
            synchronized (JVM_LOCKS.computeIfAbsent(destination, ignored -> new Object())) {
                try (var lock = FileChannel.open(destination.resolveSibling(destination.getFileName() + ".lock"),
                        StandardOpenOption.CREATE, StandardOpenOption.WRITE); var held = lock.lock()) {
                    // Stream reports from disk instead of keeping every test's replay history in
                    // memory. --stdin avoids OS argument-length limits.
                    try (var writer = Files.newBufferedWriter(input)) {
                        writer.write("[");
                        boolean first = true;
                        if (Files.isRegularFile(destination) && Files.size(destination) > 0) {
                            try (var reader = Files.newBufferedReader(destination)) { reader.transferTo(writer); }
                            first = false;
                        }
                        for (var report : reports) {
                            if (!first) writer.write(",");
                            first = false;
                            try (var reader = Files.newBufferedReader(report)) { reader.transferTo(writer); }
                        }
                        writer.write("]");
                    }
                    temporary = Files.createTempFile(destination.getParent(), "snappea-coverage-", ".json");
                    int exit = Processes.run(List.of(observer.executable(), "coverage", "merge-observe", "--stdin",
                            "--json-report", temporary.toString()), input, console, observer.timeout());
                    if (exit != 0) throw new IOException("Coverage merge exit " + exit);
                    var merged = Json.read(temporary);
                    if (!"observed_monitor_suite".equals(Json.get(merged, "scope"))
                            || !Integer.valueOf(1).equals(Json.get(merged, "schema_version"))
                            || !(Json.get(merged, "specs") instanceof List)
                            || !(Json.get(merged, "observations") instanceof List))
                        throw new IOException("Malformed merged coverage report");
                    try {
                        Files.move(temporary, destination, StandardCopyOption.REPLACE_EXISTING, StandardCopyOption.ATOMIC_MOVE);
                    } catch (java.nio.file.AtomicMoveNotSupportedException unsupported) {
                        Files.move(temporary, destination, StandardCopyOption.REPLACE_EXISTING);
                    }
                }
            }
            System.out.print(Processes.tail(console));
            System.out.println("Observed monitor coverage JSON: " + destination);
            Recording.deleteTree(directory);
        } catch (SetupError error) {
            throw error;
        } catch (IOException | RuntimeException error) {
            throw new SetupError(SetupError.CONFIG, "Cannot merge observed monitor coverage into " + destination + ": "
                    + error + "\n" + Processes.tail(console) + "\n  snappea output: " + directory, directory, null, error);
        } finally {
            if (temporary != null) {
                try { Files.deleteIfExists(temporary); } catch (IOException ignored) { /* keep the original failure */ }
            }
        }
    }
}
