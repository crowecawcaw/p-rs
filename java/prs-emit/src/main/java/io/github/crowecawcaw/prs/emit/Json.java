package io.github.crowecawcaw.prs.emit;

import java.lang.reflect.Array;
import java.math.BigDecimal;
import java.math.BigInteger;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicLong;

/** A small JSON writer for payloads. Throws IllegalArgumentException for values JSON cannot hold. */
final class Json {
    private static final int MAX_DEPTH = 256;

    private Json() {}

    static void write(StringBuilder out, Object value) { write(out, value, 0); }

    private static void write(StringBuilder out, Object value, int depth) {
        if (depth > MAX_DEPTH) throw new IllegalArgumentException("payload nesting exceeds " + MAX_DEPTH + " (a cycle?)");
        if (value == null) {
            out.append("null");
        } else if (value instanceof String s) {
            string(out, s);
        } else if (value instanceof Character c) {
            string(out, c.toString());
        } else if (value instanceof Boolean b) {
            out.append(b.booleanValue());
        } else if (value instanceof Integer || value instanceof Long || value instanceof Short || value instanceof Byte
                || value instanceof BigInteger || value instanceof AtomicInteger || value instanceof AtomicLong) {
            out.append(value);
        } else if (value instanceof BigDecimal d) {
            out.append(d.toString());
        } else if (value instanceof Double || value instanceof Float) {
            double d = ((Number) value).doubleValue();
            if (!Double.isFinite(d)) throw new IllegalArgumentException("payload number is not finite: " + d);
            out.append(value);
        } else if (value instanceof Enum<?> e) {
            string(out, e.name());
        } else if (value instanceof Optional<?> o) {
            write(out, o.orElse(null), depth + 1);
        } else if (value instanceof Map<?, ?> map) {
            out.append('{');
            boolean first = true;
            for (var entry : map.entrySet()) {
                if (!first) out.append(',');
                first = false;
                string(out, key(entry.getKey()));
                out.append(':');
                write(out, entry.getValue(), depth + 1);
            }
            out.append('}');
        } else if (value instanceof Record record) {
            out.append('{');
            boolean first = true;
            for (var component : record.getClass().getRecordComponents()) {
                if (!first) out.append(',');
                first = false;
                string(out, component.getName());
                out.append(':');
                Object field;
                try {
                    var accessor = component.getAccessor();
                    accessor.trySetAccessible();
                    field = accessor.invoke(record);
                } catch (ReflectiveOperationException | RuntimeException error) {
                    throw new IllegalArgumentException("cannot read record component " + component.getName(), error);
                }
                write(out, field, depth + 1);
            }
            out.append('}');
        } else if (value instanceof Iterable<?> items) {
            out.append('[');
            boolean first = true;
            for (var item : items) {
                if (!first) out.append(',');
                first = false;
                write(out, item, depth + 1);
            }
            out.append(']');
        } else if (value.getClass().isArray()) {
            out.append('[');
            int length = Array.getLength(value);
            for (int i = 0; i < length; i++) {
                if (i > 0) out.append(',');
                write(out, Array.get(value, i), depth + 1);
            }
            out.append(']');
        } else if (value instanceof java.time.temporal.TemporalAccessor || value instanceof UUID
                || value instanceof java.time.Duration) {
            string(out, value.toString());
        } else {
            throw new IllegalArgumentException("unsupported payload type " + value.getClass().getName()
                    + ": use a Map, a record, a List, a String, a number, a boolean, an enum or null");
        }
    }

    private static String key(Object key) {
        if (key instanceof String s) return s;
        if (key instanceof Enum<?> e) return e.name();
        if (key instanceof Number || key instanceof Character || key instanceof Boolean) return key.toString();
        throw new IllegalArgumentException("payload map keys must be strings, not "
                + (key == null ? "null" : key.getClass().getName()));
    }

    static void string(StringBuilder out, String s) {
        out.append('"');
        for (int i = 0; i < s.length(); i++) {
            char c = s.charAt(i);
            switch (c) {
                case '"' -> out.append("\\\"");
                case '\\' -> out.append("\\\\");
                case '\n' -> out.append("\\n");
                case '\r' -> out.append("\\r");
                case '\t' -> out.append("\\t");
                case '\b' -> out.append("\\b");
                case '\f' -> out.append("\\f");
                default -> {
                    if (c < 0x20 || c == 0x2028 || c == 0x2029) out.append(String.format("\\u%04x", (int) c));
                    else out.append(c);
                }
            }
        }
        out.append('"');
    }
}
