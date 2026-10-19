import { AnimatePresence, motion } from "motion/react";
import { useEffect, useRef, useState } from "react";
import { Cyborg, PARTS } from "./cyborg";
import { preloadSite } from "./preload";

// The loading screen (owner, 2026-10-19: "Preloads everything, and is a real loading screen. A
// progress bar out of 100%, and a Cyborg getting assembled. Each 10% = one Cyborg part. Black and
// white"). It shows on every full load, never when moving between pages.
//
// The bar never runs ahead of what has really arrived. It may trail it: it fills at most RATE of
// the way a second, so on a fast connection the assembly can still be seen part by part.
const RATE = 0.7;
const GIVE_UP_MS = 15000;
const HOLD_MS = 450; // at 100%, with the visor lit, before the screen fades

export function LoadingScreen() {
  const [shown, setShown] = useState(0);
  const [phase, setPhase] = useState<"loading" | "online" | "gone">("loading");
  const real = useRef(0);

  useEffect(() => {
    const root = document.documentElement;
    if (!root.classList.contains("loading")) { setPhase("gone"); return; }
    // In development, ?loading=40 holds the screen at 40% to look at it.
    const hold = import.meta.env.DEV ? new URLSearchParams(location.search).get("loading") : null;
    if (hold !== null) { real.current = Number(hold) / 100; setShown(real.current); return; }

    preloadSite((f) => { real.current = Math.max(real.current, f); });
    const giveUp = setTimeout(() => { real.current = 1; }, GIVE_UP_MS);
    // Animation frames drive the bar while the page is watched. A tab in the background gets
    // none, so there a timer steps it instead, and nobody returns to a screen stuck halfway.
    let last = performance.now(), value = 0, frame = 0, timer: ReturnType<typeof setTimeout> | undefined;
    const tick = (now: number) => {
      value = Math.min(real.current, value + ((now - last) / 1000) * RATE);
      last = now;
      setShown(value);
      if (value >= 1) { setPhase("online"); return; }
      if (document.hidden) timer = setTimeout(() => tick(performance.now()), 100);
      else frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => { cancelAnimationFrame(frame); clearTimeout(timer); clearTimeout(giveUp); };
  }, []);

  useEffect(() => {
    if (phase !== "online") return;
    const t = setTimeout(() => {
      // .ready lets the page's own entrance start as the screen fades (app.css).
      document.documentElement.classList.add("ready");
      setPhase("gone");
    }, HOLD_MS);
    return () => clearTimeout(t);
  }, [phase]);

  const percent = Math.floor(shown * 100);
  const done = Math.min(10, Math.floor(shown * 10));
  const status = done < 10 ? `Attaching ${PARTS[done]!.name}` : "Online.";

  return (
    <AnimatePresence onExitComplete={() => document.documentElement.classList.remove("loading")}>
      {phase !== "gone" && (
        <motion.div
          className="loader"
          exit={{ opacity: 0 }}
          transition={{ duration: 0.5, ease: "easeOut" }}
          role="progressbar"
          aria-label="Loading CyborgPL"
          aria-valuemin={0}
          aria-valuemax={100}
          aria-valuenow={percent}
        >
          <Cyborg done={done} />
          <div className="loader-bar" aria-hidden="true">
            <span style={{ transform: `scaleX(${shown})` }} />
          </div>
          <p className="loader-percent">{percent}%</p>
          <p className="loader-status">{status}</p>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
