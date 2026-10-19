// Read-only code blocks, highlighted with the playground's P tokenizer.
import { Fragment, useMemo, type ReactNode } from "react";
import type { StringStream } from "@codemirror/language";
import { pTokenizer } from "../../editor/pTokens";

// Token names from the tokenizer, mapped to the classes in learn.css.
const TOKEN_CLASS: Record<string, string> = {
  keyword: "k",
  typeName: "ty",
  className: "ev",
  "variableName.special": "nm",
  string: "s",
  number: "n",
  atom: "n",
  comment: "c",
  operator: "op",
};

/** The parts of CodeMirror's StringStream that the P tokenizer uses. */
class LineStream {
  pos = 0;
  start = 0;
  constructor(readonly string: string) {}
  eol() { return this.pos >= this.string.length; }
  next() { return this.pos < this.string.length ? this.string[this.pos++] : undefined; }
  skipToEnd() { this.pos = this.string.length; }
  skipTo(s: string) {
    const i = this.string.indexOf(s, this.pos);
    if (i < 0) return false;
    this.pos = i;
    return true;
  }
  eatSpace() {
    const from = this.pos;
    while (this.pos < this.string.length && /\s/.test(this.string[this.pos])) this.pos++;
    return this.pos > from;
  }
  match(pattern: string | RegExp) {
    const rest = this.string.slice(this.pos);
    if (typeof pattern === "string") {
      if (!rest.startsWith(pattern)) return false;
      this.pos += pattern.length;
      return true;
    }
    const m = rest.match(pattern);
    if (!m || m.index !== 0) return null;
    this.pos += m[0].length;
    return m;
  }
  current() { return this.string.slice(this.start, this.pos); }
}

const PY_KEYWORDS = new Set("def with as import from assert return class if else for in pass raise".split(" "));

function highlightP(code: string): ReactNode[] {
  const out: ReactNode[] = [];
  const state = pTokenizer.startState!(2);
  code.split("\n").forEach((line, n) => {
    if (n > 0) out.push("\n");
    const stream = new LineStream(line);
    while (!stream.eol()) {
      stream.start = stream.pos;
      const tok = pTokenizer.token(stream as unknown as StringStream, state);
      const text = stream.current();
      const cls = tok ? TOKEN_CLASS[tok] : undefined;
      out.push(cls ? <span key={out.length} className={cls}>{text}</span> : text);
    }
  });
  return out;
}

// Enough for short snippets: comments, strings, keywords, event names.
function highlightPython(code: string): ReactNode[] {
  const re = /(#.*$)|("(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*')|\b([A-Za-z_]\w*)\b/gm;
  const out: ReactNode[] = [];
  let last = 0;
  for (const m of code.matchAll(re)) {
    const i = m.index ?? 0;
    if (i > last) out.push(code.slice(last, i));
    const cls = m[1] ? "c" : m[2] ? (/^["']e[A-Z]/.test(m[2]) ? "ev" : "s") : PY_KEYWORDS.has(m[3]) ? "k" : undefined;
    out.push(cls ? <span key={i} className={cls}>{m[0]}</span> : m[0]);
    last = i + m[0].length;
  }
  out.push(code.slice(last));
  return out;
}

interface Props {
  lang: "p" | "python" | "shell";
  children: string;
  /** Shown above the code, e.g. a file name. */
  title?: string;
}

export function Code({ lang, children, title }: Props) {
  const code = children.replace(/^\n/, "").replace(/\s+$/, "");
  const parts = useMemo(
    () => (lang === "p" ? highlightP(code) : lang === "python" ? highlightPython(code) : [code]),
    [lang, code],
  );
  return (
    <figure className={`code lang-${lang}`}>
      {title && <figcaption>{title}</figcaption>}
      <pre>
        <code>
          {parts.map((p, i) => (
            <Fragment key={i}>{p}</Fragment>
          ))}
        </code>
      </pre>
    </figure>
  );
}
