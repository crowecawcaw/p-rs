package io.github.crowecawcaw.prs;

import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Duration;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Order;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.BeforeEachCallback;
import org.junit.jupiter.api.extension.RegisterExtension;
import org.junit.jupiter.api.io.TempDir;
import org.junit.platform.launcher.core.LauncherDiscoveryRequestBuilder;
import org.junit.platform.launcher.core.LauncherFactory;
import org.junit.platform.launcher.listeners.SummaryGeneratingListener;
import org.junit.platform.launcher.listeners.TestExecutionSummary;
import static org.junit.jupiter.api.Assertions.*;
import static org.junit.platform.engine.discovery.DiscoverySelectors.selectClass;

class ExtensionTest {
    @TempDir Path temporary;
    private static Path root;
    private static CountDownLatch simultaneous;

    private static PrsExtension.Builder builder(String... specs) {
        return PrsExtension.builder().specFiles(Path.of("src/test/resources/spec.p"))
                .specs(specs).artifacts(root.resolve("prs"));
    }

    private TestExecutionSummary run(boolean parallel, Class<?>... classes) {
        root = temporary;
        var request = LauncherDiscoveryRequestBuilder.request();
        for (var type : classes) request.selectors(selectClass(type));
        request.configurationParameter("junit.jupiter.execution.parallel.enabled", Boolean.toString(parallel));
        request.configurationParameter("junit.jupiter.execution.parallel.mode.default", "concurrent");
        request.configurationParameter("junit.jupiter.execution.parallel.mode.classes.default", "concurrent");
        request.configurationParameter("junit.jupiter.execution.parallel.config.strategy", "fixed");
        request.configurationParameter("junit.jupiter.execution.parallel.config.fixed.parallelism", "4");
        var listener = new SummaryGeneratingListener();
        LauncherFactory.create().execute(request.build(), listener);
        return listener.getSummary();
    }

    @Test void capturesFixturesAndParallelInvocationsWithOneSuiteUnion() throws Exception {
        simultaneous = new CountDownLatch(2);
        var summary = run(true, ParallelFixture.class, AnotherClassFixture.class);
        assertEquals(3, summary.getTestsSucceededCount(), summary.getFailures().toString());
        assertEquals(0, summary.getTotalFailureCount(), summary.getFailures().toString());
        var json = Files.readString(temporary.resolve("coverage/model.json"));
        assertTrue(json.contains("observed_monitor_suite"));
        assertTrue(json.contains("first("));
        assertTrue(json.contains("second("));
        assertTrue(json.contains("another("));
    }

    @Test void primaryBodyFailureKeepsMonitorFailureSuppressedAndCoverage() throws Exception {
        var summary = run(false, BodyFailureFixture.class);
        var error = summary.getFailures().get(0).getException();
        assertEquals("body failed", error.getMessage());
        assertSuppressedObservation(error);
        assertTrue(Files.readString(temporary.resolve("failed-coverage.json")).contains("spec_violation"));
    }

    @Test void setupFailureKeepsObservationSuppressed() {
        var summary = run(false, SetupFailureFixture.class);
        var error = summary.getFailures().get(0).getException();
        assertEquals("setup failed", error.getMessage());
        assertSuppressedObservation(error);
    }

    @Test void teardownFailureKeepsObservationSuppressed() {
        var summary = run(false, TeardownFailureFixture.class);
        var error = summary.getFailures().get(0).getException();
        assertEquals("teardown failed", error.getMessage());
        assertSuppressedObservation(error);
    }

    @Test void anotherCallbacksPrimaryFailureIsPreserved() {
        var summary = run(false, CallbackFailureFixture.class);
        var error = summary.getFailures().get(0).getException();
        assertEquals("another callback failed", error.getMessage());
        assertSuppressedObservation(error);
    }

    @Test void multipleExtensionsHaveIndependentQualifiedRecordings() {
        var summary = run(false, MultipleFixture.class);
        assertEquals(1, summary.getTestsSucceededCount(), summary.getFailures().toString());
    }

    @Test void sameCoverageDestinationRejectsConflictingConfiguration() {
        var summary = run(false, ConflictingFixture.class);
        assertEquals(1, summary.getTestsFailedCount());
        assertTrue(summary.getFailures().get(0).getException().getMessage().contains("Conflicting"));
    }

    @Test void rootCoverageMergeFailureFailsLauncherAndPreservesDestination() throws Exception {
        Files.writeString(temporary.resolve("changed.p"), Files.readString(Path.of("src/test/resources/spec.p"))
                + "\n// incompatible fingerprint\n");
        var destination = temporary.resolve("incompatible.json");
        Files.writeString(destination, "previous destination");
        var summary = run(false, IncompatibleFixture.class);
        assertTrue(summary.getTotalFailureCount() > 0, "root merge failure must fail the launcher");
        assertEquals("previous destination", Files.readString(destination));
    }

    @Test void abortedTestCannotHideObservationFailure() {
        var summary = run(false, AbortedFixture.class);
        assertEquals(1, summary.getTestsFailedCount());
        assertEquals(0, summary.getTestsAbortedCount());
    }

    private static void assertSuppressedObservation(Throwable error) {
        assertTrue(java.util.Arrays.stream(error.getSuppressed()).anyMatch(ObservationFailure.class::isInstance),
                () -> "Expected suppressed observation: " + java.util.Arrays.toString(error.getSuppressed()));
    }

    static class ParallelFixture {
        @RegisterExtension static final PrsExtension PRS = builder("Counter", "Other")
                .coverage(root.resolve("coverage/model.json")).build();
        @BeforeEach void setup(Recording events) throws Exception {
            events.emit("eAdd", "sku", 2);
            simultaneous.countDown();
            assertTrue(simultaneous.await(5, TimeUnit.SECONDS), "tests did not overlap");
        }
        @Test void first(Recording events) { events.emit("eTake", "sku", 1); }
        @Test void second(Recording events) { events.emit("eTake", "sku", 2); }
        @AfterEach void teardown(Recording events) { events.emit("eOther", "other"); }
    }

    static class AnotherClassFixture {
        @RegisterExtension static final PrsExtension PRS = builder("Other")
                .coverage(root.resolve("coverage/model.json")).build();
        @Test void another(Recording events) { events.emit("eOther", "other"); }
    }

    static class BodyFailureFixture {
        @RegisterExtension static final PrsExtension PRS = builder("Counter")
                .coverage(root.resolve("failed-coverage.json")).build();
        @Test void fails(Recording events) {
            events.emit("eTake", "sku", 1);
            throw new IllegalStateException("body failed");
        }
    }

    static class SetupFailureFixture {
        @RegisterExtension static final PrsExtension PRS = builder("Counter").build();
        @BeforeEach void setup(Recording events) {
            events.emit("eTake", "sku", 1);
            throw new IllegalStateException("setup failed");
        }
        @Test void neverReached() { fail("body executed after setup failure"); }
    }

    static class TeardownFailureFixture {
        @RegisterExtension static final PrsExtension PRS = builder("Counter").build();
        @Test void body(Recording events) { events.emit("eAdd", "sku", 1); }
        @AfterEach void teardown(Recording events) {
            events.emit("eTake", "sku", 2);
            throw new IllegalStateException("teardown failed");
        }
    }

    static class CallbackFailureFixture {
        @RegisterExtension @Order(0) static final PrsExtension PRS = builder("Counter").build();
        @RegisterExtension @Order(1) static final BeforeEachCallback FAILURE = context -> {
            throw new IllegalStateException("another callback failed");
        };
        @Test void neverReached() { fail("body executed after callback failure"); }
    }

    static class MultipleFixture {
        @RegisterExtension static final PrsExtension FIRST = builder("Counter").name("first").build();
        @RegisterExtension static final PrsExtension SECOND = builder("Other").name("second").build();
        @Test void separate(@PrsRecording("first") Recording first, @PrsRecording("second") Recording second) {
            first.emit("eAdd", "sku", 1);
            second.emit("eOther", "sku");
        }
    }

    static class ConflictingFixture {
        @RegisterExtension @Order(0) static final PrsExtension FIRST = builder("Counter").name("first")
                .coverage(root.resolve("conflict.json")).build();
        @RegisterExtension @Order(1) static final PrsExtension SECOND = builder("Counter").name("second")
                .timeout(Duration.ofSeconds(3)).coverage(root.resolve("conflict.json")).build();
        @Test void neverReached() { fail("conflicting destination accepted"); }
    }
    static class IncompatibleFixture {
        @RegisterExtension static final PrsExtension FIRST = builder("Counter").name("first")
                .coverage(root.resolve("incompatible.json")).build();
        @RegisterExtension static final PrsExtension SECOND = PrsExtension.builder().name("second")
                .specFiles(root.resolve("changed.p")).specs("Counter").artifacts(root.resolve("prs"))
                .coverage(root.resolve("incompatible.json")).build();
        @Test void sources(@PrsRecording("first") Recording first, @PrsRecording("second") Recording second) {
            first.emit("eAdd", "sku", 1);
            second.emit("eAdd", "sku", 1);
        }
    }

    static class AbortedFixture {
        @RegisterExtension static final PrsExtension PRS = builder("Counter").build();
        @Test void abort(Recording events) {
            events.emit("eTake", "sku", 1);
            org.junit.jupiter.api.Assumptions.assumeTrue(false, "abort after invalid event");
        }
    }

}
