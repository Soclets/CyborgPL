// The simulation behind "Merging with the machine." (owner, 2026-10-20: "I want a real simulation
// in the background of `Merging with the machine`", then "let's do a neural network growing into a
// circuit board", "not interactable", in TypeScript, "make sure it's easily removeable").
//
// Everything it needs is in this folder. To take it out, delete app/hero-sim/ and the two lines
// that mention HeroSim in app/routes/home.tsx.
//
// It starts once the loading screen fades, runs only while the hero is on screen and the tab is
// visible, and draws one still frame for anyone who asks for reduced motion. When a network has
// fully turned into a board, it fades and a new one grows.

import { useEffect, useRef } from "react";
import { CONVERT_AT, Sim } from "./sim";
import { draw } from "./draw";
import "./hero-sim.css";

// Real ms per model ms: the network runs in slow motion, so a spike can be seen crossing an axon.
const SLOW = 12;
const STILL_AT = CONVERT_AT + 1300;

export function HeroSim() {
  const ref = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = ref.current, ctx = canvas?.getContext("2d");
    if (!canvas || !ctx) return;
    const root = document.documentElement;
    const still = matchMedia("(prefers-reduced-motion: reduce)").matches;
    let sim: Sim | null = null, w = 0, h = 0;
    let started = false, onScreen = true, raf = 0, last = 0, owed = 0;

    const fresh = () => {
      sim = new Sim(w, h, (Math.random() * 2 ** 32) >>> 0);
      if (still) sim.runTo(STILL_AT);
    };
    const paint = () => { if (sim && started) draw(ctx, sim, w, h); };

    const fit = () => {
      const r = canvas.getBoundingClientRect(), dpr = Math.min(2, devicePixelRatio || 1);
      canvas.width = Math.round(r.width * dpr);
      canvas.height = Math.round(r.height * dpr);
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      const nw = Math.round(r.width);
      h = r.height;
      if (nw !== w) { w = nw; fresh(); }
      paint();
    };

    const running = () => started && onScreen && !document.hidden && !still;
    const frame = (now: number) => {
      raf = 0;
      if (!running() || !sim) return;
      owed += Math.min(50, now - last) / SLOW;
      last = now;
      for (; owed >= 1; owed--) sim.step();
      if (sim.done) fresh();
      paint();
      raf = requestAnimationFrame(frame);
    };
    const wake = () => {
      if (raf || !running()) return;
      last = performance.now();
      raf = requestAnimationFrame(frame);
    };

    const sized = new ResizeObserver(fit);
    sized.observe(canvas);
    const seen = new IntersectionObserver(([entry]) => { onScreen = !!entry?.isIntersecting; wake(); });
    seen.observe(canvas);
    document.addEventListener("visibilitychange", wake);

    const start = () => { started = true; canvas.classList.add("is-on"); paint(); wake(); };
    const ready = () => root.classList.contains("ready") || !root.classList.contains("loading");
    const waiting = new MutationObserver(() => { if (ready()) { waiting.disconnect(); start(); } });
    if (ready()) start(); else waiting.observe(root, { attributes: true, attributeFilter: ["class"] });

    return () => {
      cancelAnimationFrame(raf);
      sized.disconnect(); seen.disconnect(); waiting.disconnect();
      document.removeEventListener("visibilitychange", wake);
    };
  }, []);

  return <canvas ref={ref} className="hero-sim" aria-hidden="true" />;
}
