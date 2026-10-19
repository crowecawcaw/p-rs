package io.github.crowecawcaw.prs;

import java.io.IOException;
import java.math.BigInteger;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * Reads and writes the JSON of prs reports, coverage files and event lines, so the
 * helpers need no JSON library. Values are {@code Map<String, Object>} (in document
 * order), {@code List<Object>}, {@code String}, {@code Integer}, {@code Long},
 * {@code BigInteger}, {@code Double}, {@code Boolean} and {@code null}.
 */
final class Json {
    private static final int MAX_DEPTH = 512;

    private final String text;
    private int at;

    private Json(String text) { this.text = text; }

    /** Parses one JSON value; malformed input throws {@link IOException}. */
    static Object parse(String text) throws IOException {
        var parser = new Json(text);
        parser.space();
        var value = parser.value(0);
        parser.space();
        if (parser.at != text.length()) throw parser.error("unexpected text after the value");
        return value;
    }

    static Object read(Path path) throws IOException {
        return parse(Files.readString(path));
    }

    // ---- navigation: absent or mistyped values are null or empty, like a missing field

    /** The field of an object, or null when {@code value} is not an object or has no such field. */
    static Object get(Object value, String field) {
        return value instanceof Map<?, ?> map ? map.get(field) : null;
    }

    static boolean has(Object value, String field) {
        return value instanceof Map<?, ?> map && map.containsKey(field);
    }

    /** The items of an array; empty when {@code value} is not an array. */
    @SuppressWarnings("unchecked")
    static List<Object> items(Object value) {
        return value instanceof List<?> list ? (List<Object>) list : List.of();
    }

    /** The string, or null when {@code value} is not a string. */
    static String text(Object value) {
        return value instanceof String s ? s : null;
    }

    /** The integer, or null when {@code value} is not an integer that fits a {@code long}. */
    static Long integer(Object value) {
        return value instanceof Integer || value instanceof Long ? ((Number) value).longValue() : null;
    }

    // ---- writing

    static String write(Object value) {
        var out = new StringBuilder();
        write(out, value, -1, 0);
        return out.toString();
    }

    /** Indented by two spaces per level, for files people read. */
    static String pretty(Object value) {
        var out = new StringBuilder();
        write(out, value, 0, 0);
        return out.append('\n').toString();
    }

    private static void write(StringBuilder out, Object value, int indent, int depth) {
        if (depth > MAX_DEPTH) throw new IllegalArgumentException("JSON nesting exceeds " + MAX_DEPTH);
        if (value == null || value instanceof Boolean || value instanceof Integer || value instanceof Long
                || value instanceof BigInteger) {
            out.append(value);
        } else if (value instanceof Double d) {
            if (!Double.isFinite(d)) throw new IllegalArgumentException("JSON number is not finite: " + d);
            out.append(d);
        } else if (value instanceof Map<?, ?> map) {
            if (map.isEmpty()) { out.append("{}"); return; }
            out.append('{');
            boolean first = true;
            for (var entry : map.entrySet()) {
                if (!first) out.append(',');
                first = false;
                newline(out, indent, depth + 1);
                string(out, String.valueOf(entry.getKey()));
                out.append(indent < 0 ? ":" : ": ");
                write(out, entry.getValue(), indent, depth + 1);
            }
            newline(out, indent, depth);
            out.append('}');
        } else if (value instanceof List<?> list) {
            if (list.isEmpty()) { out.append("[]"); return; }
            out.append('[');
            boolean first = true;
            for (var item : list) {
                if (!first) out.append(',');
                first = false;
                newline(out, indent, depth + 1);
                write(out, item, indent, depth + 1);
            }
            newline(out, indent, depth);
            out.append(']');
        } else {
            string(out, value.toString());
        }
    }

    private static void newline(StringBuilder out, int indent, int depth) {
        if (indent < 0) return;
        out.append('\n');
        out.append("  ".repeat(depth));
    }

    private static void string(StringBuilder out, String s) {
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
                    if (c < 0x20) out.append(String.format("\\u%04x", (int) c));
                    else out.append(c);
                }
            }
        }
        out.append('"');
    }

    // ---- parsing

    private Object value(int depth) throws IOException {
        if (depth > MAX_DEPTH) throw error("nesting exceeds " + MAX_DEPTH);
        if (at >= text.length()) throw error("unexpected end of input");
        char c = text.charAt(at);
        return switch (c) {
            case '{' -> object(depth);
            case '[' -> array(depth);
            case '"' -> string();
            case 't' -> literal("true", Boolean.TRUE);
            case 'f' -> literal("false", Boolean.FALSE);
            case 'n' -> literal("null", null);
            default -> {
                if (c == '-' || (c >= '0' && c <= '9')) yield number();
                throw error("unexpected character '" + c + "'");
            }
        };
    }

    private Map<String, Object> object(int depth) throws IOException {
        at++; // {
        var map = new LinkedHashMap<String, Object>();
        space();
        if (peek() == '}') { at++; return map; }
        while (true) {
            space();
            if (peek() != '"') throw error("expected a field name");
            var key = string();
            space();
            expect(':');
            space();
            map.put(key, value(depth + 1));
            space();
            char c = next();
            if (c == '}') return map;
            if (c != ',') throw error("expected ',' or '}'", at - 1);
        }
    }

    private List<Object> array(int depth) throws IOException {
        at++; // [
        var list = new ArrayList<Object>();
        space();
        if (peek() == ']') { at++; return list; }
        while (true) {
            space();
            list.add(value(depth + 1));
            space();
            char c = next();
            if (c == ']') return list;
            if (c != ',') throw error("expected ',' or ']'", at - 1);
        }
    }

    private String string() throws IOException {
        at++; // "
        var out = new StringBuilder();
        while (true) {
            char c = next();
            if (c == '"') return out.toString();
            if (c < 0x20) throw error("control character in a string", at - 1);
            if (c != '\\') { out.append(c); continue; }
            char escape = next();
            switch (escape) {
                case '"', '\\', '/' -> out.append(escape);
                case 'b' -> out.append('\b');
                case 'f' -> out.append('\f');
                case 'n' -> out.append('\n');
                case 'r' -> out.append('\r');
                case 't' -> out.append('\t');
                case 'u' -> {
                    if (at + 4 > text.length()) throw error("truncated \\u escape");
                    int code = 0;
                    for (int i = 0; i < 4; i++) {
                        int digit = Character.digit(text.charAt(at + i), 16);
                        if (digit < 0) throw error("bad \\u escape");
                        code = code * 16 + digit;
                    }
                    at += 4;
                    out.append((char) code); // surrogate pairs arrive as two escapes and join here
                }
                default -> throw error("bad escape '\\" + escape + "'", at - 1);
            }
        }
    }

    private Object number() throws IOException {
        int start = at;
        if (peek() == '-') at++;
        if (peek() == '0') at++;
        else if (!digits()) throw error("bad number", start);
        boolean integral = true;
        if (peek() == '.') {
            at++;
            integral = false;
            if (!digits()) throw error("bad number", start);
        }
        if (peek() == 'e' || peek() == 'E') {
            at++;
            integral = false;
            if (peek() == '+' || peek() == '-') at++;
            if (!digits()) throw error("bad number", start);
        }
        var number = text.substring(start, at);
        if (!integral) return Double.parseDouble(number);
        var big = new BigInteger(number);
        if (big.bitLength() < 32) return big.intValue();
        if (big.bitLength() < 64) return big.longValue();
        return big;
    }

    private boolean digits() {
        int start = at;
        while (at < text.length() && text.charAt(at) >= '0' && text.charAt(at) <= '9') at++;
        return at > start;
    }

    private Object literal(String word, Object value) throws IOException {
        if (!text.startsWith(word, at)) throw error("unexpected character '" + text.charAt(at) + "'");
        at += word.length();
        return value;
    }

    private void space() {
        while (at < text.length()) {
            char c = text.charAt(at);
            if (c != ' ' && c != '\t' && c != '\n' && c != '\r') return;
            at++;
        }
    }

    private char peek() { return at < text.length() ? text.charAt(at) : '\0'; }

    private char next() throws IOException {
        if (at >= text.length()) throw error("unexpected end of input");
        return text.charAt(at++);
    }

    private void expect(char c) throws IOException {
        if (next() != c) throw error("expected '" + c + "'", at - 1);
    }

    private IOException error(String message) { return error(message, at); }

    private IOException error(String message, int position) {
        return new IOException("Malformed JSON at offset " + position + ": " + message);
    }
}
