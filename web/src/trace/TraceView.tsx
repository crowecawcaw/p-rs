import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { search, visible } from "./filter";
import { parseTrace, type Trace } from "./model";
import { laneColors, SequenceDiagram } from "./SequenceDiagram";

interface Props {
  json: string;
  /** "diagram": the chart (plus the step list beside it when there's room); "log": the step list alone. */
  view: "diagram" | "log";
  onOpenLocation: (file: string, line: number) => void;
  /** Picking a step in the log view asks to show it in the diagram. */
  onShowDiagram: () => void;
}

const LOC = /([\w./-]+\.p):(\d+):(\d+)/;
/** Show the step list beside the diagram when the trace panel is at least this wide… */
const SIDE_LIST_MIN = 820;
/** …and below it when the panel is at least this tall (phones have the Log tab instead). */
const BOTTOM_LIST_MIN_H = 560;

/** Bug trace: step-through controls, search and focus, the diagram, and the step list. */
export function TraceView({ json, view, onOpenLocation, onShowDiagram }: Props) {
  const trace = useMemo(() => parseTrace(json), [json]);
  const color = useMemo(() => laneColors(trace.lanes), [trace]);
  const [cursor, setCursor] = useState(trace.steps.length - 1);
  const [playing, setPlaying] = useState(false);
  const [bannerOpen, setBannerOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [focus, setFocus] = useState<ReadonlySet<number>>(new Set());
  const [hover, setHover] = useState<number>();
  const root = useRef<HTMLDivElement>(null);
  const [place, setPlace] = useState<"side" | "bottom" | "none">("none");

  const { rows, cols } = useMemo(() => visible(trace, focus), [trace, focus]);
  const matches = useMemo(() => search(trace, rows, query), [trace, rows, query]);
  const pos = rows.indexOf(cursor);
  const lastPos = rows.length - 1;

  // A new trace: start at the bug, with no search or focus.
  useEffect(() => {
    setCursor(trace.steps.length - 1);
    setPlaying(false);
    setFocus(new Set());
    setQuery("");
  }, [trace]);

  // Focusing can hide the current step: move to the nearest earlier one still shown.
  useEffect(() => {
    if (pos >= 0 || !rows.length) return;
    const before = rows.filter((i) => i < cursor);
    setCursor(before.length ? before[before.length - 1] : rows[0]);
  }, [pos, rows, cursor]);

  useEffect(() => {
    if (!playing) return;
    if (pos >= lastPos) {
      setPlaying(false);
      return;
    }
    const t = setTimeout(() => setCursor(rows[Math.min(pos + 1, lastPos)]), 220);
    return () => clearTimeout(t);
  }, [playing, pos, lastPos, rows]);

  useLayoutEffect(() => {
    const el = root.current;
    if (!el) return;
    const ro = new ResizeObserver(() =>
      setPlace(el.clientWidth >= SIDE_LIST_MIN ? "side" : el.clientHeight >= BOTTOM_LIST_MIN_H && !matchMedia("(max-width: 900px)").matches ? "bottom" : "none"),
    );
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  const go = (i: number | undefined) => {
    if (i === undefined) return;
    setPlaying(false);
    setCursor(i);
  };
  const toPos = (p: number) => go(rows[Math.max(0, Math.min(p, lastPos))]);
  const hits = matches ? rows.filter((i) => matches.has(i)) : [];
  const nextHit = (dir: 1 | -1) => {
    if (!hits.length) return;
    const after = dir > 0 ? hits.find((i) => i > cursor) ?? hits[0] : [...hits].reverse().find((i) => i < cursor) ?? hits[hits.length - 1];
    go(after);
  };
  const toggleLane = (l: number) =>
    setFocus((f) => {
      const n = new Set(f);
      if (n.has(l)) n.delete(l);
      else n.add(l);
      return n;
    });

  const step = trace.steps[cursor];
  const loc = trace.error ? LOC.exec(trace.error) : null;
  const list = (
    <StepList
      trace={trace}
      rows={rows}
      color={color}
      cursor={cursor}
      hover={hover}
      matches={matches}
      query={query}
      onSelect={(i) => {
        go(i);
        if (view === "log") onShowDiagram();
      }}
      onHover={setHover}
    />
  );

  return (
    <div className="trace" ref={root}>
      {trace.error && (
        <div className={bannerOpen ? "trace-error open" : "trace-error"} onClick={() => setBannerOpen(!bannerOpen)}>
          <span className="trace-error-text">
            <strong>Bug:</strong> {trace.error}
          </span>
          {loc && (
            <button
              className="link"
              onClick={(e) => {
                e.stopPropagation();
                onOpenLocation(loc[1], Number(loc[2]));
              }}
            >
              <span className="goto-long">Go to {loc[1]}:{loc[2]}</span>
              <span className="goto-short">line {loc[2]} →</span>
            </button>
          )}
        </div>
      )}
      {view === "diagram" && (
        <div className="trace-controls">
          <button onClick={() => toPos(0)} title="First step" aria-label="First step" className="jump">⏮</button>
          <button onClick={() => toPos(pos - 1)} title="Previous step" aria-label="Previous step">◀</button>
          <button
            className="primary"
            onClick={() => {
              if (!playing && pos >= lastPos) setCursor(rows[0]);
              setPlaying(!playing);
            }}
            title={playing ? "Pause" : "Play"}
          >
            {playing ? "Pause" : "Play"}
          </button>
          <button onClick={() => toPos(pos + 1)} title="Next step" aria-label="Next step">▶</button>
          <button onClick={() => toPos(lastPos)} title="Last step" aria-label="Last step" className="jump">⏭</button>
          <input
            type="range"
            min={0}
            max={Math.max(lastPos, 0)}
            value={Math.max(pos, 0)}
            onChange={(e) => toPos(Number(e.target.value))}
            aria-label="Step"
          />
          <span className="muted">
            {pos + 1}/{rows.length}
          </span>
        </div>
      )}
      <div className="trace-find">
        <input
          type="search"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              e.preventDefault();
              nextHit(e.shiftKey ? -1 : 1);
            } else if (e.key === "Escape") setQuery("");
          }}
          placeholder="Search steps or machine=Client"
          title="Matches text in each step. Also: machine=, from=, to=, event=, state=, kind=. Enter: next match."
          aria-label="Search steps"
          spellCheck={false}
          autoCapitalize="off"
          autoCorrect="off"
        />
        {matches && (
          <>
            <span className="muted hits">{hits.length ? `${hits.indexOf(cursor) + 1 || "–"}/${hits.length}` : "0"}</span>
            <button onClick={() => nextHit(-1)} disabled={!hits.length} aria-label="Previous match" title="Previous match (Shift+Enter)">↑</button>
            <button onClick={() => nextHit(1)} disabled={!hits.length} aria-label="Next match" title="Next match (Enter)">↓</button>
          </>
        )}
      </div>
      {focus.size > 0 && (
        <div className="trace-focus">
          <span className="muted">Only steps of</span>
          {[...focus].map((l) => (
            <button key={l} className="chip" style={{ "--c": color[l] } as React.CSSProperties} onClick={() => toggleLane(l)} aria-label={`Stop focusing ${trace.lanes[l].id}`}>
              <span className="chip-name">{trace.lanes[l].id}</span> <span aria-hidden="true">✕</span>
            </button>
          ))}
          <button className="link" onClick={() => setFocus(new Set())}>
            Show all
          </button>
        </div>
      )}
      {view === "log" ? (
        list
      ) : (
        <>
          <div className={`trace-main list-${place}`}>
            <SequenceDiagram
              trace={trace}
              rows={rows}
              cols={cols}
              color={color}
              focus={focus}
              cursor={cursor}
              hover={hover}
              matches={matches}
              onSelect={go}
              onHover={setHover}
              onToggleLane={toggleLane}
            />
            {place !== "none" && list}
          </div>
          {step && place === "none" && (
            <div className="step-detail">
              <div>{step.log}</div>
              {step.payload !== undefined && <pre>{JSON.stringify(step.payload, null, 2)}</pre>}
            </div>
          )}
        </>
      )}
    </div>
  );
}

/** Every shown step's log line; linked to the diagram by cursor and hover. */
function StepList({ trace, rows, color, cursor, hover, matches, query, onSelect, onHover }: {
  trace: Trace;
  rows: number[];
  color: string[];
  cursor: number;
  hover?: number;
  matches: ReadonlySet<number> | null;
  query: string;
  onSelect: (i: number) => void;
  onHover: (i: number | undefined) => void;
}) {
  const box = useRef<HTMLOListElement>(null);

  // Keep the current step in view, scrolling only the list.
  useEffect(() => {
    const el = box.current;
    const li = el?.querySelector<HTMLElement>(`[data-step="${cursor}"]`);
    if (!el || !li) return;
    const top = li.offsetTop - el.offsetTop;
    if (top < el.scrollTop) el.scrollTop = top - 8;
    else if (top + li.offsetHeight > el.scrollTop + el.clientHeight) el.scrollTop = top + li.offsetHeight - el.clientHeight + 8;
  }, [cursor, rows]);

  return (
    <ol className="step-list" ref={box} aria-label="Steps">
      {rows.map((i) => {
        const s = trace.steps[i];
        const cls = [
          i === cursor && "current",
          i === hover && "hover",
          matches && !matches.has(i) && "dim",
          s.kind === "error" && "error",
        ]
          .filter(Boolean)
          .join(" ");
        return (
          <li key={i} data-step={i} className={cls}>
            <button
              onClick={() => onSelect(i)}
              onMouseEnter={() => onHover(i)}
              onMouseLeave={() => onHover(undefined)}
              aria-current={i === cursor ? "step" : undefined}
            >
              <span className="n">{i + 1}</span>
              <span className="swatch" style={{ background: s.lane !== undefined ? color[s.lane] : "var(--muted)" }} />
              <span className="text">
                <Highlight text={s.log} query={matches && matches.has(i) ? query : ""} />
              </span>
            </button>
          </li>
        );
      })}
    </ol>
  );
}

/** Marks plain-text search terms in a log line. */
function Highlight({ text, query }: { text: string; query: string }) {
  const terms = query
    .trim()
    .split(/\s+/)
    .filter((t) => t && !/^\w+[=:]/.test(t))
    .map((t) => t.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"));
  if (!terms.length) return <>{text}</>;
  const parts = text.split(new RegExp(`(${terms.join("|")})`, "gi"));
  return <>{parts.map((p, k) => (k % 2 ? <mark key={k}>{p}</mark> : p))}</>;
}
