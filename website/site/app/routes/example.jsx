import { useEffect, useRef, useState } from "react";
import { useLoaderData, useLocation, useNavigate } from "react-router";
import { animate, motion, useDragControls, useMotionValue } from "motion/react";
import Code from "../components/Code.jsx";

export const meta = ({ loaderData }) => [{ title: `${loaderData?.title ?? "Example"} · CyborgPL` }];

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
  const y = useMotionValue(0);
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
    animate(y, window.innerHeight, { type: "tween", duration: 0.15, ease: "easeIn" }).then(leave);
    // If the browser pauses animations (a background tab), close anyway.
    setTimeout(leave, 260);
  };

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
      <motion.div className="scrim" onClick={close} initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ duration: 0.12 }} />
      <motion.section
        className="sheet"
        style={{ y }}
        initial={{ y: "100%" }}
        animate={{ y: 0 }}
        transition={{ type: "spring", stiffness: 800, damping: 60, mass: 0.7 }}
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
            {/* No visible close button: swipe down (owner, 2026-09-25). This one is for screen
                readers only, since VoiceOver can't do the drag. */}
            <button className="sr-only" onClick={close}>Close</button>
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
