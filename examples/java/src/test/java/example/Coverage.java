package example;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ObjectNode;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;
import java.util.TreeMap;
import org.junit.jupiter.api.extension.ExtensionContext;

/** Root-scoped resource: JUnit closes it after all tests, including parallel ones. */
final class Coverage implements ExtensionContext.Store.CloseableResource {
    private static final ObjectMapper JSON = new ObjectMapper();
    private final Map<String, ObjectNode> specs = new TreeMap<>();

    synchronized void add(JsonNode reports) {
        for (var report : reports) {
            var coverage = report.get("coverage");
            if (coverage == null) throw new AssertionError("Missing model coverage");
            var target = specs.computeIfAbsent(report.path("spec").asText(), key -> JSON.createObjectNode());
            target.put("scope", "observed_monitor");
            for (var category : new String[]{"states", "handlers"}) {
                var items = target.withObject("/" + category);
                for (var kind : new String[]{"declared", "visited"})
                    items.set(kind, union(items.get(kind), coverage.path(category).path(kind)));
                items.put("total", items.path("declared").size());
                items.put("covered", items.path("visited").size());
            }
            target.set("transitions", union(target.get("transitions"), coverage.path("transitions")));
        }
    }

    private static JsonNode union(JsonNode left, JsonNode right) {
        var values = new TreeMap<String, JsonNode>();
        if (left != null) for (var value : left) values.put(value.toString(), value);
        for (var value : right) values.put(value.toString(), value);
        var result = JSON.createArrayNode();
        values.values().forEach(result::add);
        return result;
    }

    synchronized ObjectNode report() {
        var report = JSON.createObjectNode().put("scope", "junit_completed_checks");
        var entries = report.putArray("specs");
        for (var entry : specs.entrySet())
            entries.addObject().put("spec", entry.getKey()).set("coverage", entry.getValue().deepCopy());
        return report;
    }

    @Override public synchronized void close() throws Exception {
        var report = report();
        for (var entry : specs.entrySet()) {
            var coverage = entry.getValue();
            System.out.printf("P model coverage %s: states %d/%d, handlers %d/%d, observed transitions %d%n",
                    entry.getKey(), coverage.path("states").path("covered").asInt(),
                    coverage.path("states").path("total").asInt(),
                    coverage.path("handlers").path("covered").asInt(),
                    coverage.path("handlers").path("total").asInt(), coverage.path("transitions").size());
        }
        var path = Path.of("target", "coverage", "model.json");
        Files.createDirectories(path.getParent());
        JSON.writerWithDefaultPrettyPrinter().writeValue(path.toFile(), report);
        System.out.println("P model coverage JSON: " + path);
    }
}
