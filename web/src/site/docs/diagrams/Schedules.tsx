// Diagram 2: a unit test runs one ordering; the checker runs them all. Each client
// reads the counter and then writes back what it read plus one.
import { useState } from "react";
import { useInterval, useReducedMotion } from "../../motion";

type Step = "A.read" | "A.write" | "B.read" | "B.write";

const UNIT: Step[] = ["A.read", "A.write", "B.read", "B.write"];
const CHECKER: Step[][] = [
  UNIT,
  ["B.read", "B.write", "A.read", "A.write"],
  ["A.read", "B.read", "A.write", "B.write"],
  ["A.read", "B.read", "B.write", "A.write"],
  ["B.read", "A.read", "A.write", "B.write"],
  ["B.read", "A.read", "B.write", "A.write"],
];

interface Snap {
  counter: number;
  a?: number;
  b?: number;
}

/** The state after the first `n` steps of `order`. */
function run(order: Step[], n: number): Snap {
  const s: Snap = { counter: 0 };
  for (const step of order.slice(0, n)) {
    if (step === "A.read") s.a = s.counter;
    if (step === "B.read") s.b = s.counter;
    if (step === "A.write") s.counter = (s.a ?? 0) + 1;
    if (step === "B.write") s.counter = (s.b ?? 0) + 1;
  }
  return s;
}

const finalOf = (order: Step[]) => run(order, 4).counter;
const BUGS = CHECKER.filter((o) => finalOf(o) !== 2).length;

type Mode = "idle" | "unit" | "checker";

export function Schedules() {
  const reduced = useReducedMotion();
  const [mode, setMode] = useState<Mode>("idle");
  // Which row is playing, and how many of its steps have run.
  const [row, setRow] = useState(-1);
  const [step, setStep] = useState(0);
  const [done, setDone] = useState<number>(0); // checker rows finished
  const [playing, setPlaying] = useState(false);

  const rows = mode === "checker" ? CHECKER : [UNIT];

  const tick = () => {
    if (step < 4) return setStep(step + 1);
    // Row finished.
    if (mode === "checker") setDone(row + 1);
    if (mode === "checker" && row + 1 < CHECKER.length) {
      setRow(row + 1);
      setStep(0);
    } else setPlaying(false);
  };
  useInterval(tick, reduced ? 120 : 380, playing);

  const start = (m: Mode) => {
    setMode(m);
    setRow(0);
    setStep(0);
    setDone(0);
    setPlaying(true);
  };

  const order = row >= 0 ? rows[row] : UNIT;
  const snap = row >= 0 ? run(order, step) : { counter: 0 };
  const lastStep = row >= 0 && step > 0 ? order[step - 1] : undefined;
  const finished = !playing && row >= 0;

  return (
    <div className="diagram schedules">
      <div className="sched-controls">
        <button onClick={() => start("unit")} disabled={playing}>Run the unit test</button>
        <button className="primary" onClick={() => start("checker")} disabled={playing}>Run snappea check</button>
      </div>

      <div className="sched-body">
        <div className="sched-state" aria-live="polite">
          <div className={`cell counter${lastStep?.endsWith("write") ? " changed" : ""}`}>
            <span className="cell-label">counter</span>
            <span className="cell-value">{snap.counter}</span>
          </div>
          <div className={`cell lane-a${lastStep === "A.read" ? " changed" : ""}`}>
            <span className="cell-label">A read</span>
            <span className="cell-value">{snap.a ?? "–"}</span>
          </div>
          <div className={`cell lane-b${lastStep === "B.read" ? " changed" : ""}`}>
            <span className="cell-label">B read</span>
            <span className="cell-value">{snap.b ?? "–"}</span>
          </div>
        </div>

        <ol className="sched-rows">
          {rows.map((o, i) => {
            const shown = row >= 0 && (i < row || (i === row && (step > 0 || playing)));
            const complete = mode === "checker" ? i < done : finished && i === 0;
            const final = finalOf(o);
            return (
              <li key={i} className={`sched-row${i === row && playing ? " active" : ""}${shown ? "" : " pending"}`}>
                <span className="row-label">{mode === "checker" ? `schedule ${i + 1}` : "unit test"}</span>
                <span className="chips">
                  {o.map((s, j) => (
                    <span
                      key={j}
                      className={`chip lane-${s[0].toLowerCase()}${i === row && j === step - 1 ? " now" : ""}${
                        i < row || (i === row && j < step) ? " ran" : ""
                      }`}
                    >
                      {s.replace(".", " ")}
                    </span>
                  ))}
                </span>
                <span className={`verdict${complete ? (final === 2 ? " good" : " bad") : ""}`}>
                  {complete ? (final === 2 ? "✓ 2" : `✗ ${final}`) : ""}
                </span>
              </li>
            );
          })}
        </ol>
      </div>

      <p className="diagram-caption" aria-live="polite">
        {mode === "idle" && "Two clients each read the counter, then write back what they read plus one. The answer should be 2."}
        {mode === "unit" && (finished ? "Passes. The test calls client A, then client B, so they never overlap." : "Client A, then client B, one at a time…")}
        {mode === "checker" &&
          (finished
            ? `${BUGS} of ${CHECKER.length} orderings lose an update. The unit test happened to run one of the two that don't.`
            : "The checker tries every ordering of the four steps…")}
      </p>
      <p className="diagram-note">Simplified: each read here includes the counter's reply. The real model also lets each message wait in an inbox, so there are many more orderings.</p>
    </div>
  );
}
