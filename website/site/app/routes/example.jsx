import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { useLoaderData, useLocation, useNavigate } from "react-router";
import { animate, motion, useDragControls, useMotionValue } from "motion/react";
import Code from "../components/Code.jsx";
import Icon from "../components/Icon.jsx";
import { seo } from "../lib/seo.js";

export const meta = ({ loaderData: ex, params }) => seo({
  title: `${ex?.title ?? "Example"} · CyborgPL`,
  description: ex?.summary || `examples/${params.id}.cpls — a CyborgPL program and the output it prints.`,
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
  // On a computer the tint goes down with the window when it closes (owner, 2026-09-28).
  const scrimY = useMotionValue(0);
  const drag = useDragControls();

  const closing = useRef(false);
  const close = () => {
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
    const down = { type: "tween", duration: 0.15, ease: "easeIn" };
    if (document.documentElement.classList.contains("desktop")) animate(scrimY, window.innerHeight, down);
    animate(y, window.innerHeight, down).then(leave);
    // If the browser pauses animations (a background tab), close anyway.
    setTimeout(leave, 260);
  };

  // In from the bottom of the screen, in pixels. Starting from "100%" made Motion convert
  // units by measuring, and the very first open measured wrong and came in from the top.
  // A layout effect runs before the first paint, so the window never shows in place first.
  useLayoutEffect(() => {
    y.set(window.innerHeight);
    const a = animate(y, 0, { type: "spring", stiffness: 800, damping: 60, mass: 0.7 });
    return () => a.stop();
  }, []);

  useEffect(() => {
    // Lock the page behind the sheet, and let Escape close it.
    const root = document.documentElement;
    const before = root.style.overflow;
    root.style.overflow = "hidden";
    const onKey = (e) => e.key === "Escape" && close();
    addEventListener("keydown", onKey);
    return () => { root.style.overflow = before; removeEventListener("keydown", onKey); };
  }, []);

  useEffect(() => setTab("code"), [ex.id]);

  return (
    <div className="sheet-layer" role="dialog" aria-modal="true" aria-label={ex.title}>
      <motion.div className="scrim" onClick={close} style={{ y: scrimY }} initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ duration: 0.12 }} />
      <motion.section
        className="sheet"
        style={{ y }}
        drag="y"
        dragListener={false}
        dragControls={drag}
        dragConstraints={{ top: 0, bottom: 0 }}
        dragElastic={{ top: 0, bottom: 0.9 }}
        onDragEnd={(_, info) => { if (info.offset.y > 120 || info.velocity.y > 600) close(); }}
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
            <button className="sheet-close" onClick={close} aria-label="Close"><Icon name="x" size={20} /></button>
          </div>
          {ex.summary && <p className="sheet-summary">{ex.summary}</p>}
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
