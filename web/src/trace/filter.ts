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
