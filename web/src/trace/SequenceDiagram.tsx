import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { formatPayload, type Step, type Trace } from "./model";

const LANE_W = 150;
const ROW_H = 28;
const TOP = 10;
const HEADER_H = 44;
const MIN_ZOOM = 0.35;
const MAX_ZOOM = 2.5;
/** Machine lanes cycle through this many hues (--lane-0 … in styles.css). */
const HUES = 6;

const laneX = (i: number) => i * LANE_W + LANE_W / 2;
const rowY = (i: number) => TOP + i * ROW_H + ROW_H / 2;
const clamp = (z: number) => Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, z));

function clip(s: string, px: number) {
  const n = Math.max(4, Math.floor(px / 6.6));
  return s.length > n ? s.slice(0, n - 1) + "…" : s;
}

/** A filled arrowhead ending at (x2, y2), pointing away from (x1, y1). */
function head(x1: number, y1: number, x2: number, y2: number, size = 7) {
  const a = Math.atan2(y2 - y1, x2 - x1);
  const p = (da: number) => `${x2 - size * Math.cos(a + da)},${y2 - size * Math.sin(a + da)}`;
  return `M${x2},${y2} L${p(0.42)} L${p(-0.42)} z`;
}

interface Props {
  trace: Trace;
  cursor: number;
  onSelect: (i: number) => void;
}

/** A message sequence chart of a bug trace: one lane per machine or spec. Zoomable. */
export function SequenceDiagram({ trace, cursor, onSelect }: Props) {
  const { lanes, steps } = trace;
  const width = Math.max(lanes.length, 1) * LANE_W;
  const height = TOP * 2 + steps.length * ROW_H;
  const scroller = useRef<HTMLDivElement>(null);
  const [zoom, setZoom] = useState(1);
  // Content point to keep under the screen point after a zoom.
  const anchor = useRef<{ cx: number; cy: number; sx: number; sy: number } | null>(null);

  const color = useMemo(() => {
    let n = 0;
    return lanes.map((l) => (l.monitor ? "var(--spec)" : `var(--lane-${n++ % HUES})`));
  }, [lanes]);

  // Where each lane's lifeline starts: its creation step (monitors: the top).
  const born = useMemo(() => {
    const b: number[] = lanes.map((l) => (l.monitor ? 0 : -1));
    steps.forEach((s, i) => {
      if (s.kind === "create" && s.to !== undefined && b[s.to] < 0) b[s.to] = i;
      if (s.lane !== undefined && b[s.lane] < 0) b[s.lane] = i;
    });
    return b.map((v) => Math.max(v, 0));
  }, [lanes, steps]);

  const fitZoom = useCallback(() => {
    const el = scroller.current;
    return el ? clamp((el.clientWidth - 4) / width) : 1;
  }, [width]);

  const zoomRef = useRef(zoom);

  /**
   * Zooms keeping the content under screen point (sx, sy) (relative to the scroller) in
   * place. Several calls can land before React re-renders (fast pinches, wheel bursts),
   * so the current position is taken from the pending anchor when there is one.
   */
  const zoomTo = useCallback((z: number, sx?: number, sy?: number) => {
    const el = scroller.current;
    if (!el) return;
    const cur = zoomRef.current;
    const next = clamp(z);
    const px = sx ?? el.clientWidth / 2;
    const py = sy ?? el.clientHeight / 2;
    const p = anchor.current;
    const left = p ? p.cx * cur - p.sx : el.scrollLeft;
    const top = p ? p.cy * cur + HEADER_H * cur - p.sy : el.scrollTop;
    anchor.current = { cx: (left + px) / cur, cy: (top + py - HEADER_H * cur) / cur, sx: px, sy: py };
    zoomRef.current = next;
    setZoom(next);
  }, []);

  useLayoutEffect(() => {
    const el = scroller.current;
    const a = anchor.current;
    if (!el || !a) return;
    anchor.current = null;
    el.scrollLeft = a.cx * zoom - a.sx;
    el.scrollTop = a.cy * zoom + HEADER_H * zoom - a.sy;
  }, [zoom]);

  // Start at 100% on wide screens. On phones shrink toward fitting the width, but not
  // so far that labels become unreadable; pinch or the buttons do the rest.
  useLayoutEffect(() => {
    const narrow = (scroller.current?.clientWidth ?? 1000) < 600;
    const z = narrow ? Math.min(1, Math.max(fitZoom(), 0.8)) : 1;
    zoomRef.current = z;
    setZoom(z);
  }, [trace, fitZoom]);

  // Ctrl/⌘ + wheel (and trackpad pinch, which browsers report the same way).
  useEffect(() => {
    const el = scroller.current;
    if (!el) return;
    const onWheel = (e: WheelEvent) => {
      if (!e.ctrlKey && !e.metaKey) return;
      e.preventDefault();
      const r = el.getBoundingClientRect();
      const f = Math.exp(-Math.max(-50, Math.min(50, e.deltaY)) * 0.006);
      zoomTo(zoomRef.current * f, e.clientX - r.left, e.clientY - r.top);
    };
    el.addEventListener("wheel", onWheel, { passive: false });
    return () => el.removeEventListener("wheel", onWheel);
  }, [zoomTo]);

  // Two-finger pinch. One finger still scrolls natively (touch-action: pan-x pan-y).
  const pointers = useRef(new Map<number, { x: number; y: number }>());
  const pinch = useRef<{ d: number; z: number } | null>(null);
  const dist = () => {
    const [a, b] = [...pointers.current.values()];
    return Math.hypot(a.x - b.x, a.y - b.y);
  };
  const onPointerDown = (e: React.PointerEvent) => {
    if (e.pointerType !== "touch") return;
    pointers.current.set(e.pointerId, { x: e.clientX, y: e.clientY });
    if (pointers.current.size === 2) pinch.current = { d: dist(), z: zoomRef.current };
  };
  const onPointerMove = (e: React.PointerEvent) => {
    if (!pointers.current.has(e.pointerId)) return;
    pointers.current.set(e.pointerId, { x: e.clientX, y: e.clientY });
    const p = pinch.current;
    const el = scroller.current;
    if (!p || pointers.current.size !== 2 || !el) return;
    const [a, b] = [...pointers.current.values()];
    const r = el.getBoundingClientRect();
    zoomTo(p.z * (dist() / p.d), (a.x + b.x) / 2 - r.left, (a.y + b.y) / 2 - r.top);
  };
  const onPointerUp = (e: React.PointerEvent) => {
    pointers.current.delete(e.pointerId);
    if (pointers.current.size < 2) pinch.current = null;
  };

  // Keep the cursor row visible, scrolling only the diagram (never the page).
  useEffect(() => {
    const el = scroller.current;
    if (!el) return;
    const head = HEADER_H * zoomRef.current;
    const top = head + (rowY(cursor) - ROW_H) * zoomRef.current;
    const bottom = head + (rowY(cursor) + ROW_H) * zoomRef.current;
    if (top < el.scrollTop + head) el.scrollTop = top - head;
    else if (bottom > el.scrollTop + el.clientHeight) el.scrollTop = bottom - el.clientHeight;
  }, [cursor]);

  return (
    <div className="seq-wrap">
      <div
        className="seq"
        ref={scroller}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onPointerCancel={onPointerUp}
      >
        <svg
          className="seq-head"
          width={width * zoom}
          height={HEADER_H * zoom}
          viewBox={`0 0 ${width} ${HEADER_H}`}
          role="presentation"
        >
          {lanes.map((l, i) => (
            <g key={l.id} transform={`translate(${laneX(i)},${HEADER_H / 2})`} style={{ "--c": color[i] } as React.CSSProperties}>
              <rect
                x={-LANE_W / 2 + 8}
                y={-15}
                width={LANE_W - 16}
                height={30}
                rx={15}
                className={l.monitor ? "lane-head monitor" : "lane-head"}
              />
              <LaneLabel id={l.id} monitor={l.monitor} />
            </g>
          ))}
        </svg>
        <svg width={width * zoom} height={height * zoom} viewBox={`0 0 ${width} ${height}`} className="seq-body">
          {lanes.map((l, i) => (
            <line
              key={l.id}
              x1={laneX(i)}
              x2={laneX(i)}
              y1={rowY(born[i]) - ROW_H / 2}
              y2={height}
              className={l.monitor ? "lifeline monitor" : "lifeline"}
              style={{ "--c": color[i] } as React.CSSProperties}
            />
          ))}
          {steps.map((s, i) => (
            <Row key={i} i={i} s={s} width={width} cursor={cursor} color={color} onSelect={onSelect} />
          ))}
        </svg>
      </div>
      <div className="zoom-controls" role="group" aria-label="Zoom">
        <button onClick={() => zoomTo(zoom / 1.25)} aria-label="Zoom out" disabled={zoom <= MIN_ZOOM}>
          −
        </button>
        <button className="zoom-level" onClick={() => zoomTo(zoom === 1 ? fitZoom() : 1)} title="Toggle 100% / fit width">
          {Math.round(zoom * 100)}%
        </button>
        <button onClick={() => zoomTo(zoom * 1.25)} aria-label="Zoom in" disabled={zoom >= MAX_ZOOM}>
          +
        </button>
      </div>
    </div>
  );
}

function Row({ i, s, width, cursor, color, onSelect }: {
  i: number; s: Step; width: number; cursor: number; color: string[]; onSelect: (i: number) => void;
}) {
  const y = rowY(i);
  const future = i > cursor;
  const selected = i === cursor;
  const linked = s.peer !== undefined && s.peer === cursor;
  const cls = ["row", future && "future", (selected || linked) && "selected", `k-${s.kind}`].filter(Boolean).join(" ");
  const x = s.lane !== undefined ? laneX(s.lane) : 0;
  const c = s.lane !== undefined ? color[s.lane] : "var(--fg)";
  let body: React.ReactNode = null;

  switch (s.kind) {
    case "create":
      if (s.lane !== undefined && s.to !== undefined && s.lane !== s.to) {
        const x2 = laneX(s.to);
        const end = x2 + (x2 > x ? -6 : 6);
        body = (
          <>
            <line x1={x} y1={y} x2={end} y2={y} className="msg create" />
            <path d={head(x, y, end, y)} className="arrowhead create" />
            <text x={(x + x2) / 2} y={y - 5} textAnchor="middle" className="msg-label muted">new</text>
          </>
        );
      } else if (s.to !== undefined) {
        body = <circle cx={laneX(s.to)} cy={y} r={4} className="dot" style={{ fill: color[s.to] }} />;
      }
      break;
    case "send": {
      if (s.lane === undefined || s.to === undefined) break;
      const x2 = laneX(s.to);
      const y2 = s.peer !== undefined ? rowY(s.peer) : y;
      const dir = x2 >= x ? 1 : -1;
      const self = s.to === s.lane;
      body = self ? (
        <>
          <path d={`M${x},${y} h18 a8,8 0 0 1 8,8 V${y2 - 8} a8,8 0 0 1 -8,8 h-12`} className="msg" fill="none" />
          <path d={head(x + 18, y2, x + 6, y2)} className="arrowhead" />
          <MsgLabel x={x + 32} y={y - 4} anchor="start" event={s.label} payload={s.payload} max={130} />
        </>
      ) : (
        <>
          <line x1={x} y1={y} x2={x2 - dir * 5} y2={y2} className={s.peer === undefined ? "msg pending" : "msg"} />
          <path d={head(x, y, x2 - dir * 5, y2)} className="arrowhead" />
          <MsgLabel x={(x + x2) / 2} y={(y + y2) / 2 - 7} anchor="middle" event={s.label} payload={s.payload} max={Math.max(Math.abs(x2 - x) - 16, 90)} />
        </>
      );
      break;
    }
    case "dequeue":
      body = <circle cx={x} cy={y} r={4} className="dot" />;
      break;
    case "state": {
      const w = Math.min(LANE_W - 16, 18 + s.label.length * 6.6);
      body = (
        <>
          <rect x={x - w / 2} y={y - 10} width={w} height={20} rx={10} className={s.hot ? "state hot" : "state"} />
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
          <rect x={x - 5} y={y - 5} width={10} height={10} rx={2} transform={`rotate(45 ${x} ${y})`} className="diamond" />
          <text x={x + 11} y={y - 6} className="msg-label">{clip(s.label, LANE_W - 20)}</text>
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
          <rect x={x - 4} y={y - 4} width={8} height={8} rx={2} className="dot" />
          <text x={x + 9} y={y} dy="0.35em" className="note-label">{clip(s.label, LANE_W - 20)}</text>
        </>
      );
      break;
    case "print":
      body = (
        <text x={10} y={y} dy="0.35em" className="print-label">
          {clip(`print: ${s.label}`, width - 20)}
        </text>
      );
      break;
    case "error":
      body = (
        <>
          <rect x={4} y={y - 12} width={Math.min(width - 8, 24 + shortError(s.label).length * 7.2)} height={24} rx={12} className="error-bar" />
          <text x={16} y={y} dy="0.35em" className="error-label">{shortError(s.label)}</text>
        </>
      );
      break;
  }
  return (
    <g className={cls} data-row={i} onClick={() => onSelect(i)} style={{ "--c": c } as React.CSSProperties}>
      <rect x={0} y={y - ROW_H / 2} width={width} height={ROW_H} className="row-hit" />
      {selected && <rect x={0} y={y - ROW_H / 2 + 3} width={3} height={ROW_H - 6} rx={1.5} className="row-cursor" />}
      <title>{s.log}</title>
      {body}
    </g>
  );
}

/** "Assertion Failed: file.p:1:2 message" -> "✕ Assertion Failed"; the full text is in the banner. */
function shortError(e: string) {
  const m = /^([^:.]+)/.exec(e);
  return `✕ ${(m?.[1] ?? e).trim()}`;
}

/** "Client(3)" -> name "Client" and a muted "(3)". */
function LaneLabel({ id, monitor }: { id: string; monitor: boolean }) {
  const m = /^(.*?)(\(\d+\))?$/.exec(id)!;
  const n = m[2] ?? "";
  return (
    <text textAnchor="middle" dy="0.35em" className="lane-label">
      <title>{monitor ? `spec ${id}` : id}</title>
      {clip(m[1], LANE_W - 28 - n.length * 7)}
      {n && <tspan className="lane-id">{n}</tspan>}
    </text>
  );
}

/** Event name plus a muted payload, on a soft background so lines stay visible. */
function MsgLabel({ x, y, anchor, event, payload, max }: {
  x: number; y: number; anchor: "start" | "middle"; event: string; payload: unknown; max: number;
}) {
  const ev = clip(event, max);
  const room = max - ev.length * 6.6 - 6;
  const pl = payload !== undefined && room > 30 ? clip(formatPayload(payload), room) : "";
  const w = (ev.length + (pl ? pl.length + 1 : 0)) * 6.6 + 8;
  const left = anchor === "middle" ? x - w / 2 : x - 4;
  return (
    <>
      <rect x={left} y={y - 9} width={w} height={13} rx={4} className="label-bg" />
      <text x={x} y={y} textAnchor={anchor} className="msg-label">
        <tspan className="ev">{ev}</tspan>
        {pl && <tspan className="payload"> {pl}</tspan>}
      </text>
    </>
  );
}
