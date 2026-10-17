import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { DEMOS, loadDemo, sortPaths, type Demo } from "./demos";
import { PEditor } from "./editor/PEditor";
import type { CheckResult, Diagnostic, Files } from "./protocol";
import { Runner } from "./runner";
import { decodeShare, encodeShare } from "./share";
import { TraceView } from "./trace/TraceView";

type Tab = "console" | "trace" | "log";

const STRATEGIES: { label: string; args: string[] }[] = [
  { label: "random", args: [] },
  { label: "pct (10)", args: ["--sch-pct", "10"] },
  { label: "pos", args: ["--sch-pos"] },
  { label: "feedback pct (10)", args: ["--sch-feedbackpct", "10"] },
];

const randomSeed = () => Math.floor(Math.random() * 1_000_000);
const splitArgs = (s: string) => s.split(/\s+/).filter(Boolean);
const REPO = "https://github.com/crowecawcaw/p-rs";
const WIDE = "(min-width: 901px)";

export function App() {
  const runner = useMemo(() => new Runner(), []);
  const [demo, setDemo] = useState<Demo>(DEMOS[0]);
  const [files, setFiles] = useState<Files>({});
  const [open, setOpen] = useState("");
  const [tests, setTests] = useState<string[]>([]);
  const [testCase, setTestCase] = useState(DEMOS[0].testCase);
  const [schedules, setSchedules] = useState(DEMOS[0].schedules);
  const [strategy, setStrategy] = useState(0);
  const [seed, setSeed] = useState(randomSeed);
  const [fixedSeed, setFixedSeed] = useState(false);
  const [extraArgs, setExtraArgs] = useState("");
  const [diagnostics, setDiagnostics] = useState<Diagnostic[]>([]);
  const [compileConsole, setCompileConsole] = useState("");
  const [compileOk, setCompileOk] = useState<boolean | undefined>();
  const [running, setRunning] = useState(false);
  const [result, setResult] = useState<CheckResult>();
  const [runError, setRunError] = useState<string>();
  const [tab, setTab] = useState<Tab>("console");
  const [jump, setJump] = useState<{ line: number; nonce: number }>();
  const [toast, setToast] = useState<string>();
  const [loaded, setLoaded] = useState(false);
  const edited = useRef(false);
  const [blurbOpen, setBlurbOpen] = useState(false);
  const [blurbClamped, setBlurbClamped] = useState(false);
  const blurbRef = useRef<HTMLParagraphElement>(null);
  const [optionsOpen, setOptionsOpen] = useState(() => matchMedia(WIDE).matches);

  const applyDemo = useCallback((d: Demo) => {
    const f = loadDemo(d);
    setDemo(d);
    setFiles(f);
    setOpen(f[d.open] !== undefined ? d.open : sortPaths(Object.keys(f))[0]);
    setTestCase(d.testCase);
    setSchedules(d.schedules);
    setExtraArgs(d.extraArgs ?? "");
    setStrategy(0);
    setResult(undefined);
    setRunError(undefined);
    setTab("console");
    edited.current = false;
  }, []);

  // Offer "more" only when the phone layout actually cuts the blurb off.
  useEffect(() => {
    const el = blurbRef.current;
    if (!el) return;
    const measure = () => setBlurbClamped(el.scrollHeight > el.clientHeight + 1);
    measure();
    const ro = new ResizeObserver(measure);
    ro.observe(el);
    return () => ro.disconnect();
  }, [demo]);

  // Initial load: a share link, ?demo=<id>, or the first demo.
  useEffect(() => {
    (async () => {
      const shared = await decodeShare(location.hash);
      if (shared) {
        const d = DEMOS.find((x) => x.id === shared.demo);
        if (d) setDemo(d);
        setFiles(shared.files);
        setOpen(shared.open && shared.files[shared.open] !== undefined ? shared.open : sortPaths(Object.keys(shared.files))[0]);
        if (shared.testCase) setTestCase(shared.testCase);
        if (shared.schedules) setSchedules(shared.schedules);
        setExtraArgs(shared.extraArgs ?? "");
        edited.current = true;
      } else {
        const id = new URLSearchParams(location.search).get("demo");
        applyDemo(DEMOS.find((d) => d.id === id) ?? DEMOS[0]);
      }
      setLoaded(true);
    })();
  }, [applyDemo]);

  // Compile shortly after each edit: diagnostics and the list of test cases.
  useEffect(() => {
    if (!loaded || Object.keys(files).length === 0) return;
    let stale = false;
    const t = setTimeout(async () => {
      try {
        const r = await runner.compile(files);
        if (stale) return;
        setDiagnostics(r.diagnostics);
        setCompileConsole(r.console);
        setCompileOk(r.ok);
        if (r.ok) {
          setTests(r.tests);
          setTestCase((tc) => (r.tests.includes(tc) ? tc : r.tests[0] ?? ""));
        }
      } catch {
        /* cancelled */
      }
    }, 400);
    return () => {
      stale = true;
      clearTimeout(t);
    };
  }, [files, loaded, runner]);

  const check = useCallback(async () => {
    if (running || !testCase) return;
    const s = fixedSeed ? seed : randomSeed();
    setSeed(s);
    setRunning(true);
    setRunError(undefined);
    try {
      const r = await runner.check({
        files,
        testCase,
        schedules,
        seed: s,
        extraArgs: [...STRATEGIES[strategy].args, ...splitArgs(extraArgs)],
      });
      setResult(r);
      if (r.diagnostics.length) setDiagnostics(r.diagnostics);
      setTab(r.bug && r.traceJson ? "trace" : "console");
      // Phones: the output is the last thing on the page; bring all of it into view.
      if (!matchMedia(WIDE).matches) window.scrollTo({ top: document.documentElement.scrollHeight, behavior: "smooth" });
    } catch (e) {
      const msg = String((e as Error).message ?? e);
      setRunError(msg === "cancelled" ? "Stopped." : `The checker crashed: ${msg}`);
      setResult(undefined);
      setTab("console");
    } finally {
      setRunning(false);
    }
  }, [running, testCase, fixedSeed, seed, runner, files, schedules, strategy, extraArgs]);

  const openLocation = useCallback(
    (file: string, line: number) => {
      const f = file.replace(/^\/work\//, "");
      const match = files[f] !== undefined ? f : Object.keys(files).find((k) => k.endsWith("/" + f) || k === f);
      if (!match) return;
      setOpen(match);
      setJump({ line, nonce: Date.now() });
    },
    [files],
  );

  const share = async () => {
    const hash = await encodeShare({ demo: demo.id, files, open, testCase, schedules, extraArgs });
    const url = `${location.origin}${location.pathname}${hash}`;
    history.replaceState(null, "", hash);
    try {
      await navigator.clipboard.writeText(url);
      flash("Link copied to clipboard");
    } catch {
      flash("Link is in the address bar");
    }
  };

  const flash = (msg: string) => {
    setToast(msg);
    setTimeout(() => setToast(undefined), 2200);
  };

  const pickDemo = (id: string) => {
    if (edited.current && !confirm("Discard your edits and load another demo?")) return;
    const d = DEMOS.find((x) => x.id === id)!;
    history.replaceState(null, "", `?demo=${d.id}`);
    applyDemo(d);
  };

  const addFile = () => {
    const name = prompt("New file name", "PSrc/NewMachine.p")?.trim();
    if (!name) return;
    const path = name.endsWith(".p") || name.includes(".") ? name : `${name}.p`;
    if (files[path] === undefined) setFiles({ ...files, [path]: "" });
    setOpen(path);
    edited.current = true;
  };

  const paths = useMemo(() => sortPaths(Object.keys(files)), [files]);
  const errorsByFile = useMemo(() => {
    const m = new Map<string, number>();
    for (const d of diagnostics) m.set(d.file, (m.get(d.file) ?? 0) + 1);
    return m;
  }, [diagnostics]);

  const status = running
    ? "Checking…"
    : runError
      ? runError
      : result
        ? !result.ok
          ? "Compile error"
          : result.bug
            ? "Bug found"
            : `No bugs in ${schedules} schedules`
        : compileOk === false
          ? "Compile error"
          : compileOk
            ? "Ready"
            : "Loading…";
  const statusClass = running ? "" : result?.bug || compileOk === false || (result && !result.ok) ? "bad" : result?.ok ? "good" : "";

  return (
    <div className="app">
      <header className="top">
        <div className="brand">
          <span className="logo">P</span>
          <span>Playground</span>
        </div>
        <select className="demo-select" value={demo.id} onChange={(e) => pickDemo(e.target.value)} aria-label="Demo">
          {DEMOS.map((d) => (
            <option key={d.id} value={d.id}>
              {d.title}
            </option>
          ))}
        </select>
        <div className="spacer" />
        <button className="ghost" onClick={share}>
          Share
        </button>
        <a className="button ghost" href={REPO} target="_blank" rel="noreferrer">
          GitHub
        </a>
      </header>

      <div className={blurbOpen ? "blurb open" : "blurb"}>
        <p ref={blurbRef}>
          {demo.blurb}{" "}
          <span className="muted">
            Runs entirely in your browser: the <a href={REPO}>Rust P toolchain</a> compiled to WebAssembly.
          </span>
        </p>
        {(blurbClamped || blurbOpen) && (
          <button className="more" onClick={() => setBlurbOpen(!blurbOpen)}>
            {blurbOpen ? "less" : "more"}
          </button>
        )}
      </div>

      <main className="panes">
        <section className="pane editor-pane">
          <div className="file-tabs" role="tablist">
            {paths.map((p) => {
              const slash = p.lastIndexOf("/");
              const errs = errorsByFile.get(p);
              return (
                <button
                  key={p}
                  role="tab"
                  aria-selected={p === open}
                  className={p === open ? "file-tab active" : "file-tab"}
                  onClick={() => setOpen(p)}
                  title={p}
                >
                  {slash >= 0 && <span className="dir">{p.slice(0, slash + 1)}</span>}
                  {p.slice(slash + 1)}
                  {errs ? <span className="badge">{errs}</span> : null}
                </button>
              );
            })}
            <button className="file-tab add" onClick={addFile} title="New file" aria-label="New file">
              +
            </button>
          </div>
          {open && files[open] !== undefined && (
            <PEditor
              path={open}
              value={files[open]}
              diagnostics={diagnostics.filter((d) => d.file === open)}
              jump={jump}
              onChange={(p, text) => {
                edited.current = true;
                setFiles((f) => ({ ...f, [p]: text }));
              }}
              onRun={check}
            />
          )}
        </section>

        <section className="pane run-pane">
          <div className="controls">
            <div className="run-bar">
              <label className="tc">
                <span className="lbl">Test case</span>
                <select
                  value={testCase}
                  onChange={(e) => setTestCase(e.target.value)}
                  disabled={!tests.length}
                  aria-label="Test case"
                >
                  {tests.length === 0 && <option value={testCase}>{testCase || "—"}</option>}
                  {tests.map((t) => (
                    <option key={t} value={t}>
                      {t}
                    </option>
                  ))}
                </select>
              </label>
              {running ? (
                <button className="danger run" onClick={() => runner.cancel()}>
                  Stop
                </button>
              ) : (
                <button className="primary run" onClick={check} disabled={!testCase || compileOk === false} title="Ctrl/⌘+Enter">
                  Check
                </button>
              )}
            </div>
            <details className="options" open={optionsOpen} onToggle={(e) => setOptionsOpen((e.target as HTMLDetailsElement).open)}>
              <summary>
                Options
                <span className="muted">
                  {" "}· {schedules} schedules · {STRATEGIES[strategy].label}
                  {fixedSeed ? ` · seed ${seed}` : ""}
                  {extraArgs.trim() ? ` · ${extraArgs.trim()}` : ""}
                </span>
              </summary>
              <div className="options-grid">
                <label>
                  Schedules
                  <input
                    type="number"
                    inputMode="numeric"
                    min={1}
                    max={1_000_000}
                    value={schedules}
                    onChange={(e) => setSchedules(Math.max(1, Number(e.target.value) || 1))}
                  />
                </label>
                <label>
                  Strategy
                  <select value={strategy} onChange={(e) => setStrategy(Number(e.target.value))}>
                    {STRATEGIES.map((s, i) => (
                      <option key={s.label} value={i}>
                        {s.label}
                      </option>
                    ))}
                  </select>
                </label>
                <label>
                  Seed
                  <input
                    type="number"
                    inputMode="numeric"
                    value={seed}
                    onChange={(e) => {
                      setSeed(Number(e.target.value) || 0);
                      setFixedSeed(true);
                    }}
                  />
                </label>
                <label className="check">
                  <input type="checkbox" checked={fixedSeed} onChange={(e) => setFixedSeed(e.target.checked)} />
                  Keep seed
                </label>
                <label className="span2">
                  Extra flags
                  <input
                    value={extraArgs}
                    placeholder="e.g. --timed --max-steps 500"
                    onChange={(e) => setExtraArgs(e.target.value)}
                    spellCheck={false}
                    autoCapitalize="off"
                    autoCorrect="off"
                  />
                </label>
              </div>
            </details>
          </div>

          <div className="out-head">
            <div className="out-tabs" role="tablist">
              {(["console", "trace", "log"] as Tab[]).map((t) => (
                <button
                  key={t}
                  role="tab"
                  aria-selected={tab === t}
                  className={tab === t ? "out-tab active" : "out-tab"}
                  onClick={() => setTab(t)}
                  disabled={t !== "console" && !result?.traceJson}
                >
                  {t === "console" ? "Console" : t === "trace" ? "Trace" : "Log"}
                </button>
              ))}
            </div>
            <div className="status-line">
              <span className={`status ${statusClass}`}>{status}</span>
              {!running && (compileOk === false || (result && !result.ok)) && diagnostics[0] && (
                <button className="diag" onClick={() => openLocation(diagnostics[0].file, diagnostics[0].line)}>
                  {diagnostics[0].file}:{diagnostics[0].line} {diagnostics[0].message}
                </button>
              )}
              {result && !running && <span className="muted">{(result.millis / 1000).toFixed(2)} s</span>}
              {result?.schedule && !running && (
                <span className="downloads">
                  <Download name={`${result.testCase}.schedule`} text={result.schedule} />
                  {result.traceText && <Download name={`${result.testCase}.txt`} text={result.traceText} />}
                  {result.traceJson && <Download name={`${result.testCase}.trace.json`} text={result.traceJson} />}
                </span>
              )}
            </div>
          </div>

          <div className="output">
            {tab === "console" && (
              <Console
                text={runError ?? (result ? result.console : compileOk === false ? compileConsole : "")}
                onOpenLocation={openLocation}
                empty={compileOk === false ? "" : "Press Check (Ctrl/⌘+Enter) to explore schedules of the selected test case."}
              />
            )}
            {tab === "trace" && result?.traceJson && <TraceView json={result.traceJson} onOpenLocation={openLocation} />}
            {tab === "log" && result?.traceText && <Console text={result.traceText} onOpenLocation={openLocation} />}
          </div>
        </section>
      </main>
      {toast && <div className="toast">{toast}</div>}
    </div>
  );
}

const LOC_G = /([\w./-]+\.p):(\d+)(?::(\d+))?/g;

function Console({ text, empty, onOpenLocation }: { text: string; empty?: string; onOpenLocation: (f: string, l: number) => void }) {
  if (!text) return <div className="console empty">{empty}</div>;
  return (
    <pre className="console">
      {text.split("\n").map((line, i) => {
        const cls =
          /Error|Assertion Failed|\bbug\b(?! found: 0)|Found [1-9]/i.test(line) && !/Found 0 bug/.test(line)
            ? "line bad"
            : /succeeded|Found 0 bug/.test(line)
              ? "line good"
              : line.startsWith("$ ")
                ? "line cmd"
                : "line";
        const parts: React.ReactNode[] = [];
        let last = 0;
        for (const m of line.matchAll(LOC_G)) {
          parts.push(line.slice(last, m.index));
          const [all, file, ln] = m;
          parts.push(
            <button key={m.index} className="loc" onClick={() => onOpenLocation(file, Number(ln))}>
              {all}
            </button>,
          );
          last = (m.index ?? 0) + all.length;
        }
        parts.push(line.slice(last));
        return (
          <div key={i} className={cls}>
            {parts}
          </div>
        );
      })}
    </pre>
  );
}

function Download({ name, text }: { name: string; text: string }) {
  const href = useMemo(() => URL.createObjectURL(new Blob([text], { type: "text/plain" })), [text]);
  useEffect(() => () => URL.revokeObjectURL(href), [href]);
  return (
    <a className="download" href={href} download={name} title={`Download ${name}`}>
      ⤓ {name.slice(name.indexOf(".") + 1)}
    </a>
  );
}
