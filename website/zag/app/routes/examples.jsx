import { useMemo, useState } from "react";
import { Link, useParams } from "react-router";
import { useSiteSky } from "../sky.jsx";
import { seo } from "../seo.js";
import Md from "../md.jsx";
// Every program in the repo's examples/, coloured at build time (scripts/pre-build.mjs), with what
// it prints. They are the language's tests too: the reference interpreter runs each one and checks
// its output, so what the page shows is what the language does today.
import DATA from "../generated/examples.json";
import "../generated/code-styles.css";

const { programs } = DATA;
const byId = new Map(programs.map((p, i) => [p.id, i]));

// A few to begin with, from the smallest to the longest (owner, 2026-10-15: "Do Examples").
const START = ["hello", "fizzbuzz", "collections", "generics", "testing", "c_headers", "asm_beside_zag", "corner_shop"].filter((id) => byId.has(id));

export const meta = ({ params }) => {
  const p = params.id && programs[byId.get(params.id)];
  if (!p) return seo({ title: "Examples · Zag", path: "/examples", description: `All ${programs.length} of Zag's example programs, each with exactly what it prints.` });
  return seo({ title: `${p.title} · Examples · Zag`, path: `/examples/${p.id}`, description: p.blurb ? p.blurb.replace(/`/g, "") : `${p.title}, an example Zag program, with what it prints.` });
};

export default function Examples() {
  useSiteSky();
  const { id } = useParams();
  return (
    <main className="page page-in" key={id ?? "all"}>
      <div className="page-body">{id ? <Program id={id} /> : <All />}</div>
    </main>
  );
}

function All() {
  const [query, setQuery] = useState("");
  const shown = useMemo(() => {
    const words = query.toLowerCase().split(/\s+/).filter(Boolean);
    if (!words.length) return programs;
    return programs.filter((p) => {
      const text = `${p.id} ${p.title} ${p.blurb}`.toLowerCase().replace(/_/g, " ");
      return words.every((w) => text.includes(w));
    });
  }, [query]);
  return (
    <>
      <h1 className="page-title">Examples</h1>
      <p className="page-lead">All {programs.length} programs from the repo, each with exactly what it prints. They are Zag’s tests as well: every one runs and is checked, so they show the language as it is today.</p>
      {!query && (
        <section className="info" aria-labelledby="start-title">
          <h2 id="start-title">Start here</h2>
          <ul className="ex-grid">{START.map((id) => <Item key={id} p={programs[byId.get(id)]} />)}</ul>
        </section>
      )}
      <section className="info" aria-labelledby="all-title">
        <div className="ex-find">
          <h2 id="all-title">{query ? `${shown.length} found` : "Every program"}</h2>
          <input type="search" value={query} onChange={(e) => setQuery(e.target.value)} placeholder="Search, e.g. simd, failures, C" aria-label="Search the examples" />
        </div>
        {shown.length ? <ul className="ex-grid">{shown.map((p) => <Item key={p.id} p={p} />)}</ul> : <p>Nothing matches that.</p>}
      </section>
    </>
  );
}

function Item({ p }) {
  return (
    <li>
      <Link className="ex-item" to={`/examples/${p.id}`}>
        <span className="ex-item-title">{p.title}</span>
        {p.blurb && <span className="ex-item-blurb"><Md text={p.blurb} /></span>}
        <span className="ex-item-file">{p.folder ? `${p.id}/ · ${p.files.length} files` : `${p.id}.zg`}</span>
      </Link>
    </li>
  );
}

function Program({ id }) {
  const i = byId.get(id);
  const [file, setFile] = useState(0);
  if (i === undefined) {
    return (
      <>
        <h1 className="page-title">No example called “{id}”</h1>
        <p className="page-lead"><Link className="home-link" to="/examples">All examples</Link></p>
      </>
    );
  }
  const p = programs[i], f = p.files[Math.min(file, p.files.length - 1)];
  const prev = programs[i - 1], next = programs[i + 1];
  return (
    <>
      <Link className="back-link" to="/examples">← All examples</Link>
      <h1 className="page-title">{p.title}</h1>
      {p.blurb && <p className="page-lead"><Md text={p.blurb} /></p>}
      <div className="ex-card ex-one">
        {p.files.length > 1 && (
          <div className="ex-files" role="tablist" aria-label="Files">
            {p.files.map((g, j) => <button key={g.name} role="tab" aria-selected={g === f} onClick={() => setFile(j)}>{g.name}</button>)}
          </div>
        )}
        <div className="ex-panes">
          <figure className="ex-code">
            <figcaption>{p.folder ? `${p.id}/${f.name}` : f.name}</figcaption>
            {f.lines ? <Code lines={f.lines} /> : <pre className="ex-binary">{f.bytes} bytes of raw data, read into the program while it is built.</pre>}
          </figure>
          <Output p={p} />
        </div>
      </div>
      <nav className="ex-nav" aria-label="Other examples">
        {prev ? <Link to={`/examples/${prev.id}`}>← {prev.title}</Link> : <span />}
        {next ? <Link to={`/examples/${next.id}`}>{next.title} →</Link> : <span />}
      </nav>
    </>
  );
}

// One line per row, so the line numbers (CSS counters) stay out of anything copied.
function Code({ lines }) {
  return (
    <pre className="ex-lines"><code>{lines.map((line, i) => (
      <span className="ex-line" key={i}>{line.map(([text, k], j) => (k < 0 ? text : <span key={j} className={`k${k}`}>{text}</span>))}{"\n"}</span>
    ))}</code></pre>
  );
}

function Output({ p }) {
  return (
    <figure className="ex-out">
      <figcaption>{p.tests != null ? "Output, then zomp test" : "Output"}</figcaption>
      {p.args && <p className="ex-meta">Run with {p.args.map((a, i) => <code key={i}>{a}</code>)}</p>}
      <pre>{p.out || <span className="ex-quiet">Prints nothing.</span>}</pre>
      {p.tests != null && <pre className="ex-tests">{p.tests}</pre>}
      {p.exit && <p className="ex-meta">Exits with code <code>{p.exit}</code>.</p>}
      {p.noOracle && <p className="ex-meta">The reference interpreter can’t run this one to the end, as it doesn’t run assembly.</p>}
    </figure>
  );
}
