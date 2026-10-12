import { Link, useLoaderData } from "react-router";
import Code from "../components/Code.jsx";
import Md from "../components/Md.jsx";
import { seo } from "../lib/seo.js";

export const meta = () => seo({ title: "Good practice · Nerve", description: "How Nerve is meant to be written: the practices its rules make easy. Advice, not rules.", path: "/good-practice" });

export async function loader() {
  const { readPractice } = await import("../lib/practice.server.js");
  return { groups: await readPractice() };
}

// Good practice (owner, 2026-10-02): design/style.md, linked from Learn's title and from More.
export default function Practice() {
  const { groups } = useLoaderData();
  return (
    <div className="practice">
      <Link to="/learn" className="crumb">‹ Learn</Link>
      <h1 className="h2big">Good practice</h1>
      <p className="lede small">How Nerve is meant to be written: the practices its rules make easy. They’re advice, not rules — the compiler enforces none of them.</p>
      <div className="notice"><b>Names are free</b><p>A quoted name may be anything, and no style for it is recommended.</p></div>
      {groups.map((g) => (
        <section key={g.title}>
          <h2 className="section">{g.title}</h2>
          <div className="stack">
            {g.items.map((it) => (
              <article key={it.n} id={`p-${it.n}`} className="lesson">
                <div className="lesson-head"><span className="num">{it.n}</span><b><Md text={it.title} /></b></div>
                {it.code && <Code html={it.code} className="small" />}
                <p><Md text={it.text} /></p>
              </article>
            ))}
          </div>
        </section>
      ))}
    </div>
  );
}
