import { AnimatePresence, motion } from "motion/react";
import { useEffect, useRef, useState, type ReactNode } from "react";
import { createPortal } from "react-dom";
import EXAMPLES from "../generated/examples.json";

// Three programs from the repo, each with exactly what it prints (scripts/pre-build.ts). The tab
// highlight slides between tabs like the topbar's, and the code crossfades.
const glide = { type: "spring", stiffness: 520, damping: 40, mass: 0.8 } as const;
const grow = { type: "spring", stiffness: 300, damping: 34 } as const;

function FullscreenIcon({ open }: { open: boolean }) {
  return (
    <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      {open
        ? <path d="M5.5 1.5v4h-4M8.5 1.5v4h4M5.5 12.5v-4h-4M8.5 12.5v-4h4" />
        : <path d="M1.5 5V1.5H5M9 1.5h3.5V5M1.5 9v3.5H5M9 12.5h3.5V9" />}
    </svg>
  );
}

// Copies a panel's text in one press (owner, 2026-10-21: "Add easy copy"); the icon turns into a
// tick for a moment. Where the clipboard API is missing or refused, a hidden textarea does it.
async function copyText(text: string) {
  try { await navigator.clipboard.writeText(text); return true; } catch {}
  const area = Object.assign(document.createElement("textarea"), { value: text });
  area.setAttribute("readonly", "");
  area.style.cssText = "position:fixed;top:0;left:0;opacity:0";
  document.body.append(area);
  area.select();
  const done = document.execCommand("copy");
  area.remove();
  return done;
}

function CopyButton({ text, what }: { text: string; what: string }) {
  const [state, setState] = useState<"idle" | "done" | "failed">("idle");
  const timer = useRef(0);
  useEffect(() => () => clearTimeout(timer.current), []);
  const copy = async () => {
    setState((await copyText(text)) ? "done" : "failed");
    clearTimeout(timer.current);
    timer.current = window.setTimeout(() => setState("idle"), 1600);
  };
  return (
    <button className="fs-button copy-button" onClick={copy} aria-label={`Copy ${what}`} data-state={state}>
      <AnimatePresence mode="popLayout" initial={false}>
        <motion.svg key={state} width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"
          initial={{ opacity: 0, scale: 0.6 }} animate={{ opacity: 1, scale: 1 }} exit={{ opacity: 0, scale: 0.6 }} transition={{ type: "spring", stiffness: 600, damping: 30 }}>
          {state === "done" ? <path d="m2.5 7.5 3 3 6-7" /> : state === "failed" ? <path d="M3.5 3.5l7 7M10.5 3.5l-7 7" /> : <><rect x="4.5" y="4.5" width="8" height="8" rx="1.8" /><path d="M9.5 2.5V2.3A.8.8 0 0 0 8.7 1.5H2.3a.8.8 0 0 0-.8.8v6.4a.8.8 0 0 0 .8.8h.2" /></>}
        </motion.svg>
      </AnimatePresence>
      <span className="copy-said" aria-live="polite">{state === "done" ? "Copied" : state === "failed" ? "Couldn't copy" : ""}</span>
    </button>
  );
}

// A box that can go fullscreen (owner, 2026-10-19: "Add a fullscreen button for each (output too)").
// It grows out of its place to fill the window (Motion's shared layout), and asks the browser for
// real fullscreen where it allows it; iPhone Safari doesn't, so there it fills the window. Escape,
// the button, or leaving the browser's fullscreen brings it back.
function Panel({ id, className, title, copy, extra, children }: { id: string; className: string; title: string; copy: string; extra?: ReactNode; children: ReactNode }) {
  const [open, setOpen] = useState(false);
  useEffect(() => {
    if (!open) return;
    const root = document.documentElement;
    root.classList.add("fs-open");
    root.requestFullscreen?.().catch(() => {});
    const esc = (e: KeyboardEvent) => { if (e.key === "Escape") setOpen(false); };
    // Leaving the browser's fullscreen closes the box too, but only once it was really entered:
    // a browser that refuses or bounces back (iPhone Safari, an embedded view) keeps the window fill.
    let entered = false;
    const left = () => { if (document.fullscreenElement) entered = true; else if (entered) setOpen(false); };
    addEventListener("keydown", esc);
    document.addEventListener("fullscreenchange", left);
    return () => {
      root.classList.remove("fs-open");
      removeEventListener("keydown", esc);
      document.removeEventListener("fullscreenchange", left);
      if (document.fullscreenElement) document.exitFullscreen?.().catch(() => {});
    };
  }, [open]);
  const head = (
    <figcaption>
      <span>{title}</span>
      <span className="fs-side">
        {extra}
        <CopyButton text={copy} what={title === "Output" ? "the output" : title} />
        <button className="fs-button" onClick={() => setOpen(!open)} aria-label={open ? `Leave fullscreen` : `Show ${title} fullscreen`}>
          <FullscreenIcon open={open} />
        </button>
      </span>
    </figcaption>
  );
  return (
    <>
      <motion.figure className={className} layoutId={`panel-${id}`} transition={grow} style={{ visibility: open ? "hidden" : "visible" }}>
        {head}
        {children}
      </motion.figure>
      {/* Attached to <body>, so no moving parent can hold it inside itself. */}
      {typeof document !== "undefined" && createPortal(<AnimatePresence>
        {open && (
          <motion.div className="fs-backdrop" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} transition={{ duration: 0.2 }}>
            <motion.figure className={`${className} fs-panel`} layoutId={`panel-${id}`} transition={grow} role="dialog" aria-label={title}>
              {head}
              {children}
            </motion.figure>
          </motion.div>
        )}
      </AnimatePresence>, document.body)}
    </>
  );
}

export function Examples() {
  const [on, setOn] = useState(0);
  const ex = EXAMPLES[on]!;
  return (
    <div className="examples">
      <div className="ex-tabs" role="tablist" aria-label="Examples">
        {EXAMPLES.map((e, i) => (
          <button key={e.id} role="tab" aria-selected={i === on} className="ex-tab" onClick={() => setOn(i)}>
            {i === on && <motion.span className="ex-tab-on" layoutId="ex-tab-on" transition={glide} />}
            <span className="ex-tab-label">{e.tab}</span>
          </button>
        ))}
      </div>
      <AnimatePresence mode="wait" initial={false}>
        <motion.div key={ex.id} className="ex-panes" role="tabpanel" initial={{ opacity: 0, y: 8 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: -6 }} transition={{ duration: 0.18, ease: "easeOut" }}>
          <Panel id={`${ex.id}-code`} className="ex-code" title={ex.file} copy={ex.code.map((line) => line.map(([text]) => text).join("")).join("\n")} extra={<span className="fs-note">{ex.lines} {ex.lines === 1 ? "line" : "lines"}</span>}>
            <pre><code>{ex.code.map((line, i) => (
              <span className="code-line" key={i}>{line.map(([text, kind], j) => (kind ? <span key={j} className={`tk-${kind}`}>{text}</span> : text))}{"\n"}</span>
            ))}</code></pre>
          </Panel>
          <Panel id={`${ex.id}-out`} className="ex-out" title="Output" copy={ex.out}>
            <pre>{ex.out}</pre>
          </Panel>
        </motion.div>
      </AnimatePresence>
    </div>
  );
}
