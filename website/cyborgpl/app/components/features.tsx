import { motion } from "motion/react";
import type { ReactNode } from "react";

// What CyborgPL is, in four (owner, 2026-10-20: "That's pretty old. So, let's do a new one. 1. Precog,
// <explain> 2. Ownership <explain> 3. `::` marks a kit, `.` narrows (`std::print.useful`) 4. LLVM as
// the backend"). The explanations are website claude's, written to the current design.
const FEATURES: { title: ReactNode; text: ReactNode }[] = [
  {
    title: "Precog",
    text: <>Built into <code>KAYO\n</code>, it works out every value your program can hold, across the whole program, while it builds. Every overflow, out-of-range index and division by zero is proven impossible, or the final build stops and shows you the value that breaks it. Nothing is left to check at run time.</>,
  },
  {
    title: "Ownership",
    text: <>Every value has exactly one owner and is freed when that owner's scope ends, with no garbage collector. Moves, copies and borrows (<code>lend</code>, <code>lent</code>) are proven while building, so a borrow never outlives its owner or changes under you.</>,
  },
  {
    title: <><code>::</code> marks a kit, <code>.</code> narrows</>,
    text: <>In <code>std::print.useful</code>, <code>std</code> is the kit, and <code>print.useful</code> narrows <code>print</code> down to the useful one.</>,
  },
  {
    title: "LLVM as the backend",
    text: <><code>KAYO\n</code> compiles to native code through LLVM, and hands it what Precog proved, so LLVM can optimise further.</>,
  },
];

export function Features() {
  return (
    <section className="features" aria-label="What CyborgPL is">
      {FEATURES.map((f, i) => (
        <motion.article
          key={i}
          className="feature"
          // The hero's entrance (owner, 2026-10-20: "make it have that smooth load anim"): each card
          // rises and comes into focus, one after another, as the row scrolls into view.
          initial={{ opacity: 0, y: 18, filter: "blur(6px)" }}
          whileInView={{ opacity: 1, y: 0, filter: "blur(0px)" }}
          viewport={{ once: true, amount: 0.3 }}
          transition={{ duration: 0.8, ease: [0.2, 0.8, 0.2, 1], delay: 0.11 * i }}
          whileHover={{ y: -4, transition: { type: "spring", stiffness: 400, damping: 26 } }}
        >
          <span className="feature-index">{String(i + 1).padStart(2, "0")}</span>
          <h3>{f.title}</h3>
          <p>{f.text}</p>
        </motion.article>
      ))}
    </section>
  );
}
