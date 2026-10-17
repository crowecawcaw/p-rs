import type { Step, Trace } from "./model";

/** Lanes a step touches: where it happens, its target, and the other end of a message. */
export function involved(steps: Step[], i: number): number[] {
  const s = steps[i];
  const peer = s.peer !== undefined ? steps[s.peer]?.lane : undefined;
  return [s.lane, s.to, peer].filter((l): l is number => l !== undefined);
}

/**
 * Steps and lanes to show when focusing on some lanes: every step that touches a
 * focused lane, plus prints and the bug; and the lanes those steps touch.
 */
export function visible(trace: Trace, focus: ReadonlySet<number>): { rows: number[]; cols: number[] } {
  const all = trace.steps.map((_, i) => i);
  if (!focus.size) return { rows: all, cols: trace.lanes.map((_, i) => i) };
  const rows = all.filter((i) => {
    const s = trace.steps[i];
    return s.kind === "error" || s.kind === "print" || involved(trace.steps, i).some((l) => focus.has(l));
  });
  const used = new Set(focus);
  for (const i of rows) for (const l of involved(trace.steps, i)) used.add(l);
  return { rows, cols: trace.lanes.map((_, i) => i).filter((l) => used.has(l)) };
}

/**
 * Steps matching a search. Plain text matches anywhere in the step (case-insensitive);
 * `field=value` matches a step's machine, event, state or kind; terms separated by
 * spaces must all match.
 */
export function search(trace: Trace, rows: number[], query: string): Set<number> | null {
  const terms = query.trim().toLowerCase().split(/\s+/).filter(Boolean);
  if (!terms.length) return null;
  const hay = (i: number) => {
    const s = trace.steps[i];
    return `${s.log} ${s.label} ${s.payload === undefined ? "" : JSON.stringify(s.payload)}`.toLowerCase();
  };
  const field = (i: number, k: string, v: string) => {
    const s = trace.steps[i];
    const lane = (l?: number) => (l === undefined ? "" : trace.lanes[l].id.toLowerCase());
    const has = (x: string) => x.includes(v);
    switch (k) {
      case "machine":
      case "m":
        return involved(trace.steps, i).some((l) => has(lane(l)));
      case "from":
        return has(lane(s.lane));
      case "to":
        return has(lane(s.to));
      case "event":
      case "e":
        return (s.kind === "send" || s.kind === "dequeue" || s.kind === "monitor") && has(s.label.toLowerCase());
      case "state":
        return s.kind === "state" && has(s.label.toLowerCase());
      case "kind":
        return s.kind.startsWith(v);
      default:
        return hay(i).includes(`${k}=${v}`);
    }
  };
  const ok = (i: number) =>
    terms.every((t) => {
      const m = /^(\w+)[=:](.+)$/.exec(t);
      return m ? field(i, m[1], m[2]) : hay(i).includes(t);
    });
  return new Set(rows.filter(ok));
}
