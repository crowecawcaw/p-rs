package example;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.concurrent.TimeUnit;
import org.junit.jupiter.api.extension.ExtensionContext;

/** Root-scoped resource: JUnit closes it after all tests, including parallel ones. */
final class Coverage implements ExtensionContext.Store.CloseableResource {
    private static final ObjectMapper JSON = new ObjectMapper();
    private final com.fasterxml.jackson.databind.node.ArrayNode reports = JSON.createArrayNode();

    synchronized void add(JsonNode report) {
        reports.add(report.deepCopy());
    }

    synchronized JsonNode write(Path path) throws Exception {
        // Core owns schema validation, compatible-spec checks, and set union.
        Files.createDirectories(path.toAbsolutePath().getParent());
        var input = Files.createTempFile(path.toAbsolutePath().getParent(), "coverage-input-", ".json");
        var output = Files.createTempFile(path.toAbsolutePath().getParent(), "coverage-console-", ".txt");
        try {
            JSON.writeValue(input.toFile(), reports);
            var binary = System.getenv().getOrDefault("PRS_BIN", "prs");
            var process = new ProcessBuilder(binary, "coverage", "merge-observe", "--stdin",
                    "--json-report", path.toString()).redirectInput(input.toFile())
                    .redirectErrorStream(true).redirectOutput(output.toFile()).start();
            if (!process.waitFor(30, TimeUnit.SECONDS)) {
                process.destroyForcibly().waitFor();
                throw new AssertionError("P model coverage merge timed out");
            }
            var diagnostics = Files.readString(output);
            System.out.print(diagnostics);
            if (process.exitValue() != 0)
                throw new AssertionError("P model coverage merge exit " + process.exitValue() + "\n" + diagnostics);
            System.out.println("P model coverage JSON: " + path);
            return JSON.readTree(path.toFile());
        } finally {
            Files.deleteIfExists(input);
            Files.deleteIfExists(output);
        }
    }

    @Override public synchronized void close() throws Exception {
        write(Path.of("target", "coverage", "model.json"));
    }
}
