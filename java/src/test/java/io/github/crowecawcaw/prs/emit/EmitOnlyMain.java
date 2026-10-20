package io.github.crowecawcaw.prs.emit;

import java.util.Map;

/** Production use for {@link EmitOnlyTest}: run with only the p-rs classes and the JDK on the classpath. */
public final class EmitOnlyMain {
    public static void main(String[] args) throws Exception {
        for (var absent : new String[] {"org.junit.jupiter.api.Test", "org.opentest4j.AssertionFailedError"}) {
            try {
                Class.forName(absent);
                throw new IllegalStateException(absent + " is on the classpath");
            } catch (ClassNotFoundException expected) { /* the point */ }
        }
        PrsEmit.setSink(Sink.stdout());
        PrsEmit.emit("eReserved", "coffee", Map.of("units", 2));
        PrsEmit.close();
        if (PrsEmit.dropped() != 0) throw new IllegalStateException("dropped " + PrsEmit.dropped());
    }
}
