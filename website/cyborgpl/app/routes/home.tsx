import { motion } from "motion/react";
import type { CSSProperties } from "react";
import { Link } from "react-router";
import type { Route } from "./+types/home";
import { CompileChart } from "../components/compile-chart";
import { COMPILE } from "../data/compile-times";
import { RUN } from "../data/run-times";
import { Examples } from "../components/examples";
import { Features } from "../components/features";
import { HeroSim } from "../hero-sim/hero-sim";

export const meta: Route.MetaFunction = () => [
  { title: "CyborgPL" },
  { name: "description", content: "Very slow at the development, fast & safe at the final product. No warranty." },
];

const order = (i: number) => ({ "--i": i }) as CSSProperties;
const springy = { whileHover: { scale: 1.035 }, whileTap: { scale: 0.97 }, transition: { type: "spring", stiffness: 500, damping: 26 } } as const;

export default function Home() {
  return (
    <main>
      <section className="hero">
        <HeroSim />
        <div style={order(0)}>
          <motion.span className="pressable" {...springy}>
          <Link className="badge" to="/download">
            <span className="badge-new">New</span>
            <span className="badge-name">CyDT 1</span>
            <span className="badge-dot" aria-hidden="true" />
            <span className="badge-sub">try out our latest innovation</span>
            <span className="badge-arrow" aria-hidden="true">
              <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round"><path d="M3.5 8.5 8.5 3.5M4.5 3.5h4v4" /></svg>
            </span>
          </Link>
          </motion.span>
        </div>
        <h1 className="hero-title" style={order(1)}>
          Merging with the <strong className="hero-underline">machine</strong>.
        </h1>
        {/* Owner, 2026-10-24: "**Very slow** at the **development**, **fast & safe** at the **final product.** No warranty." */}
        <p className="hero-sub" style={order(2)}>
          <strong>Very slow</strong> at the <strong>development</strong>, <strong>fast &amp; safe</strong> at the <strong>final product.</strong> No warranty.
        </p>
        <div className="hero-actions" style={order(3)}>
          <motion.span className="pressable" {...springy}>
            <Link className="button-primary" to="/download">
              Get CyborgPL
              <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d="m5.5 3.5 3.5 3.5-3.5 3.5" /></svg>
            </Link>
          </motion.span>
        </div>
      </section>

      <Features />

      {/* Below the hero (owner, 2026-10-19: "Scroll down, and you'll get "Don't script, don't
          iterate, but fucking wait." on the right side, and compilation time benchmark on the
          right"): the line, then KAYO\n's compile times beside it. */}
      <section className="wait">
        <motion.h2 className="wait-line" initial={{ opacity: 0, x: -28 }} whileInView={{ opacity: 1, x: 0 }} viewport={{ once: true, amount: 0.5 }} transition={{ type: "spring", stiffness: 90, damping: 20 }}>
          Don't script, don't iterate, but fucking <strong>wait</strong>.
        </motion.h2>
        <motion.div initial={{ opacity: 0, x: 28 }} whileInView={{ opacity: 1, x: 0 }} viewport={{ once: true, amount: 0.4 }} transition={{ type: "spring", stiffness: 90, damping: 20, delay: 0.1 }}>
          <CompileChart run={COMPILE} />
        </motion.div>
      </section>

      {/* Further down (owner, 2026-10-19: "Scroll down, and you get "Hey, at least the final result
          is... Above average?" on the left, and benchmark on the right."): the run times. */}
      <section className="wait">
        <motion.h2 className="wait-line" initial={{ opacity: 0, x: -28 }} whileInView={{ opacity: 1, x: 0 }} viewport={{ once: true, amount: 0.5 }} transition={{ type: "spring", stiffness: 90, damping: 20 }}>
          Hey, at least the final result is... <strong>Above average</strong>?
        </motion.h2>
        <motion.div initial={{ opacity: 0, x: 28 }} whileInView={{ opacity: 1, x: 0 }} viewport={{ once: true, amount: 0.4 }} transition={{ type: "spring", stiffness: 90, damping: 20, delay: 0.1 }}>
          <CompileChart run={RUN} label="Run time" lead="Running the four bench programs, final builds, best of 3" />
        </motion.div>
      </section>

      {/* Further down again (owner, 2026-10-19: "Scroll down, and you get 3 examples: Hello World, C
          stuff, and the biggest."). */}
      <motion.section className="ex-section" initial={{ opacity: 0, y: 28 }} whileInView={{ opacity: 1, y: 0 }} viewport={{ once: true, amount: 0.2 }} transition={{ type: "spring", stiffness: 90, damping: 20 }}>
        <Examples />
      </motion.section>
    </main>
  );
}
