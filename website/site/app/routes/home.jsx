import { useRef, useState } from "react";
import { Link, useLoaderData } from "react-router";
import Icon from "../components/Icon.jsx";
import { ACCENTS, DESCRIPTION, FEATURES, REPO_URL, SYNTAX } from "../lib/content.js";

export const meta = () => [
  { title: "CyborgPL" },
  { name: "description", content: DESCRIPTION },
];

// Build time: one small index over everything worth finding.
export async function loader() {
  const { listExamples } = await import("../lib/examples.server.js");
  const pages = [
    { to: "/learn", title: "Learn", text: "Your first program, syntax in five minutes, core features", kind: "Page", icon: "book" },
    { to: "/examples", title: "Examples", text: "Every program in the repo, with its output", kind: "Page", icon: "code" },
    { to: "/download", title: "Download", text: "Nothing to download yet", kind: "Page", icon: "download" },
    { to: "/licence", title: "Licence", text: "Apache-2.0 with the LLVM exception; text CC BY 4.0", kind: "Page", icon: "scale" },
  ];
  const features = FEATURES.map((f) => ({ to: `/learn#${f.id}`, title: f.title, text: f.text, kind: "Feature", icon: f.icon }));
  const lessons = SYNTAX.map((s, i) => ({ to: `/learn#syntax-${i + 1}`, title: s.title, text: s.text, kind: "Syntax", icon: "book" }));
  const examples = listExamples().map((e) => ({ to: `/examples/${e.id}`, title: e.title, text: e.summary || `examples/${e.id}.cpls`, kind: "Example", icon: "code", id: e.id }));
  return { index: [...pages, ...features, ...lessons, ...examples] };
}

// Every word typed must start a word in the entry ("own" finds Ownership, not Download).
// Title matches rank above matches in the description.
const words = (t) => t.toLowerCase().split(/[^a-z0-9:=.]+/).filter(Boolean);
function search(index, needle) {
  const terms = words(needle);
  return index
    .map((r, order) => {
      const title = words(r.title + " " + (r.id ?? "").replace(/_/g, " ")), text = words(r.text);
      let score = 0;
      for (const t of terms) {
        if (title.some((w) => w.startsWith(t))) score += 2;
        else if (text.some((w) => w.startsWith(t))) score += 1;
        else return null;
      }
      return { r, score, order };
    })
    .filter(Boolean)
    .sort((a, b) => b.score - a.score || a.order - b.order)
    .slice(0, 40)
    .map((x) => x.r);
}

function Search({ q, setQ }) {
  return (
    <label className="search">
      <Icon name="search" size={17} />
      <input type="search" placeholder="Search CyborgPL" value={q} onChange={(e) => setQ(e.target.value)} enterKeyHint="search" aria-label="Search CyborgPL" />
    </label>
  );
}

export default function Home() {
  const { index } = useLoaderData();
  const [q, setQ] = useState("");
  const row = useRef(null);
  const [card, setCard] = useState(0);
  const onScroll = () => {
    const el = row.current;
    if (!el) return;
    const w = el.firstElementChild?.getBoundingClientRect().width ?? 1;
    setCard(Math.min(FEATURES.length - 1, Math.round(el.scrollLeft / (w + 12))));
  };

  const needle = q.trim().toLowerCase();
  if (needle) {
    const hits = search(index, needle);
    return (
      <div className="home">
        <Search q={q} setQ={setQ} />
        <div className="results">
          {hits.map((r, i) => (
            <Link key={r.to} to={r.to} state={r.kind === "Example" ? { fromGrid: true } : undefined} className="result" prefetch="intent" style={{ "--c": ACCENTS[i % ACCENTS.length] }}>
              <span className="ri"><Icon name={r.icon} size={18} /></span>
              <span><b>{r.title}</b><small>{r.text}</small></span>
              <em>{r.kind}</em>
            </Link>
          ))}
          {!hits.length && <p className="empty">Nothing matches “{q}”.</p>}
        </div>
      </div>
    );
  }

  return (
    <div className="home">
      <Search q={q} setQ={setQ} />
      <h1 className="huge">Selfless dev<br />meets <span>Selfish</span><br />language.</h1>
      <p className="lede">{DESCRIPTION}</p>
      <div className="actions">
        <Link className="btn primary" to="/download" prefetch="intent"><Icon name="download" size={18} stroke={2.3} />Download</Link>
        <Link className="btn" to="/learn" prefetch="intent">Read shit</Link>
        <a className="btn round" href={REPO_URL} target="_blank" rel="noopener noreferrer" aria-label="Source on GitHub"><Icon name="github" size={20} stroke={1.9} /></a>
      </div>
      <h2 className="section">Core features</h2>
      <div className="cards" ref={row} onScroll={onScroll}>
        {FEATURES.map((f, i) => (
          <Link key={f.id} className="fcard" to={`/learn#${f.id}`} prefetch="intent" style={{ "--c": ACCENTS[i] }}>
            <span className="fi"><Icon name={f.icon} size={22} /></span>
            <b>{f.title}</b>
            <small>{f.text}</small>
          </Link>
        ))}
      </div>
      <div className="dots" aria-hidden="true">
        {FEATURES.map((f, i) => <i key={f.id} className={i === card ? "on" : ""} />)}
      </div>
    </div>
  );
}
