// Diagram 3: the checker's bug trace for the lost-update model, as a sequence
// diagram you can step through. Scripted from a real `snappea check` trace, trimmed
// to the messages that matter.
import { useEffect, useState } from "react";
import { useInView, useInterval, useReducedMotion } from "../../motion";

const LANES = [
  { id: "A", label: "Client A", cls: "lane-a" },
  { id: "C", label: "Counter", cls: "lane-c" },
  { id: "B", label: "Client B", cls: "lane-b" },
  { id: "M", label: "Main (test)", cls: "lane-m" },
] as const;
type Lane = (typeof LANES)[number]["id"];

interface Step {
  from?: Lane;
  to?: Lane;
  label?: string;
  text: string;
  state: { value: number; a?: number; b?: number; final?: number };
  /** Which value changed, to highlight it. */
  changed?: "value" | "a" | "b" | "final";
  bug?: boolean;
}

const STEPS: Step[] = [
  { text: "Main creates the Counter and two Clients. The counter starts at 0.", state: { value: 0 } },
  { from: "A", to: "C", label: "eRead", text: "Client A asks the Counter for its value.", state: { value: 0 } },
  { from: "C", to: "A", label: "eValue(0)", text: "The Counter replies 0. A will write 0 + 1.", state: { value: 0, a: 0 }, changed: "a" },
  { from: "B", to: "C", label: "eRead", text: "The checker lets Client B ask before A's write arrives.", state: { value: 0, a: 0 } },
  { from: "C", to: "B", label: "eValue(0)", text: "B also gets 0.", state: { value: 0, a: 0, b: 0 }, changed: "b" },
  { from: "A", to: "C", label: "eWrite(1)", text: "A writes 1.", state: { value: 1, a: 0, b: 0 }, changed: "value" },
  { from: "B", to: "C", label: "eWrite(1)", text: "B also writes 1, overwriting A's update.", state: { value: 1, a: 0, b: 0 }, changed: "value" },
  { from: "M", to: "C", label: "eGet", text: "Both clients are done, so Main asks for the final value.", state: { value: 1, a: 0, b: 0 } },
  { from: "C", to: "M", label: "eValue(1)", text: "The Counter replies 1.", state: { value: 1, a: 0, b: 0, final: 1 }, changed: "final" },
  {
    text: "assert final == 2 fails: “expected 2 increments, counter is 1”. snappea check saves this schedule so you can replay it.",
    state: { value: 1, a: 0, b: 0, final: 1 },
    bug: true,
  },
];

const W = 640;
const LANE_X = (i: number) => 80 + i * ((W - 160) / 3);
const laneX = (id: Lane) => LANE_X(LANES.findIndex((l) => l.id === id));
const TOP = 54;
const ROW = 32;
const H = TOP + ROW * (STEPS.length - 1) + 24;

export function Trace() {
  const [ref, inView] = useInView<HTMLDivElement>();
  const reduced = useReducedMotion();
  const [i, setI] = useState(0);
  const [playing, setPlaying] = useState(false);
  const [started, setStarted] = useState(false);

  // Play once, the first time it scrolls into view.
  useEffect(() => {
    if (inView && !started && !reduced) {
      setStarted(true);
      setPlaying(true);
    }
  }, [inView, started, reduced]);

  useInterval(
    () => {
      if (i + 1 >= STEPS.length) setPlaying(false);
      else setI(i + 1);
    },
    1400,
    playing && inView,
  );

  const go = (n: number) => {
    setPlaying(false);
    setI(Math.max(0, Math.min(STEPS.length - 1, n)));
  };
  const toggle = () => {
    if (!playing && i === STEPS.length - 1) setI(0);
    setPlaying(!playing);
  };

  const cur = STEPS[i];

  return (
    <div className="diagram tracedemo" ref={ref}>
      <div className="tracedemo-body">
        <svg viewBox={`0 0 ${W} ${H}`} role="img" aria-label="Sequence diagram of the lost-update bug">
          <defs>
            <marker id="trace-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto">
              <path d="M0,0 L10,5 L0,10 z" fill="context-stroke" />
            </marker>
          </defs>
          {LANES.map((l, k) => (
            <g key={l.id} className={`sq-lane ${l.cls}`}>
              <text x={LANE_X(k)} y={20} textAnchor="middle" className="sq-lane-label">{l.label}</text>
              <line x1={LANE_X(k)} x2={LANE_X(k)} y1={30} y2={H - 8} />
            </g>
          ))}
          {STEPS.map((s, k) => {
            if (!s.from || !s.to || k > i) return null;
            const y = TOP + (k - 1) * ROW;
            const x1 = laneX(s.from);
            const x2 = laneX(s.to);
            const dir = Math.sign(x2 - x1);
            const fromCls = LANES.find((l) => l.id === s.from)!.cls;
            const lost = k === 6;
            return (
              <g key={`${k}-${k === i ? "now" : "old"}`} className={`sq-msg ${fromCls}${k === i ? " now" : ""}${lost ? " lost" : ""}`}>
                <line x1={x1 + dir * 4} x2={x2 - dir * 6} y1={y} y2={y} markerEnd="url(#trace-arrow)" pathLength={1} />
                <text x={(x1 + x2) / 2} y={y - 7} textAnchor="middle" className="sq-msg-label">{s.label}</text>
              </g>
            );
          })}
          {cur.bug && (
            <g className="assert-fail">
              <circle cx={laneX("M")} cy={TOP + (STEPS.length - 2) * ROW} r={12} />
              <text x={laneX("M")} y={TOP + (STEPS.length - 2) * ROW + 5} textAnchor="middle">✗</text>
            </g>
          )}
        </svg>

        <div className="tracedemo-state" aria-label="Values after this step">
          <Cell label="counter" value={cur.state.value} on={cur.changed === "value"} cls="lane-c" />
          <Cell label="A read" value={cur.state.a} on={cur.changed === "a"} cls="lane-a" />
          <Cell label="B read" value={cur.state.b} on={cur.changed === "b"} cls="lane-b" />
          <Cell label="final" value={cur.state.final} on={cur.changed === "final"} cls="lane-m" bad={cur.bug} />
        </div>
      </div>

      <p className={`diagram-caption${cur.bug ? " bad" : ""}`} aria-live="polite">
        <span className="step-no">{i + 1}/{STEPS.length}</span> {cur.text}
      </p>

      <div className="player">
        <button onClick={() => go(0)} aria-label="First step" disabled={i === 0}>⏮</button>
        <button onClick={() => go(i - 1)} aria-label="Previous step" disabled={i === 0}>◀</button>
        <button className="primary" onClick={toggle} aria-label={playing ? "Pause" : "Play"}>{playing ? "❚❚" : "▶"}</button>
        <button onClick={() => go(i + 1)} aria-label="Next step" disabled={i === STEPS.length - 1}>▶|</button>
        <input type="range" min={0} max={STEPS.length - 1} value={i} onChange={(e) => go(+e.target.value)} aria-label="Step" />
      </div>
    </div>
  );
}

function Cell(props: { label: string; value?: number; on?: boolean; cls: string; bad?: boolean }) {
  const { label, value, on, cls, bad } = props;
  return (
    <div className={`cell ${cls}${on ? " changed" : ""}${bad ? " bad" : ""}`}>
      <span className="cell-label">{label}</span>
      <span className="cell-value">{value ?? "–"}</span>
    </div>
  );
}
