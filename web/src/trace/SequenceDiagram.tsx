import { useEffect, useMemo, useRef } from "react";
import { formatPayload, type Step, type Trace } from "./model";

const LANE_W = 150;
const ROW_H = 26;
const TOP = 8;
const HEADER_H = 40;

const laneX = (i: number) => i * LANE_W + LANE_W / 2;
const rowY = (i: number) => TOP + i * ROW_H + ROW_H / 2;

function clip(s: string, px: number) {
  const n = Math.max(4, Math.floor(px / 6.6));
  return s.length > n ? s.slice(0, n - 1) + "…" : s;
}

interface Props {
  trace: Trace;
  cursor: number;
  onSelect: (i: number) => void;
}

/** A message sequence chart of a bug trace: one lane per machine or spec. */
export function SequenceDiagram({ trace, cursor, onSelect }: Props) {
  const { lanes, steps } = trace;
  const width = Math.max(lanes.length, 1) * LANE_W;
  const height = TOP * 2 + steps.length * ROW_H;
  const scroller = useRef<HTMLDivElement>(null);

  // Where each lane's lifeline starts: its creation step (monitors: the top).
  const born = useMemo(() => {
    const b: number[] = lanes.map((l) => (l.monitor ? 0 : -1));
    steps.forEach((s, i) => {
      if (s.kind === "create" && s.to !== undefined && b[s.to] < 0) b[s.to] = i;
      if (s.lane !== undefined && b[s.lane] < 0) b[s.lane] = i;
    });
    return b.map((v) => Math.max(v, 0));
  }, [lanes, steps]);

  useEffect(() => {
    const el = scroller.current?.querySelector(`[data-row="${cursor}"]`);
    el?.scrollIntoView({ block: "nearest", inline: "nearest" });
  }, [cursor]);

  return (
    <div className="seq" ref={scroller}>
      <svg className="seq-head" width={width} height={HEADER_H} role="presentation">
        {lanes.map((l, i) => (
          <g key={l.id} transform={`translate(${laneX(i)},${HEADER_H / 2})`}>
            <rect x={-LANE_W / 2 + 6} y={-14} width={LANE_W - 12} height={28} rx={6} className={l.monitor ? "lane-head monitor" : "lane-head"} />
            <text textAnchor="middle" dy="0.35em" className="lane-label">
              <title>{l.monitor ? `spec ${l.id}` : l.id}</title>
              {clip(l.label, LANE_W - 20)}
            </text>
          </g>
        ))}
      </svg>
      <svg width={width} height={height} className="seq-body">
        <defs>
          <marker id="arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
            <path d="M0,0 L10,5 L0,10 z" className="arrowhead" />
          </marker>
          <marker id="arrow-hl" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
            <path d="M0,0 L10,5 L0,10 z" className="arrowhead hl" />
          </marker>
        </defs>
        {lanes.map((l, i) => (
          <line
            key={l.id}
            x1={laneX(i)}
            x2={laneX(i)}
            y1={rowY(born[i]) - ROW_H / 2}
            y2={height}
            className={l.monitor ? "lifeline monitor" : "lifeline"}
          />
        ))}
        {steps.map((s, i) => (
          <Row key={i} i={i} s={s} width={width} cursor={cursor} onSelect={onSelect} />
        ))}
      </svg>
    </div>
  );
}

function Row({ i, s, width, cursor, onSelect }: {
  i: number; s: Step; width: number; cursor: number; onSelect: (i: number) => void;
}) {
  const y = rowY(i);
  const future = i > cursor;
  const selected = i === cursor;
  const linked = s.peer !== undefined && s.peer === cursor;
  const cls = ["row", future && "future", (selected || linked) && "selected", `k-${s.kind}`].filter(Boolean).join(" ");
  const x = s.lane !== undefined ? laneX(s.lane) : 0;
  let body: React.ReactNode = null;

  switch (s.kind) {
    case "create":
      if (s.lane !== undefined && s.to !== undefined && s.lane !== s.to) {
        const x2 = laneX(s.to);
        body = (
          <>
            <line x1={x} y1={y} x2={x2 + (x2 > x ? -8 : 8)} y2={y} className="msg create" markerEnd="url(#arrow)" />
            <text x={(x + x2) / 2} y={y - 4} textAnchor="middle" className="msg-label">new</text>
          </>
        );
      } else if (s.to !== undefined) {
        body = <circle cx={laneX(s.to)} cy={y} r={4} className="dot create" />;
      }
      break;
    case "send": {
      if (s.lane === undefined || s.to === undefined) break;
      const x2 = laneX(s.to);
      const y2 = s.peer !== undefined ? rowY(s.peer) : y;
      const dir = x2 >= x ? 1 : -1;
      const self = s.to === s.lane;
      const text = `${s.label}${s.payload !== undefined ? " " + formatPayload(s.payload) : ""}`;
      body = self ? (
        <>
          <path d={`M${x},${y} h24 V${y2} h-18`} className="msg" fill="none" markerEnd={`url(#${selected || linked ? "arrow-hl" : "arrow"})`} />
          <text x={x + 28} y={y - 4} className="msg-label">{clip(text, 130)}</text>
        </>
      ) : (
        <>
          <line
            x1={x}
            y1={y}
            x2={x2 - dir * 6}
            y2={y2}
            className={s.peer === undefined ? "msg pending" : "msg"}
            markerEnd={`url(#${selected || linked ? "arrow-hl" : "arrow"})`}
          />
          <text
            x={(x + x2) / 2}
            y={(y + y2) / 2 - 5}
            textAnchor="middle"
            className="msg-label"
          >
            {clip(text, Math.max(Math.abs(x2 - x) - 16, 90))}
          </text>
        </>
      );
      break;
    }
    case "dequeue":
      body = <circle cx={x} cy={y} r={3.5} className="dot" />;
      break;
    case "state": {
      const w = Math.min(LANE_W - 16, 16 + s.label.length * 6.6);
      body = (
        <>
          <rect x={x - w / 2} y={y - 9} width={w} height={18} rx={9} className={s.hot ? "state hot" : "state"} />
          <text x={x} y={y} dy="0.35em" textAnchor="middle" className="state-label">{clip(s.label, LANE_W - 24)}</text>
        </>
      );
      break;
    }
    case "monitor": {
      const from = s.to !== undefined ? laneX(s.to) : undefined;
      body = (
        <>
          {from !== undefined && <line x1={from} y1={y} x2={x} y2={y} className="observe" />}
          <path d={`M${x},${y - 6} l6,6 l-6,6 l-6,-6 z`} className="diamond" />
          <text x={x + 10} y={y - 5} className="msg-label">{clip(s.label, LANE_W - 20)}</text>
        </>
      );
      break;
    }
    case "announce":
    case "raise":
    case "halt":
    case "note":
      body = (
        <>
          <rect x={x - 4} y={y - 4} width={8} height={8} className="dot" />
          <text x={x + 9} y={y} dy="0.35em" className="note-label">{clip(s.label, LANE_W - 20)}</text>
        </>
      );
      break;
    case "print":
      body = (
        <text x={8} y={y} dy="0.35em" className="print-label">
          {clip(`print: ${s.label}`, width - 16)}
        </text>
      );
      break;
    case "error":
      body = (
        <>
          <rect x={2} y={y - 11} width={width - 4} height={22} rx={4} className="error-bar" />
          <text x={10} y={y} dy="0.35em" className="error-label">{clip(s.label, width - 20)}</text>
        </>
      );
      break;
  }
  return (
    <g className={cls} data-row={i} onClick={() => onSelect(i)}>
      <rect x={0} y={y - ROW_H / 2} width={width} height={ROW_H} className="row-hit" />
      <title>{s.log}</title>
      {body}
    </g>
  );
}
