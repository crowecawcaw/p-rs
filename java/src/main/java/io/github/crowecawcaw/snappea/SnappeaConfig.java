package io.github.crowecawcaw.snappea;

import java.io.IOException;
import java.io.InputStream;
import java.net.URISyntaxException;
import java.net.URL;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Properties;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;

/**
 * The project config file: {@code snappea.properties} (searched from the working directory
 * upwards, then on the classpath), else {@code snappea.*} keys in {@code junit-platform.properties}
 * on the classpath. Relative paths resolve against the file's folder.
 */
final class SnappeaConfig {
    static final Set<String> KEYS = Set.of("spec_files", "specs", "exclude", "strict", "check_hot",
            "unknown_events", "timeout", "artifacts", "mode", "coverage");
    static final SnappeaConfig NONE = new SnappeaConfig(null, Map.of());
    private static final Map<Path, SnappeaConfig> CACHE = new ConcurrentHashMap<>();

    /** The file the values came from, or null. */
    final Path file;
    final Map<String, String> values;

    SnappeaConfig(Path file, Map<String, String> values) {
        this.file = file;
        this.values = Map.copyOf(values);
    }

    /** The config for this JVM's working directory and classpath, read once. */
    static SnappeaConfig load() {
        var start = Path.of("").toAbsolutePath();
        return CACHE.computeIfAbsent(start, ignored -> load(start, Thread.currentThread().getContextClassLoader()));
    }

    static SnappeaConfig load(Path start, ClassLoader loader) {
        for (var folder = start; folder != null; folder = folder.getParent()) {
            var file = folder.resolve("snappea.properties");
            if (Files.isRegularFile(file)) return read(file, "");
        }
        if (loader == null) loader = SnappeaConfig.class.getClassLoader();
        var own = loader.getResource("snappea.properties");
        if (own != null) return read(own, "");
        var junit = loader.getResource("junit-platform.properties");
        if (junit != null) {
            var config = read(junit, "snappea.");
            if (!config.values.isEmpty()) return config;
        }
        return NONE;
    }

    static SnappeaConfig read(Path file, String prefix) {
        try (var in = Files.newInputStream(file)) {
            return parse(file, in, prefix);
        } catch (IOException error) {
            throw new SetupError(SetupError.CONFIG, "Cannot read " + file + ": " + error, null, null, error);
        }
    }

    private static SnappeaConfig read(URL url, String prefix) {
        Path file = null;
        if ("file".equals(url.getProtocol())) {
            try { file = Path.of(url.toURI()); } catch (URISyntaxException | IllegalArgumentException ignored) { /* below */ }
        }
        if (file != null) return read(file, prefix);
        try (var in = url.openStream()) {
            return parse(null, in, prefix);
        } catch (IOException error) {
            throw new SetupError(SetupError.CONFIG, "Cannot read " + url + ": " + error, null, null, error);
        }
    }

    static SnappeaConfig parse(Path file, InputStream in, String prefix) throws IOException {
        var properties = new Properties();
        properties.load(in);
        var values = new LinkedHashMap<String, String>();
        for (var name : properties.stringPropertyNames()) {
            if (!name.startsWith(prefix)) continue;
            var key = name.substring(prefix.length());
            if (!KEYS.contains(key))
                throw SetupError.config((file == null ? "config" : file) + ": unknown key '" + name + "'; known keys: "
                        + String.join(", ", KEYS.stream().sorted().map(k -> prefix + k).toList()));
            values.put(key, properties.getProperty(name).trim());
        }
        return new SnappeaConfig(file == null ? null : file.toAbsolutePath(), values);
    }

    String where() { return file == null ? "config" : file.toString(); }

    /** The folder relative paths resolve against. */
    Path base() {
        return file == null ? Path.of("").toAbsolutePath() : file.getParent();
    }

    String get(String key) {
        var value = values.get(key);
        return value == null || value.isEmpty() ? null : value;
    }

    List<String> list(String key) {
        var value = get(key);
        if (value == null) return null;
        return Arrays.stream(value.split(",")).map(String::trim).filter(s -> !s.isEmpty()).toList();
    }

    Boolean bool(String key) {
        var value = get(key);
        if (value == null) return null;
        if (value.equalsIgnoreCase("true")) return true;
        if (value.equalsIgnoreCase("false")) return false;
        throw SetupError.config(where() + ": " + key + " must be true or false, not '" + value + "'");
    }
}
