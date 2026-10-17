// Syntax highlighting for P (a stream tokenizer; no parse tree needed for a demo).
import { StreamLanguage, type StreamParser } from "@codemirror/language";

const KEYWORDS = new Set(
  (
    "machine state start hot cold event on do goto entry exit defer ignore var fun return if else while foreach in " +
    "send raise announce new this halt assert assume print format receive case spec observes test main module union " +
    "type enum foreign choose sizeof keys values default with as break continue implementation refines compose hidden " +
    "param interface receives sends creates to assume tuple fairness"
  ).split(" "),
);
const TYPES = new Set("int bool float string machine event seq set map any data duration".split(" "));
const ATOMS = new Set("true false null".split(" "));

interface State {
  inComment: boolean;
}

const parser: StreamParser<State> = {
  name: "p",
  startState: () => ({ inComment: false }),
  token(stream, state) {
    if (state.inComment) {
      if (stream.skipTo("*/")) {
        stream.next();
        stream.next();
        state.inComment = false;
      } else stream.skipToEnd();
      return "comment";
    }
    if (stream.eatSpace()) return null;
    if (stream.match("//")) {
      stream.skipToEnd();
      return "comment";
    }
    if (stream.match("/*")) {
      state.inComment = true;
      return "comment";
    }
    if (stream.match(/^"(?:[^"\\]|\\.)*"?/)) return "string";
    if (stream.match(/^\d+(\.\d+)?/)) return "number";
    if (stream.match(/^[A-Za-z_]\w*/)) {
      const w = stream.current();
      if (KEYWORDS.has(w)) return "keyword";
      if (TYPES.has(w)) return "typeName";
      if (ATOMS.has(w)) return "atom";
      if (/^e[A-Z]/.test(w)) return "className"; // events by convention
      if (/^[A-Z]/.test(w)) return "variableName.special";
      return "variableName";
    }
    if (stream.match(/^(==|!=|<=|>=|&&|\|\||\+=|-=|[-+*/%<>=!])/)) return "operator";
    stream.next();
    return null;
  },
  languageData: { commentTokens: { line: "//", block: { open: "/*", close: "*/" } } },
};

export const pLanguage = StreamLanguage.define(parser);
