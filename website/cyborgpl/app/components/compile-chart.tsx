import { motion } from "motion/react";
import { useState } from "react";
import type { CompileRow, CompileRun } from "../data/compile-times";

// How long each language takes to build the four bench programs, as one bar each on one linear
// axis: CyborgPL's in white, the others in grey, every bar named and labelled with its time, the
// per-program split on hover, and a table for screen readers.
const PROGRAMS = ["sum", "fib", "lists", "sieve"] as const;
const total = (r: CompileRow) => PROGRAMS.reduce((s, p) => s + r.seconds[p], 0);
const fmt = (s: number) => (s >= 10 ? `${s.toFixed(0)} s` : s >= 1 ? `${s.toFixed(1)} s` : `${(s * 1000).toFixed(0)} ms`);

export function CompileChart({ run }: { run: CompileRun | null }) {
  const [hover, setHover] = useState<string | null>(null);
  if (!run) {
    // Nothing measured yet: a blueprint of the chart, dashed like the loading screen's.
    return (
      <div className="chart chart-pending" role="img" aria-label="Compile times: being measured">
        {[92, 18, 24, 12, 30].map((w, i) => (
          <div className="chart-row" key={i}>
            <span className="chart-name chart-ghost-text" />
            <span className="chart-track"><span className="chart-ghost" style={{ width: `${w}%` }} /></span>
          </div>
        ))}
        <p className="chart-note">The compile times are being measured.</p>
      </div>
    );
  }
  const rows = [...run.rows].sort((a, b) => total(b) - total(a));
  const max = Math.max(...rows.map(total));
  return (
    <figure className="chart">
      <div className="chart-rows" aria-hidden="true">
        {rows.map((r, i) => (
          <div className={`chart-row${r.ours ? " is-ours" : ""}`} key={r.name} onPointerEnter={() => setHover(r.name)} onPointerLeave={() => setHover(null)}>
            <span className="chart-name">{r.name}</span>
            <span className="chart-track">
              <motion.span
                className="chart-bar"
                initial={{ scaleX: 0 }}
                whileInView={{ scaleX: total(r) / max }}
                viewport={{ once: true, amount: 0.6 }}
                transition={{ type: "spring", stiffness: 120, damping: 22, delay: 0.08 * i }}
              />
            </span>
            <span className="chart-value">{fmt(total(r))}</span>
            {hover === r.name && (
              <motion.span className="chart-tip" initial={{ opacity: 0, y: 4 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.15 }}>
                {PROGRAMS.map((p) => <span key={p}>{p} <b>{fmt(r.seconds[p])}</b></span>)}
              </motion.span>
            )}
          </div>
        ))}
      </div>
      <figcaption className="chart-note">Building the four bench programs from source, best of 3, no cache. {run.date}, {run.machine}.</figcaption>
      <table className="sr-only">
        <caption>Compile time per language</caption>
        <thead><tr><th>Language</th>{PROGRAMS.map((p) => <th key={p}>{p}</th>)}<th>Total</th></tr></thead>
        <tbody>{rows.map((r) => <tr key={r.name}><td>{r.name}</td>{PROGRAMS.map((p) => <td key={p}>{fmt(r.seconds[p])}</td>)}<td>{fmt(total(r))}</td></tr>)}</tbody>
      </table>
    </figure>
  );
}
