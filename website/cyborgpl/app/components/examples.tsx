import { AnimatePresence, motion } from "motion/react";
import { useState } from "react";
import EXAMPLES from "../generated/examples.json";

// Three programs from the repo, each with exactly what it prints (scripts/pre-build.ts). The tab
// highlight slides between tabs like the topbar's, and the code crossfades.
const glide = { type: "spring", stiffness: 520, damping: 40, mass: 0.8 } as const;

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
          <figure className="ex-code">
            <figcaption>{ex.file}<span>{ex.lines} {ex.lines === 1 ? "line" : "lines"}</span></figcaption>
            <pre><code>{ex.source}</code></pre>
          </figure>
          <figure className="ex-out">
            <figcaption>Output</figcaption>
            <pre>{ex.out}</pre>
          </figure>
        </motion.div>
      </AnimatePresence>
    </div>
  );
}
