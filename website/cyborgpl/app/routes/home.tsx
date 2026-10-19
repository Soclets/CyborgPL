import { motion } from "motion/react";
import type { CSSProperties } from "react";
import { Link } from "react-router";
import type { Route } from "./+types/home";

export const meta: Route.MetaFunction = () => [
  { title: "CyborgPL" },
  { name: "description", content: "CyborgPL, slow development, great final result." },
];

const order = (i: number) => ({ "--i": i }) as CSSProperties;
const springy = { whileHover: { scale: 1.035 }, whileTap: { scale: 0.97 }, transition: { type: "spring", stiffness: 500, damping: 26 } } as const;

export default function Home() {
  return (
    <main>
      <section className="hero">
        <div style={order(0)}>
          <motion.span className="pressable" {...springy}>
          <Link className="badge" to="/download">
            <span className="badge-new">New</span>
            <span className="badge-name">Meet CyDT 1</span>
            <span className="badge-dot" aria-hidden="true" />
            <span className="badge-sub">Our new model</span>
            <span className="badge-arrow" aria-hidden="true">
              <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round"><path d="M3.5 8.5 8.5 3.5M4.5 3.5h4v4" /></svg>
            </span>
          </Link>
          </motion.span>
        </div>
        <h1 className="hero-title" style={order(1)}>
          Merging with the <span className="hero-underline">machine</span>.
        </h1>
        <p className="hero-sub" style={order(2)}>CyborgPL, slow development, great final result.</p>
        <div className="hero-actions" style={order(3)}>
          <motion.span className="pressable" {...springy}>
            <Link className="button-primary" to="/download">
              Get CyborgPL
              <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d="m5.5 3.5 3.5 3.5-3.5 3.5" /></svg>
            </Link>
          </motion.span>
        </div>
      </section>
    </main>
  );
}
