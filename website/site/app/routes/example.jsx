import { Suspense, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { Await, useLoaderData, useLocation, useNavigate, useRouteLoaderData } from "react-router";
import { animate, motion, useDragControls, useMotionValue } from "motion/react";
import Code from "../components/Code.jsx";
import CopyButton from "../components/CopyButton.jsx";
import { cached, loadExamples } from "../lib/example-cache.js";
import Icon from "../components/Icon.jsx";
import Md from "../components/Md.jsx";
import { plain } from "../lib/md.js";
import { seo } from "../lib/seo.js";

// The grid's entry: title, summary and the plain source, ready before a card is clicked.
const briefOf = (list, id) => list?.examples?.find((e) => e.id === id);

export const meta = ({ loaderData, params, matches }) => {
  const list = matches.find((m) => m?.id === "routes/examples");
  const ex = loaderData?.title ? loaderData : briefOf(list?.loaderData ?? list?.data, params.id);
  return seo({
  title: `${ex?.title ? plain(ex.title) : "Example"} · Nerve`,
  description: (ex?.summary && plain(ex.summary)) || `examples/${params.id}.nrv — a Nerve program and the output it prints.`,
  path: `/examples/${params.id}`,
  });
};

export async function loader({ params }) {
  const { getExample } = await import("../lib/examples.server.js");
  return getExample(params.id);
}

// Opening doesn't wait (owner: they took a while to pop up; then "there shouldn't be a delay for
// syntax highlighting and the copy button"): the Examples grid loads every example whole in the
// background (lib/example-cache.js), so a click finds it ready. If it isn't there yet, the plain
// source shows while it arrives, and the example's own data is the last resort. A direct link is
// pre-rendered whole, as before.
export function clientLoader({ serverLoader, params }) {
  return { id: params.id, load: serverLoader };
}

// The example opens as a bottom sheet over the grid. Swipe it down to put it away
// (tapping the dimmed strip above it, or Escape, also works).
export default function Example() {
  const data = useLoaderData();
  const id = data.id;
  const list = useRouteLoaderData("routes/examples");
  const brief = data.load ? briefOf(list, id) : data;
  // the whole example: the grid's entry (or the pre-rendered page's), else fetched once
  const full = useMemo(() => {
    if (brief?.html) return brief;                       // the pre-rendered page: already whole
    const got = cached(id);
    if (got) return { ...brief, ...got };                // the usual case: loaded with the grid
    return loadExamples().then((all) => ({ ...brief, ...all[id] })).catch(() => data.load?.());
  }, [id]);
  const title = brief?.title ?? id;
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

  useEffect(() => setTab("code"), [id]);

  return (
    <div className="sheet-layer" role="dialog" aria-modal="true" aria-label={plain(title)}>
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
              <b><Md text={title} /></b>
              <small>examples/{id}.nrv{brief?.lines ? ` · ${brief.lines} lines` : ""}</small>
            </div>
            {/* On a phone there is no visible close button: swipe down (owner, 2026-09-25), and this
                one is for screen readers only. A computer can't swipe, so there it shows (app.css). */}
            <button className="sheet-close" onClick={() => close()} aria-label="Close"><Icon name="x" size={20} /></button>
          </div>
          {brief?.summary && <p className="sheet-summary"><Md text={brief.summary} /></p>}
          <div className="seg-row">
            <div className="seg" role="tablist">
              <button role="tab" aria-selected={tab === "code"} className={tab === "code" ? "on" : ""} onClick={() => setTab("code")}>Code</button>
              <button role="tab" aria-selected={tab === "output"} className={tab === "output" ? "on" : ""} onClick={() => setTab("output")}>Output</button>
            </div>
            {/* Copy sits beside the tabs, never over the code (owner, 2026-10-12). */}
            {tab === "code" && (full?.source ?? brief?.source) && <CopyButton text={full?.source ?? brief?.source} label="Copy code" />}
            {tab === "output" && full?.out && <CopyButton text={full.out} label="Copy output" />}
          </div>
        </div>
        <div className="sheet-body">
          {full?.html ? <Body ex={full} tab={tab} /> : (
            <Suspense fallback={<div className="code sheet-code sheet-pending"><pre style={{ "--example-lines": brief?.lines }}>{brief?.source ?? brief?.preview}</pre></div>}>
              <Await resolve={full} errorElement={<p className="empty">This example didn’t load. Close it and try again.</p>}>
                {(ex) => <Body ex={ex} tab={tab} />}
              </Await>
            </Suspense>
          )}
        </div>
      </motion.section>
    </div>
  );
}

// An example's code or output, once it is here.
function Body({ ex, tab }) {
  return tab === "code" ? <Code html={ex.html} className="sheet-code" /> : (
    <div className="output">
      <pre>{ex.out || "(prints nothing)"}</pre>
      {ex.exit && <p className="exit">Exits with code {ex.exit}.</p>}
      <p className="checked">This is the output the reference interpreter’s tests check for.</p>
    </div>
  );
}
