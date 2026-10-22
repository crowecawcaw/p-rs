package io.github.crowecawcaw.snappea;

import io.github.crowecawcaw.snappea.emit.SnappeaEmit;

/** A child process for EventsTest: emits {@code eTake} with each argument as the amount. */
public final class ChildEmitter {
    public static void main(String[] args) {
        for (var amount : args) SnappeaEmit.emit("eTake", "sku", Integer.parseInt(amount));
    }
}
