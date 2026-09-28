import { useLoaderData } from "react-router";
import Code from "../components/Code.jsx";
import Icon from "../components/Icon.jsx";
import { ACCENTS, FEATURES, SYNTAX } from "../lib/content.js";
import { seo } from "../lib/seo.js";

export const meta = () => seo({ title: "Learn · CyborgPL", description: "Your first CyborgPL program, the syntax in five minutes, and the core features.", path: "/learn" });

// Build time: highlight the snippets and the first program with the repo's grammar.
export async function loader() {
  const { highlight, getExample } = await import("../lib/examples.server.js");
  const hello = await getExample("hello");
  return { hello, syntax: await Promise.all(SYNTAX.map(async (s) => ({ ...s, html: await highlight(s.code) }))) };
}

export default function Learn() {
  const { hello, syntax } = useLoaderData();
  return (
    <div className="learn">
      <h1 className="h2big">Learn</h1>

      <h2 className="section">Your first program</h2>
      <div className="panel">
        <Code html={hello.html} />
        <div className="outline"><span>Prints</span><code>{hello.out}</code></div>
      </div>

      <h2 className="section">Syntax in five minutes</h2>
      <div className="stack">
        {syntax.map((s, i) => (
          <article key={s.title} id={`syntax-${i + 1}`} className="lesson">
            <div className="lesson-head"><span className="num">{i + 1}</span><b>{s.title}</b></div>
            <p>{s.text}</p>
            <Code html={s.html} className="small" />
          </article>
        ))}
      </div>

      <h2 className="section">Core features</h2>
      <div className="stack">
        {FEATURES.map((f, i) => (
          <article key={f.id} id={f.id} className="feature" style={{ "--c": ACCENTS[i] }}>
            <span className="fi"><Icon name={f.icon} size={20} /></span>
            <div><b>{f.title}</b><p>{f.text}</p></div>
          </article>
        ))}
      </div>
    </div>
  );
}
