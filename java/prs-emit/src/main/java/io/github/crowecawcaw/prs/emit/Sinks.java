package io.github.crowecawcaw.prs.emit;

import java.io.IOException;
import java.io.OutputStream;
import java.io.Writer;
import java.nio.ByteBuffer;
import java.nio.channels.FileChannel;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import java.security.SecureRandom;
import java.time.Duration;
import java.util.concurrent.ArrayBlockingQueue;
import java.util.concurrent.BlockingQueue;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicLong;
import java.util.concurrent.atomic.LongAdder;
import java.util.function.Consumer;

/** Where events go outside an in-process recording. Never throws from {@link Sink#offer}. */
final class Sinks {
    static final LongAdder DROPPED = new LongAdder();
    private static final AtomicBoolean WARNED = new AtomicBoolean();
    static final int QUEUE = 65_536;

    private Sinks() {}

    interface Sink {
        void offer(String line);
        default boolean wantsSource() { return false; }
        default boolean flush(Duration timeout) { return true; }
        default void close() {}
    }

    static void drop(String why, Throwable error) {
        DROPPED.increment();
        if (WARNED.compareAndSet(false, true)) {
            try {
                System.err.println("prs-emit: dropping events (" + why + (error == null ? "" : ": " + error)
                        + "); see PrsEmit.dropped(). This warning is printed once.");
            } catch (Throwable ignored) {
                // nothing more to do
            }
        }
    }

    static Sink fromEnvironment(String folder) {
        return folder == null || folder.isBlank() ? null : new FolderSink(Path.of(folder), processName());
    }

    static Sink file(Path file) {
        var path = file.toAbsolutePath();
        return new AsyncSink("file " + path, () -> {
            var parent = path.getParent();
            if (parent != null) Files.createDirectories(parent);
            return Files.newOutputStream(path, StandardOpenOption.CREATE, StandardOpenOption.APPEND);
        });
    }

    static Sink stdout() {
        return new AsyncSink("stdout", () -> new java.io.FileOutputStream(java.io.FileDescriptor.out) {
            @Override public void close() throws IOException { flush(); } // never close the process's stdout
        });
    }

    static Sink consumer(Consumer<String> consumer) {
        return line -> {
            try {
                consumer.accept(line);
            } catch (Throwable error) {
                drop("the consumer threw", error);
            }
        };
    }

    /** The program name for events folder file names: {@code -Dprs.name}, else the main class or jar. */
    static String processName() {
        String name = System.getProperty("prs.name");
        if (name == null || name.isBlank()) {
            String command = System.getProperty("sun.java.command", "");
            String first = command.isBlank() ? "java" : command.trim().split("\\s+")[0];
            first = first.substring(Math.max(first.lastIndexOf('/'), first.lastIndexOf('\\')) + 1);
            if (first.endsWith(".jar")) first = first.substring(0, first.length() - 4);
            else if (first.contains(".")) first = first.substring(first.lastIndexOf('.') + 1);
            name = first;
        }
        name = name.replaceAll("[^A-Za-z0-9._-]", "_");
        return name.isEmpty() ? "java" : name;
    }

    /**
     * This process's own file in a {@code PRS_EVENTS_DIR} folder. Each line is one append,
     * written straight to the file (no user-space buffer), so a crash loses at most that line.
     */
    static final class FolderSink implements Sink {
        private final Path folder;
        private final String name;
        private FileChannel channel;
        private boolean failed;

        FolderSink(Path folder, String name) {
            this.folder = folder;
            this.name = name;
        }

        @Override public boolean wantsSource() { return true; }

        @Override public synchronized void offer(String line) {
            if (failed) {
                drop("cannot write to " + folder, null);
                return;
            }
            try {
                if (channel == null) {
                    Files.createDirectories(folder);
                    var random = new SecureRandom();
                    String file = name + "-" + ProcessHandle.current().pid() + "-"
                            + Long.toHexString(random.nextLong() & 0xffffffffL) + ".jsonl";
                    channel = FileChannel.open(folder.resolve(file), StandardOpenOption.CREATE_NEW,
                            StandardOpenOption.WRITE, StandardOpenOption.APPEND);
                }
                var bytes = ByteBuffer.wrap((line + "\n").getBytes(StandardCharsets.UTF_8));
                while (bytes.hasRemaining()) channel.write(bytes);
            } catch (IOException | RuntimeException error) {
                failed = true;
                drop("cannot write to " + folder, error);
            }
        }
    }

    interface Opener {
        OutputStream open() throws IOException;
    }

    /** A bounded queue drained by a daemon thread: {@code offer} never waits for I/O. */
    static final class AsyncSink implements Sink {
        private static final String STOP = new String("stop");
        private final String what;
        private final Opener opener;
        private final BlockingQueue<String> queue = new ArrayBlockingQueue<>(QUEUE);
        private final AtomicLong pending = new AtomicLong();
        private final Thread thread;
        private volatile boolean closed;

        AsyncSink(String what, Opener opener) {
            this.what = what;
            this.opener = opener;
            thread = new Thread(this::run, "prs-emit " + what);
            thread.setDaemon(true);
            thread.start();
            try {
                Runtime.getRuntime().addShutdownHook(new Thread(() -> flush(Duration.ofSeconds(2))));
            } catch (IllegalStateException | SecurityException ignored) {
                // shutting down already, or not allowed: best effort
            }
        }

        @Override public void offer(String line) {
            if (closed) {
                drop(what + " was replaced", null);
                return;
            }
            pending.incrementAndGet();
            if (!queue.offer(line)) {
                pending.decrementAndGet();
                drop("the queue to " + what + " is full", null);
            }
        }

        private void run() {
            Writer writer = null;
            try {
                while (true) {
                    String line = queue.poll();
                    if (line == null) {
                        if (writer != null) {
                            try { writer.flush(); } catch (IOException error) { writer = reset(writer, error); }
                        }
                        line = queue.take();
                    }
                    if (line == STOP) break;
                    try {
                        if (writer == null) writer = new java.io.BufferedWriter(new java.io.OutputStreamWriter(
                                opener.open(), StandardCharsets.UTF_8), 1 << 16);
                        writer.write(line);
                        writer.write('\n');
                        if (queue.isEmpty()) writer.flush();
                    } catch (IOException | RuntimeException error) {
                        writer = reset(writer, error);
                    } finally {
                        pending.decrementAndGet();
                    }
                }
            } catch (InterruptedException stopped) {
                Thread.currentThread().interrupt();
            } finally {
                if (writer != null) {
                    try { writer.close(); } catch (IOException ignored) { /* best effort */ }
                }
            }
        }

        private Writer reset(Writer writer, Throwable error) {
            drop("cannot write to " + what, error);
            if (writer != null) {
                try { writer.close(); } catch (IOException ignored) { /* reopened on the next line */ }
            }
            return null;
        }

        @Override public boolean flush(Duration timeout) {
            long deadline = System.nanoTime() + timeout.toNanos();
            while (pending.get() > 0) {
                if (System.nanoTime() > deadline || !thread.isAlive()) return pending.get() == 0;
                try {
                    TimeUnit.MILLISECONDS.sleep(1);
                } catch (InterruptedException interrupted) {
                    Thread.currentThread().interrupt();
                    return false;
                }
            }
            return true;
        }

        @Override public void close() {
            flush(Duration.ofSeconds(2));
            closed = true;
            queue.offer(STOP);
        }
    }
}
