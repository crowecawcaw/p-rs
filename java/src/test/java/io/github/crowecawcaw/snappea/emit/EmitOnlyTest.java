package io.github.crowecawcaw.snappea.emit;

import java.io.File;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

/** The emitter needs nothing but the JDK: JUnit and opentest4j are test-helper dependencies only. */
class EmitOnlyTest {
    private static String location(Class<?> type) throws Exception {
        return Path.of(type.getProtectionDomain().getCodeSource().getLocation().toURI()).toString();
    }

    @Test void runsWithOnlyTheP_rsClassesAndTheJdk() throws Exception {
        var java = Path.of(System.getProperty("java.home"), "bin", "java").toString();
        var classpath = location(SnappeaEmit.class) + File.pathSeparator + location(EmitOnlyMain.class);
        var process = new ProcessBuilder(java, "-cp", classpath, EmitOnlyMain.class.getName())
                .redirectErrorStream(true).start();
        process.getOutputStream().close();
        var output = new String(process.getInputStream().readAllBytes(), StandardCharsets.UTF_8);
        assertEquals(0, process.waitFor(), output);
        assertTrue(output.startsWith("{\"key\":\"coffee\","), output);
        assertTrue(output.contains("\"event\":\"eReserved\",\"payload\":{\"units\":2}"), output);
    }
}
