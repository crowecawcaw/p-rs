package example;

import com.fasterxml.jackson.databind.ObjectMapper;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HashSet;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.TimeUnit;

/** Test adapter for the public CLI, not a Java SDK. One instance per test/sequence. */
final class Recording implements Inventory.Events, AutoCloseable {
    private static final ObjectMapper JSON = new ObjectMapper();
    private final Path directory;
    private final Path log;
    private final StringBuilder lines = new StringBuilder();
    private final Set<String> keys = new HashSet<>();
    private int count;
    private final Coverage coverage;

    Recording() throws IOException { this(null); }

    Recording(Coverage coverage) throws IOException {
        this.coverage = coverage;
        Files.createDirectories(Path.of("target", "prs"));
        directory = Files.createTempDirectory(Path.of("target", "prs"), "recording-").toAbsolutePath();
        log = directory.resolve("events.jsonl");
    }

    @Override public void emit(String sku, String event, Map<String, Object> payload) {
        try {
            // Logical epoch milliseconds guarantee ordering even within one clock tick.
            lines.append(JSON.writeValueAsString(Map.of("key", sku, "event", event,
                    "timestamp", ++count, "payload", payload))).append('\n');
            keys.add(sku);
        } catch (IOException error) { throw new IllegalStateException(error); }
    }

    @Override public void close() throws Exception {
        Files.writeString(log, lines);
        if (count == 0) throw new AssertionError("No inventory events recorded: " + directory);
        var report = directory.resolve("report.json");
        var output = directory.resolve("console.txt");
        var binary = System.getenv().getOrDefault("PRS_BIN", "prs");
        var command = new java.util.ArrayList<>(java.util.List.of(binary, "observe", "--pfiles",
                Path.of("specs", "inventory.p").toAbsolutePath().toString(),
                "--spec", "InventoryCorrect", "--logs", log.toString(),
                "--outputDir", directory.resolve("observe").toString(),
                "--json-report", report.toString()));
        if (coverage != null) command.add("--coverage");
        var process = new ProcessBuilder(command).redirectErrorStream(true).redirectOutput(output.toFile()).start();
        if (!process.waitFor(30, TimeUnit.SECONDS)) {
            process.destroyForcibly().waitFor();
            throw new AssertionError("prs timed out: " + directory);
        }
        String diagnostics = "prs exit " + process.exitValue() + "; artifacts: " + directory
                + "\n" + Files.readString(output)
                + (Files.exists(report) ? "\n" + Files.readString(report) : "\nNo JSON report");
        // Parse and merge a completed observation even when it found a violation.
        // Missing/malformed reports still fail the test rather than hiding tool errors.
        var result = JSON.readTree(report.toFile());
        var specs = result.path("specs");
        if (!specs.isArray() || specs.size() != 1)
            throw new AssertionError("Invalid observation report: " + diagnostics);
        if (coverage != null) coverage.add(specs);
        if (process.exitValue() != 0) throw new AssertionError(diagnostics);
        if (result.path("exit_code").asInt(-1) != 0 || specs.size() != 1
                || !specs.get(0).path("spec").asText().equals("InventoryCorrect")
                || specs.get(0).path("events_read").asInt(-1) != count
                || specs.get(0).path("partition_keys").asInt(-1) != keys.size()
                || specs.get(0).path("verified_keys").asInt(-1) != keys.size()
                || !specs.get(0).path("errors").isArray() || !specs.get(0).path("errors").isEmpty())
            throw new AssertionError("Incomplete observation: " + diagnostics);
        // Failures retain replay logs, the input and report; successful sequences stay cheap.
        try (var paths = Files.walk(directory)) {
            for (var path : paths.sorted(java.util.Comparator.reverseOrder()).toList()) Files.delete(path);
        }
    }
}
