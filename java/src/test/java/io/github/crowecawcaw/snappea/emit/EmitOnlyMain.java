package io.github.crowecawcaw.snappea.emit;

import java.util.Map;

/** Production use for {@link EmitOnlyTest}: run with only the snappea classes and the JDK on the classpath. */
public final class EmitOnlyMain {
    public static void main(String[] args) throws Exception {
        for (var absent : new String[] {"org.junit.jupiter.api.Test", "org.opentest4j.AssertionFailedError"}) {
            try {
                Class.forName(absent);
                throw new IllegalStateException(absent + " is on the classpath");
            } catch (ClassNotFoundException expected) { /* the point */ }
        }
        SnappeaEmit.setSink(Sink.stdout());
        SnappeaEmit.emit("eReserved", "coffee", Map.of("units", 2));
        SnappeaEmit.close();
        if (SnappeaEmit.dropped() != 0) throw new IllegalStateException("dropped " + SnappeaEmit.dropped());
    }
}
