package io.github.crowecawcaw.prs;

import java.nio.file.Path;
import java.time.Duration;
import java.util.LinkedHashMap;
import java.util.Map;
import org.junit.jupiter.api.extension.AfterEachCallback;
import org.junit.jupiter.api.extension.BeforeEachCallback;
import org.junit.jupiter.api.extension.ExtensionContext;
import org.junit.jupiter.api.extension.LifecycleMethodExecutionExceptionHandler;
import org.junit.jupiter.api.extension.ParameterContext;
import org.junit.jupiter.api.extension.ParameterResolutionException;
import org.junit.jupiter.api.extension.ParameterResolver;
import org.junit.jupiter.api.extension.TestExecutionExceptionHandler;

/** Automatic observation around each Jupiter invocation, including fixture setup and teardown. */
public final class PrsExtension implements BeforeEachCallback, AfterEachCallback, ParameterResolver,
        TestExecutionExceptionHandler, LifecycleMethodExecutionExceptionHandler {
    private static final ExtensionContext.Namespace ROOT = ExtensionContext.Namespace.create(PrsExtension.class, "coverage");
    private final ExtensionContext.Namespace namespace = ExtensionContext.Namespace.create(PrsExtension.class, this);
    private final PrsObserver observer;
    private final Path destination;
    private final String name;

    private PrsExtension(Builder builder) {
        observer = builder.observer.build();
        destination = builder.destination == null ? null : builder.destination.toAbsolutePath().normalize();
        name = PrsObserver.nonempty(builder.name, "extension name");
    }

    public static Builder builder() { return new Builder(); }

    @Override public void beforeEach(ExtensionContext context) {
        CoverageCollector coverage = null;
        if (destination != null) {
            var registry = context.getRoot().getStore(ROOT).getOrComputeIfAbsent(
                    "registry", ignored -> new CoverageRegistry(), CoverageRegistry.class);
            coverage = registry.collector(observer, destination);
        }
        String label = context.getUniqueId() + " [" + name + "] " + context.getDisplayName();
        var recording = coverage == null ? observer.recording(label) : observer.recording(label, coverage);
        context.getStore(namespace).put("state", new State(recording));
    }

    @Override public void afterEach(ExtensionContext context) {
        var state = context.getStore(namespace).remove("state", State.class);
        if (state == null) return; // A preceding callback may have failed before our setup.
        try {
            state.recording.close();
        } catch (ObservationFailure failure) {
            var primary = state.primary != null ? state.primary : context.getExecutionException().orElse(null);
            if (primary != null && !(primary instanceof org.opentest4j.TestAbortedException)) {
                primary.addSuppressed(failure);
            } else {
                if (primary != null) failure.addSuppressed(primary);
                throw failure;
            }
        }
    }

    @Override public boolean supportsParameter(ParameterContext parameter, ExtensionContext context) {
        if (parameter.getParameter().getType() != Recording.class) return false;
        var qualifier = parameter.findAnnotation(PrsRecording.class);
        return qualifier.isEmpty() || qualifier.get().value().equals(name);
    }

    @Override public Object resolveParameter(ParameterContext parameter, ExtensionContext context) {
        var state = context.getStore(namespace).get("state", State.class);
        if (state == null) throw new ParameterResolutionException("Recording injection is available only in @BeforeEach, "
                + "test methods and @AfterEach; use explicit observer.recording(...) in constructors/@BeforeAll");
        return state.recording;
    }

    private void remember(ExtensionContext context, Throwable error) throws Throwable {
        var state = context.getStore(namespace).get("state", State.class);
        if (state != null && state.primary == null) state.primary = error;
        throw error;
    }

    @Override public void handleTestExecutionException(ExtensionContext context, Throwable error) throws Throwable {
        remember(context, error);
    }
    @Override public void handleBeforeEachMethodExecutionException(ExtensionContext context, Throwable error) throws Throwable {
        remember(context, error);
    }
    @Override public void handleAfterEachMethodExecutionException(ExtensionContext context, Throwable error) throws Throwable {
        remember(context, error);
    }

    private static final class State {
        private final Recording recording;
        private Throwable primary;
        private State(Recording recording) { this.recording = recording; }
    }

    private static final class CoverageRegistry implements ExtensionContext.Store.CloseableResource {
        private record Entry(String configuration, CoverageCollector collector) {}
        private final Map<Path, Entry> entries = new LinkedHashMap<>();

        synchronized CoverageCollector collector(PrsObserver observer, Path destination) {
            var existing = entries.get(destination);
            if (existing != null) {
                if (!existing.configuration.equals(observer.coverageCompatibilityKey()))
                    throw new IllegalArgumentException("Conflicting binary, timeout or artifact settings for coverage " + destination);
                return existing.collector;
            }
            var collector = observer.coverage(destination);
            entries.put(destination, new Entry(observer.coverageCompatibilityKey(), collector));
            return collector;
        }

        @Override public synchronized void close() {
            ObservationFailure failure = null;
            for (var entry : entries.values()) {
                try { entry.collector.close(); }
                catch (ObservationFailure error) {
                    if (failure == null) failure = error;
                    else failure.addSuppressed(error);
                }
            }
            if (failure != null) throw failure;
        }
    }

    public static final class Builder {
        private final PrsObserver.Builder observer = PrsObserver.builder();
        private Path destination;
        private String name = "prs";

        private Builder() {}
        public Builder specFiles(Path... values) { observer.specFiles(values); return this; }
        public Builder specs(String... values) { observer.specs(values); return this; }
        public Builder binary(String value) { observer.binary(value); return this; }
        public Builder artifacts(Path value) { observer.artifacts(value); return this; }
        public Builder timeout(Duration value) { observer.timeout(value); return this; }
        public Builder strict(boolean value) { observer.strict(value); return this; }
        public Builder checkHot(boolean value) { observer.checkHot(value); return this; }
        public Builder coverage(Path value) { destination = java.util.Objects.requireNonNull(value); return this; }
        public Builder name(String value) { name = value; return this; }
        public PrsExtension build() { return new PrsExtension(this); }
    }
}
