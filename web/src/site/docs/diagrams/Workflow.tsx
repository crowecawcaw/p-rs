// Diagram 4: where P fits, from design to production. A highlight walks the steps,
// including one trip around the "bug found, fix the design" loop.
import { useState } from "react";
import { useInView, useInterval, useReducedMotion } from "../../motion";

const STEPS = [
  { title: "Design", who: "you", text: "Write or update the design doc: components, messages, failure cases." },
  { title: "Model", who: "agent", text: "An agent writes the P model and specs from the design. You review the specs." },
  { title: "Check", who: "prs check", text: "The checker explores orderings and failures. A bug sends you back to the design." },
  { title: "Build", who: "you + agent", text: "Implement the service. Emit events where the spec needs them." },
  { title: "Test", who: "CI", text: "Ordinary tests run, and the same specs check the events your code emits." },
  { title: "Observe", who: "prs observe", text: "Optionally, check production logs against the specs." },
];

// The walk: design, model, check, a bug back to model, check again, then on.
const WALK: { step: number; loop?: boolean }[] = [
  { step: 0 }, { step: 1 }, { step: 2 }, { step: 1, loop: true }, { step: 2 }, { step: 3 }, { step: 4 }, { step: 5 },
];

export function Workflow() {
  const [ref, inView] = useInView<HTMLDivElement>();
  const reduced = useReducedMotion();
  const [w, setW] = useState(0);
  const [picked, setPicked] = useState<number>();

  useInterval(() => setW((x) => (x + 1) % WALK.length), 1700, inView && picked === undefined && !reduced);

  const active = picked ?? WALK[w].step;
  const loop = picked === undefined && WALK[w].loop;

  return (
    <div className="diagram workflow" ref={ref}>
      <ol className="flow">
        {STEPS.map((s, i) => (
          <li key={s.title} className={`flow-step${i === active ? " active" : ""}${i === 1 || i === 2 ? " p-step" : ""}`}>
            <button onClick={() => setPicked(picked === i ? undefined : i)} aria-pressed={picked === i}>
              <span className="flow-no">{i + 1}</span>
              <span className="flow-title">{s.title}</span>
              <span className="flow-who">{s.who}</span>
            </button>
          </li>
        ))}
      </ol>
      <div className={`flow-loop${loop ? " on" : ""}`} aria-hidden="true">
        <span>↺ bug found: change the design, update the model, check again</span>
      </div>
      <p className="diagram-caption" aria-live="polite">
        <strong>{STEPS[active].title}.</strong> {loop ? "The checker found a bug, so the design changes and the model with it." : STEPS[active].text}
      </p>
    </div>
  );
}
