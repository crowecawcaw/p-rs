import { useEffect, useMemo, useState } from "react";
import { parseTrace } from "./model";
import { SequenceDiagram } from "./SequenceDiagram";

interface Props {
  json: string;
  onOpenLocation: (file: string, line: number) => void;
}

const LOC = /([\w./-]+\.p):(\d+):(\d+)/;

/** Bug trace: step-through controls, the sequence diagram, and the selected step's details. */
export function TraceView({ json, onOpenLocation }: Props) {
  const trace = useMemo(() => parseTrace(json), [json]);
  const last = trace.steps.length - 1;
  const [cursor, setCursor] = useState(last);
  const [playing, setPlaying] = useState(false);
  const [bannerOpen, setBannerOpen] = useState(false);

  useEffect(() => {
    setCursor(last);
    setPlaying(false);
  }, [json, last]);

  useEffect(() => {
    if (!playing) return;
    if (cursor >= last) {
      setPlaying(false);
      return;
    }
    const t = setTimeout(() => setCursor((c) => Math.min(c + 1, last)), 220);
    return () => clearTimeout(t);
  }, [playing, cursor, last]);

  const step = trace.steps[cursor];
  const loc = trace.error ? LOC.exec(trace.error) : null;

  return (
    <div className="trace">
      {trace.error && (
        <div className={bannerOpen ? "trace-error open" : "trace-error"} onClick={() => setBannerOpen(!bannerOpen)}>
          <strong>Bug:</strong> {trace.error}
          {loc && (
            <button
              className="link"
              onClick={(e) => {
                e.stopPropagation();
                onOpenLocation(loc[1], Number(loc[2]));
              }}
            >
              Go to {loc[1]}:{loc[2]}
            </button>
          )}
        </div>
      )}
      <div className="trace-controls">
        <button onClick={() => { setPlaying(false); setCursor(0); }} title="First step" aria-label="First step" className="jump">⏮</button>
        <button onClick={() => setCursor((c) => Math.max(c - 1, 0))} title="Previous step" aria-label="Previous step">◀</button>
        <button
          className="primary"
          onClick={() => {
            if (!playing && cursor >= last) setCursor(0);
            setPlaying(!playing);
          }}
          title={playing ? "Pause" : "Play"}
        >
          {playing ? "Pause" : "Play"}
        </button>
        <button onClick={() => setCursor((c) => Math.min(c + 1, last))} title="Next step" aria-label="Next step">▶</button>
        <button onClick={() => { setPlaying(false); setCursor(last); }} title="Last step" aria-label="Last step" className="jump">⏭</button>
        <input
          type="range"
          min={0}
          max={Math.max(last, 0)}
          value={cursor}
          onChange={(e) => { setPlaying(false); setCursor(Number(e.target.value)); }}
          aria-label="Step"
        />
        <span className="muted">
          {cursor + 1}/{trace.steps.length}
        </span>
      </div>
      <SequenceDiagram trace={trace} cursor={cursor} onSelect={(i) => { setPlaying(false); setCursor(i); }} />
      {step && step.kind !== "error" && (
        <div className="step-detail">
          <div>{step.log}</div>
          {step.payload !== undefined && <pre>{JSON.stringify(step.payload, null, 2)}</pre>}
        </div>
      )}
    </div>
  );
}
