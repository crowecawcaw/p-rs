import { useEffect, useRef, useState } from "react";

/** True when the user asked the OS for less motion. */
export function useReducedMotion(): boolean {
  const q = "(prefers-reduced-motion: reduce)";
  const [reduced, setReduced] = useState(false);
  useEffect(() => {
    const mq = matchMedia(q);
    setReduced(mq.matches);
    const on = () => setReduced(mq.matches);
    mq.addEventListener("change", on);
    return () => mq.removeEventListener("change", on);
  }, []);
  return reduced;
}

/** A ref and whether its element is on screen, so diagrams animate only when seen. */
export function useInView<T extends Element>() {
  const ref = useRef<T>(null);
  const [inView, setInView] = useState(false);
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const io = new IntersectionObserver(([e]) => setInView(e.isIntersecting), { threshold: 0.25 });
    io.observe(el);
    return () => io.disconnect();
  }, []);
  return [ref, inView] as const;
}

/** Calls `tick` every `ms` while `on`. */
export function useInterval(tick: () => void, ms: number, on: boolean) {
  const saved = useRef(tick);
  saved.current = tick;
  useEffect(() => {
    if (!on) return;
    const id = setInterval(() => saved.current(), ms);
    return () => clearInterval(id);
  }, [ms, on]);
}

/** Seconds elapsed while `on`, advanced every animation frame; pauses keep their place. */
export function useClock(on: boolean): number {
  const [t, setT] = useState(0);
  useEffect(() => {
    if (!on) return;
    let raf = 0;
    let prev = performance.now();
    const frame = (now: number) => {
      // Cap the step so a background tab doesn't jump the animation ahead.
      setT((x) => x + Math.min(now - prev, 100) / 1000);
      prev = now;
      raf = requestAnimationFrame(frame);
    };
    raf = requestAnimationFrame(frame);
    return () => cancelAnimationFrame(raf);
  }, [on]);
  return t;
}
