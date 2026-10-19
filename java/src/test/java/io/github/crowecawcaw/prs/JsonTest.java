package io.github.crowecawcaw.prs;

import java.io.IOException;
import java.math.BigInteger;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

class JsonTest {
    @Test void parsesEveryKindOfValue() throws IOException {
        var value = Json.parse(" {\"a\": [1, -2, 2147483648, 9223372036854775808, 1.5, -0.25e2, 1E3],"
                + " \"b\": {\"t\": true, \"f\": false, \"n\": null}, \"empty\": {}, \"none\": [] }\n");
        var expected = new LinkedHashMap<String, Object>();
        expected.put("a", List.of(1, -2, 2147483648L, new BigInteger("9223372036854775808"), 1.5, -25.0, 1000.0));
        var b = new LinkedHashMap<String, Object>();
        b.put("t", true);
        b.put("f", false);
        b.put("n", null);
        expected.put("b", b);
        expected.put("empty", Map.of());
        expected.put("none", List.of());
        assertEquals(expected, value);
        assertEquals(List.of("a", "b", "empty", "none"), List.copyOf(((Map<?, ?>) value).keySet()), "document order");
        assertTrue(Json.has(Json.get(value, "b"), "n"));
        assertFalse(Json.has(Json.get(value, "b"), "missing"));
        assertEquals(2147483648L, Json.integer(Json.items(Json.get(value, "a")).get(2)));
        assertNull(Json.integer(1.5));
        assertNull(Json.get("not an object", "a"));
        assertEquals(List.of(), Json.items(null));
    }

    @Test void decodesStringEscapes() throws IOException {
        assertEquals("q\" b\\ s/ \b\f\n\r\t é \u2028", Json.parse("\"q\\\" b\\\\ s\\/ \\b\\f\\n\\r\\t \\u00e9 \\u2028\""));
        assertEquals("\uD83D\uDE00 x", Json.parse("\"\\ud83d\\ude00 x\""), "a surrogate pair");
        assertEquals("😀 é", Json.parse("\"😀 é\""), "raw UTF-16 passes through");
    }

    @Test void rejectsMalformedInput() {
        for (var bad : List.of("", " ", "{", "[1,]", "[1 2]", "{\"a\" 1}", "{\"a\":1,}", "{a:1}", "\"open", "\"\\x\"",
                "\"\\u12\"", "\"\\u12g4\"", "\"tab\there\"", "01", "1.", ".5", "-", "1e", "+1", "tru", "nul", "NaN",
                "[1] [2]", "{} x", "'a'")) {
            assertThrows(IOException.class, () -> Json.parse(bad), bad);
        }
        assertThrows(IOException.class, () -> Json.parse("[".repeat(1000) + "]".repeat(1000)), "depth limit");
    }

    @Test void writesWhatItReads() throws IOException {
        var text = "{\"s\":\"a\\\"b\\\\c\\nd\\u0001\",\"n\":[1,2147483648,1.5,null,true],\"o\":{}}";
        var value = Json.parse(text);
        assertEquals(text, Json.write(value));
        assertEquals(value, Json.parse(Json.pretty(value)));
        assertEquals("{\n  \"a\": [\n    1\n  ],\n  \"b\": {}\n}\n", Json.pretty(Json.parse("{\"a\":[1],\"b\":{}}")));
        assertThrows(IllegalArgumentException.class, () -> Json.write(Double.NaN));
    }
}
