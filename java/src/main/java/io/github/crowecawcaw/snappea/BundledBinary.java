package io.github.crowecawcaw.snappea;

import java.io.IOException;
import java.io.InputStream;
import java.net.URL;
import java.nio.file.AtomicMoveNotSupportedException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.nio.file.attribute.PosixFileAttributeView;
import java.nio.file.attribute.PosixFilePermissions;
import java.security.DigestInputStream;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.HexFormat;
import java.util.Locale;
import java.util.Map;
import java.util.Properties;
import java.util.concurrent.ConcurrentHashMap;
import java.util.regex.Pattern;

/**
 * The checker binary from a {@code snappea} classifier jar ({@code natives-<os>-<arch>}) on the
 * classpath. The jar holds {@code io/github/crowecawcaw/snappea/native/<classifier>/snappea} (or
 * {@code snappea.exe}) and {@code snappea.sha256} beside it. On first use the binary is copied to
 * {@code <user cache>/snappea/<version>/<classifier>-<sha256 prefix>/}, verified and made
 * executable; the copy is renamed into place, so parallel JVMs never see a partial file.
 */
final class BundledBinary {
    static final String ROOT = "io/github/crowecawcaw/snappea/native/";
    private static final Pattern SHA256 = Pattern.compile("[0-9a-f]{64}");
    private static final Map<String, Path> FOUND = new ConcurrentHashMap<>();

    private BundledBinary() {}

    /** The classifier for this JVM's platform, or null when no natives jar exists for it. */
    static String classifier() {
        return classifier(System.getProperty("os.name", ""), System.getProperty("os.arch", ""));
    }

    static String classifier(String osName, String osArch) {
        var name = osName.toLowerCase(Locale.ROOT);
        String os = name.startsWith("linux") ? "linux"
                : name.startsWith("mac") || name.startsWith("darwin") ? "macos"
                : name.startsWith("windows") ? "windows" : null;
        String arch = switch (osArch.toLowerCase(Locale.ROOT)) {
            case "amd64", "x86_64", "x64" -> "x86_64";
            case "aarch64", "arm64" -> "aarch64";
            default -> null;
        };
        if (os == null || arch == null || (os.equals("windows") && arch.equals("aarch64"))) return null;
        return "natives-" + os + "-" + arch;
    }

    static String executableName(String classifier) {
        return classifier.startsWith("natives-windows-") ? "snappea.exe" : "snappea";
    }

    /** The version of this jar, which names the cache folder. */
    static String version() {
        try (var in = BundledBinary.class.getResourceAsStream("snappea.properties")) {
            if (in != null) {
                var properties = new Properties();
                properties.load(in);
                var version = properties.getProperty("version");
                if (version != null && !version.isBlank() && !version.startsWith("$")) return version;
            }
        } catch (IOException ignored) { /* below */ }
        return "dev";
    }

    /**
     * The folder for extracted binaries: {@code $XDG_CACHE_HOME/snappea} when set, else
     * {@code ~/Library/Caches/snappea} (macOS), {@code %LOCALAPPDATA%\snappea} (Windows) or
     * {@code ~/.cache/snappea}.
     */
    static Path cacheRoot(Map<String, String> environment) {
        var xdg = environment.get("XDG_CACHE_HOME");
        if (xdg != null && !xdg.isBlank() && Path.of(xdg).isAbsolute()) return Path.of(xdg, "snappea");
        var home = Path.of(System.getProperty("user.home"));
        var os = System.getProperty("os.name", "").toLowerCase(Locale.ROOT);
        if (os.startsWith("mac")) return home.resolve("Library/Caches/snappea");
        if (os.startsWith("windows")) {
            var local = environment.get("LOCALAPPDATA");
            return (local != null && !local.isBlank() ? Path.of(local) : home.resolve("AppData/Local")).resolve("snappea");
        }
        return home.resolve(".cache/snappea");
    }

    /**
     * The extracted binary for {@code classifier}, or null when its natives jar is not on
     * the classpath of {@code loader}.
     *
     * @throws IOException when the jar is there but its binary cannot be verified or extracted
     */
    static Path find(ClassLoader loader, String classifier, Path cacheRoot, String version) throws IOException {
        var name = executableName(classifier);
        var folder = ROOT + classifier + "/";
        URL binary = loader.getResource(folder + name);
        if (binary == null) return null;
        URL sum = loader.getResource(folder + "snappea.sha256");
        if (sum == null) throw new IOException(folder + "snappea.sha256 is missing from the natives jar " + binary);
        String expected;
        try (var in = sum.openStream()) {
            var text = new String(in.readAllBytes(), java.nio.charset.StandardCharsets.US_ASCII).trim();
            expected = text.split("\\s+", 2)[0].toLowerCase(Locale.ROOT);
        }
        if (!SHA256.matcher(expected).matches()) throw new IOException("Malformed checksum in " + sum);
        var target = cacheRoot.resolve(version).resolve(classifier + "-" + expected.substring(0, 16)).resolve(name);
        var key = target + "\n" + expected;
        var found = FOUND.get(key);
        if (found != null && Files.isRegularFile(found)) return found;
        if (Files.isRegularFile(target) && expected.equals(sha256(target)) && Files.isExecutable(target)) {
            FOUND.put(key, target);
            return target;
        }
        Files.createDirectories(target.getParent());
        var temporary = Files.createTempFile(target.getParent(), name + "-", ".partial");
        try {
            String actual;
            try (var in = new DigestInputStream(binary.openStream(), digest())) {
                Files.copy(in, temporary, StandardCopyOption.REPLACE_EXISTING);
                actual = HexFormat.of().formatHex(in.getMessageDigest().digest());
            }
            if (!actual.equals(expected))
                throw new IOException("Checksum mismatch for " + binary + ": expected " + expected + ", got " + actual);
            makeExecutable(temporary);
            try {
                Files.move(temporary, target, StandardCopyOption.ATOMIC_MOVE, StandardCopyOption.REPLACE_EXISTING);
            } catch (AtomicMoveNotSupportedException e) {
                Files.move(temporary, target, StandardCopyOption.REPLACE_EXISTING);
            } catch (IOException e) {
                // Another JVM put it there first and it is in use (Windows): theirs is as good.
                if (!(Files.isRegularFile(target) && expected.equals(sha256(target)))) throw e;
            }
        } finally {
            Files.deleteIfExists(temporary);
        }
        FOUND.put(key, target);
        return target;
    }

    private static void makeExecutable(Path path) throws IOException {
        if (Files.getFileAttributeView(path, PosixFileAttributeView.class) != null) {
            Files.setPosixFilePermissions(path, PosixFilePermissions.fromString("rwxr-xr-x"));
        } else if (!path.toFile().setExecutable(true, false) && !Files.isExecutable(path)) {
            throw new IOException("Cannot make " + path + " executable");
        }
    }

    /** Forgets what this JVM extracted, as a new JVM would. For tests. */
    static void forget() { FOUND.clear(); }

    static String sha256(Path path) throws IOException {
        try (InputStream in = new DigestInputStream(Files.newInputStream(path), digest())) {
            in.transferTo(java.io.OutputStream.nullOutputStream());
            return HexFormat.of().formatHex(((DigestInputStream) in).getMessageDigest().digest());
        }
    }

    private static MessageDigest digest() {
        try {
            return MessageDigest.getInstance("SHA-256");
        } catch (NoSuchAlgorithmException e) {
            throw new IllegalStateException(e);
        }
    }
}
