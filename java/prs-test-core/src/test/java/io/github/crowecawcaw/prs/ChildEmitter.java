package io.github.crowecawcaw.prs;

import io.github.crowecawcaw.prs.emit.PrsEmit;

/** A child process for EventsTest: emits {@code eTake} with each argument as the amount. */
public final class ChildEmitter {
    public static void main(String[] args) {
        for (var amount : args) PrsEmit.emit("eTake", "sku", Integer.parseInt(amount));
    }
}
