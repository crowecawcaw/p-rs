package io.github.crowecawcaw.snappea;

import java.io.IOException;
import java.net.URL;
import java.net.URLClassLoader;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.Map;
import java.util.concurrent.Callable;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.Executors;
import java.util.jar.JarEntry;
import java.util.jar.JarOutputStream;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.condition.DisabledOnOs;
import org.junit.jupiter.api.condition.OS;
import org.junit.jupiter.api.io.TempDir;
import static org.junit.jupiter.api.Assertions.*;
import static org.junit.jupiter.api.Assumptions.assumeTrue;

/** The checker from a natives classifier jar, built here the way the {@code natives} profile builds it. */
class BundledBinaryTest {
    @TempDir Path temporary;

    /** A natives jar for {@code classifier} holding {@code binary}, with {@code checksum} as snappea.sha256 (null: the real one). */
    private Path nativesJar(String classifier, byte[] binary, String checksum) throws IOException {
        var jar = Files.createTempFile(temporary, "snappea-" + classifier, ".jar");
        var folder = BundledBinary.ROOT + classifier + "/";
        var name = BundledBinary.executableName(classifier);
        var sum = checksum != null ? checksum : BundledBinary.sha256(Files.write(temporary.resolve("payload"), binary));
        try (var out = new JarOutputStream(Files.newOutputStream(jar))) {
            out.putNextEntry(new JarEntry(folder + name));
            out.write(binary);
            out.putNextEntry(new JarEntry(folder + "snappea.sha256"));
            out.write((sum + "  " + name + "\n").getBytes(StandardCharsets.US_ASCII));
        }
        return jar;
    }

    private static URLClassLoader loader(Path jar) throws IOException {
        return new URLClassLoader(new URL[] {jar.toUri().toURL()}, null);
    }

    private static final byte[] SCRIPT = "#!/bin/sh\necho bundled snappea \"$@\"\n".getBytes(StandardCharsets.US_ASCII);

    @Test void classifiersMatchTheReleaseNames() {
        assertEquals("natives-linux-x86_64", BundledBinary.classifier("Linux", "amd64"));
        assertEquals("natives-linux-aarch64", BundledBinary.classifier("Linux", "aarch64"));
        assertEquals("natives-macos-x86_64", BundledBinary.classifier("Mac OS X", "x86_64"));
        assertEquals("natives-macos-aarch64", BundledBinary.classifier("Mac OS X", "aarch64"));
        assertEquals("natives-windows-x86_64", BundledBinary.classifier("Windows 11", "amd64"));
        assertNull(BundledBinary.classifier("Windows 11", "aarch64"));
        assertNull(BundledBinary.classifier("FreeBSD", "amd64"));
        assertNull(BundledBinary.classifier("Linux", "s390x"));
        assertEquals("snappea.exe", BundledBinary.executableName("natives-windows-x86_64"));
        assertEquals("snappea", BundledBinary.executableName("natives-linux-x86_64"));
    }

    @Test void cacheFollowsXdgCacheHome() {
        assertEquals(temporary.resolve("snappea"), BundledBinary.cacheRoot(Map.of("XDG_CACHE_HOME", temporary.toString())));
        assertNotEquals(Path.of("relative", "snappea"), BundledBinary.cacheRoot(Map.of("XDG_CACHE_HOME", "relative")));
    }

    @Test void absentJarIsNotAnError() throws Exception {
        try (var empty = new URLClassLoader(new URL[0], null)) {
            assertNull(BundledBinary.find(empty, "natives-linux-x86_64", temporary, "1.0"));
        }
    }

    @Test @DisabledOnOs(OS.WINDOWS)
    void extractsOnceIntoAVersionedCacheAndRuns() throws Exception {
        var classifier = "natives-linux-x86_64";
        try (var loader = loader(nativesJar(classifier, SCRIPT, null))) {
            var cache = temporary.resolve("cache");
            var binary = BundledBinary.find(loader, classifier, cache, "1.2.3");
            assertTrue(binary.startsWith(cache.resolve("1.2.3")), binary.toString());
            assertTrue(binary.getParent().getFileName().toString().startsWith(classifier + "-"), binary.toString());
            assertTrue(Files.isExecutable(binary));
            var run = new ProcessBuilder(binary.toString(), "--version").redirectErrorStream(true).start();
            assertEquals("bundled snappea --version\n", new String(run.getInputStream().readAllBytes(), StandardCharsets.UTF_8));
            assertEquals(0, run.waitFor());
            try (var folder = Files.list(binary.getParent())) {
                assertEquals(1, folder.count(), "no partial files left behind");
            }
            assertEquals(binary, BundledBinary.find(loader, classifier, cache, "1.2.3"));
        }
    }

    @Test @DisabledOnOs(OS.WINDOWS)
    void replacesACorruptedCopy() throws Exception {
        var classifier = "natives-macos-aarch64";
        var cache = temporary.resolve("cache");
        Path binary;
        try (var loader = loader(nativesJar(classifier, SCRIPT, null))) {
            binary = BundledBinary.find(loader, classifier, cache, "9");
        }
        Files.writeString(binary, "#!/bin/sh\necho tampered\n");
        BundledBinary.forget(); // as in the next JVM
        try (var loader = loader(nativesJar(classifier, SCRIPT, null))) {
            var again = BundledBinary.find(loader, classifier, cache, "9");
            assertEquals(binary, again);
            assertArrayEquals(SCRIPT, Files.readAllBytes(again));
        }
    }

    @Test void rejectsAChecksumMismatch() throws Exception {
        var classifier = "natives-linux-aarch64";
        var cache = temporary.resolve("cache");
        try (var loader = loader(nativesJar(classifier, SCRIPT, "0".repeat(64)))) {
            var failure = assertThrows(IOException.class, () -> BundledBinary.find(loader, classifier, cache, "1"));
            assertTrue(failure.getMessage().contains("Checksum mismatch"), failure.getMessage());
        }
        try (var walk = Files.walk(cache)) {
            assertTrue(walk.noneMatch(Files::isRegularFile), "nothing extracted");
        }
        try (var loader = loader(nativesJar(classifier, SCRIPT, "not a checksum"))) {
            assertThrows(IOException.class, () -> BundledBinary.find(loader, classifier, cache, "1"));
        }
    }

    @Test @DisabledOnOs(OS.WINDOWS)
    void parallelExtractionsAgree() throws Exception {
        var classifier = "natives-linux-x86_64";
        var cache = temporary.resolve("cache");
        var jar = nativesJar(classifier, SCRIPT, null);
        var start = new CountDownLatch(1);
        var pool = Executors.newFixedThreadPool(8);
        try {
            var tasks = new ArrayList<java.util.concurrent.Future<Path>>();
            for (int i = 0; i < 8; i++) {
                Callable<Path> task = () -> {
                    try (var loader = loader(jar)) {
                        start.await();
                        return BundledBinary.find(loader, classifier, cache, "v");
                    }
                };
                tasks.add(pool.submit(task));
            }
            start.countDown();
            var paths = new HashSet<Path>();
            for (var task : tasks) paths.add(task.get());
            assertEquals(1, paths.size());
            var binary = paths.iterator().next();
            assertArrayEquals(SCRIPT, Files.readAllBytes(binary));
            try (var folder = Files.list(binary.getParent())) {
                assertEquals(1, folder.count(), "no partial files left behind");
            }
        } finally {
            pool.shutdownNow();
        }
    }

    /** End to end: the real checker in a natives jar for this platform, with no SNAPPEA_BIN and nothing on PATH. */
    @Test void observerUsesTheBundledCheckerAfterSnappeaBin() throws Exception {
        var snappeaBin = System.getenv("SNAPPEA_BIN");
        var classifier = BundledBinary.classifier();
        assumeTrue(snappeaBin != null && !snappeaBin.isBlank() && classifier != null, "needs SNAPPEA_BIN and a supported platform");
        var cache = temporary.resolve("xdg");
        var environment = Map.of("PATH", temporary.resolve("empty").toString(), "XDG_CACHE_HOME", cache.toString());
        var observer = SnappeaObserver.builder().config(SnappeaConfig.NONE).environment(environment)
                .specFiles(ObserverTest.spec()).specs("Counter").artifacts(temporary.resolve("artifacts")).build();
        var missing = assertThrows(SetupError.class, observer::executable);
        assertEquals(SetupError.BINARY_MISSING, missing.kind());
        assertTrue(missing.getMessage().contains("io.github.crowecawcaw:snappea:" + BundledBinary.version() + ":" + classifier),
                missing.getMessage());

        var thread = Thread.currentThread();
        var previous = thread.getContextClassLoader();
        try (var loader = new URLClassLoader(new URL[] {nativesJar(classifier, Files.readAllBytes(Path.of(snappeaBin)), null)
                .toUri().toURL()}, previous)) {
            thread.setContextClassLoader(loader);
            var executable = Path.of(observer.executable());
            assertTrue(executable.startsWith(cache.resolve("snappea").resolve(BundledBinary.version())), executable.toString());
            try (var events = observer.recording("bundled checker")) {
                events.emit("eAdd", "sku", 2);
                events.emit("eTake", "sku", 1);
            }
            var violation = assertThrows(SpecViolation.class, () -> {
                try (var events = observer.recording("bundled checker finds violations")) {
                    events.emit("eTake", "sku", 1);
                }
            });
            assertEquals("Counter", violation.spec());
            // Explicit settings and SNAPPEA_BIN still win over the bundled binary.
            var explicit = SnappeaObserver.builder().config(SnappeaConfig.NONE).environment(Map.of("SNAPPEA_BIN", snappeaBin))
                    .specFiles(ObserverTest.spec()).build();
            assertEquals(Path.of(snappeaBin).toAbsolutePath().toString(), explicit.executable());
        } finally {
            thread.setContextClassLoader(previous);
        }
    }
}
