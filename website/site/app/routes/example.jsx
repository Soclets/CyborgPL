import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { useLoaderData, useLocation, useNavigate } from "react-router";
import { animate, motion, useDragControls, useMotionValue } from "motion/react";
import Code from "../components/Code.jsx";
import Icon from "../components/Icon.jsx";
import Md from "../components/Md.jsx";
import { plain } from "../lib/md.js";
import { seo } from "../lib/seo.js";

export const meta = ({ loaderData: ex, params }) => seo({
  title: `${ex?.title ?? "Example"} · CyborgPL`,
  description: (ex?.summary && plain(ex.summary)) || `examples/${params.id}.cpls — a CyborgPL program and the output it prints.`,
  path: `/examples/${params.id}`,
});

export async function loader({ params }) {
  const { getExample } = await import("../lib/examples.server.js");
  return getExample(params.id);
}

// The example opens as a bottom sheet over the grid. Swipe it down to put it away
// (tapping the dimmed strip above it, or Escape, also works).
export default function Example() {
  const ex = useLoaderData();
  const navigate = useNavigate();
  const location = useLocation();
  const [tab, setTab] = useState("code");
  // Starts far below the screen, also in the pre-rendered page, so a direct link never
  // shows the window in place before it slides in.
  const y = useMotionValue(10000);
  // The tint's opacity: it fades in with the window and out as the window leaves.
  const tint = useMotionValue(0);
  const drag = useDragControls();

  const closing = useRef(false);
  // Closing is one smooth move for the window and the tint together (owner, 2026-09-28).
  // After a swipe the window carries on at the swipe's own speed, so there is no hitch
  // where the finger let go; from ×, Escape or a click outside it glides, gathering speed.
  const close = (flick = 0) => {
    if (closing.current) return;
    closing.current = true;
    let done = false;
    const leave = () => {
      if (done) return;
      done = true;
      // Opened from inside the app (grid or search): go back there. Opened by link: land on the grid.
      if (location.state?.fromGrid) navigate(-1);
      else navigate("/examples", { replace: true, preventScrollReset: true });
    };
    const to = window.innerHeight + 40;
    const left = Math.max(0, to - y.get());
    const swiped = flick > 300;
    const duration = swiped ? Math.min(0.3, Math.max(0.12, left / flick)) : 0.3;
    animate(tint, 0, { duration, ease: "easeOut" });
    animate(y, to, { type: "tween", duration, ease: swiped ? "linear" : [0.4, 0, 1, 1] }).then(leave);
    // If the browser pauses animations (a background tab), close anyway.
    setTimeout(leave, duration * 1000 + 200);
  };

  // In from the bottom of the screen, from rest. A motion value remembers how fast it last
  // changed, and moving it to the start position counted as a huge flick, so the spring
  // launched upward and came in from the top, by a different amount each time (owner).
  // jump() moves it without a speed, and the spring is told to start still. A layout effect
  // runs before the first paint, so the window never shows in place first.
  useLayoutEffect(() => {
    y.jump(window.innerHeight);
    const a = animate(y, 0, { type: "spring", stiffness: 800, damping: 60, mass: 0.7, velocity: 0 });
    const t = animate(tint, 1, { duration: 0.2, ease: "easeOut" });
    return () => { a.stop(); t.stop(); };
  }, []);

  // Lock the page behind the sheet, and let Escape close it. Hiding the scrollbar would
  // widen the page and shift everything sideways, then back on close (owner: make it
  // smooth); padding as wide as the scrollbar holds the page exactly where it was.
  // A layout effect, so the lock and its padding land in the same frame.
  useLayoutEffect(() => {
    const root = document.documentElement, body = document.body;
    const gap = innerWidth - root.clientWidth;
    const before = [root.style.overflow, body.style.paddingRight];
    root.style.overflow = "hidden";
    if (gap > 0) { body.style.paddingRight = `${gap}px`; root.style.setProperty("--lock-gap", `${gap}px`); }
    const onKey = (e) => e.key === "Escape" && close();
    addEventListener("keydown", onKey);
    return () => { [root.style.overflow, body.style.paddingRight] = before; root.style.removeProperty("--lock-gap"); removeEventListener("keydown", onKey); };
  }, []);

  useEffect(() => setTab("code"), [ex.id]);

  return (
    <div className="sheet-layer" role="dialog" aria-modal="true" aria-label={ex.title}>
      <motion.div className="scrim" onClick={() => close()} style={{ opacity: tint }} />
      <motion.section
        className="sheet"
        style={{ y }}
        drag="y"
        dragListener={false}
        dragControls={drag}
        dragConstraints={{ top: 0, bottom: 0 }}
        dragElastic={{ top: 0, bottom: 0.9 }}
        onDragEnd={(_, info) => { if (info.offset.y > 120 || info.velocity.y > 600) close(info.velocity.y); }}
      >
        {/* Only the top of the sheet drags it, so the code inside still scrolls. */}
        <div className="sheet-grip" onPointerDown={(e) => drag.start(e)}>
          <div className="grabber" />
          <div className="sheet-head">
            <div>
              <b>{ex.title}</b>
              <small>{ex.file} · {ex.lines} lines</small>
            </div>
            {/* On a phone there is no visible close button: swipe down (owner, 2026-09-25), and this
                one is for screen readers only. A computer can't swipe, so there it shows (app.css). */}
            <button className="sheet-close" onClick={() => close()} aria-label="Close"><Icon name="x" size={20} /></button>
          </div>
          {ex.summary && <p className="sheet-summary"><Md text={ex.summary} /></p>}
          <div className="seg" role="tablist">
            <button role="tab" aria-selected={tab === "code"} className={tab === "code" ? "on" : ""} onClick={() => setTab("code")}>Code</button>
            <button role="tab" aria-selected={tab === "output"} className={tab === "output" ? "on" : ""} onClick={() => setTab("output")}>Output</button>
          </div>
        </div>
        <div className="sheet-body">
          {tab === "code" ? (
            <Code html={ex.html} className="sheet-code" />
          ) : (
            <div className="output">
              <pre>{ex.out || "(prints nothing)"}</pre>
              {ex.exit && <p className="exit">Exits with code {ex.exit}.</p>}
              <p className="checked">This is the output the reference interpreter’s tests check for.</p>
            </div>
          )}
        </div>
      </motion.section>
    </div>
  );
}
