import { useEffect, useRef, useState } from "react";
import { Link, useLoaderData, useNavigate } from "react-router";
import Code from "../components/Code.jsx";
import Icon from "../components/Icon.jsx";
import Marked from "../components/Marked.jsx";
import { ANSWERS } from "../lib/answers.js";
import { ACCENTS, DESCRIPTION, FEATURES, REPO_URL, SYNTAX } from "../lib/content.js";
import { around, runSearch } from "../lib/search.js";
import { useCodeIndex, useRecent } from "../lib/useSearch.js";

export const meta = () => [
  { title: "CyborgPL" },
  { name: "description", content: DESCRIPTION },
];

// Build time: one small index over everything worth finding.
export async function loader() {
  const { highlight, listExamples } = await import("../lib/examples.server.js");
  const pages = [
    { to: "/learn", title: "Learn", text: "Your first program, syntax in five minutes, core features", kind: "Page", icon: "book" },
    { to: "/examples", title: "Examples", text: "Every program in the repo, with its output", kind: "Page", icon: "code" },
    { to: "/download", title: "Download", text: "Nothing to download yet", kind: "Page", icon: "download" },
    { to: "/licence", title: "Licence", text: "Apache-2.0 with the LLVM exception; text CC BY 4.0", kind: "Page", icon: "scale" },
  ];
  const features = FEATURES.map((f) => ({ to: `/learn#${f.id}`, title: f.title, text: f.text, kind: "Feature", icon: f.icon }));
  const lessons = SYNTAX.map((s, i) => ({ to: `/learn#syntax-${i + 1}`, title: s.title, text: s.text, kind: "Syntax", icon: "book" }));
  const examples = listExamples().map((e) => ({ to: `/examples/${e.id}`, title: e.title, text: e.summary || `examples/${e.id}.cpls`, kind: "Example", icon: "code", id: e.id }));
  // An answer's code must be a real line of its example, or the build stops.
  const { readFileSync } = await import("node:fs");
  const answers = await Promise.all(ANSWERS.map(async (a) => {
    const src = readFileSync(`../../examples/${a.from}.cpls`, "utf8");
    for (const line of a.code.split("\n")) if (!src.includes(line)) throw new Error(`answers.js: "${line}" is not in examples/${a.from}.cpls`);
    return highlight(a.code);
  }));
  return { index: [...pages, ...features, ...lessons, ...examples], answers };
}

// Tapped on the empty bar: things worth trying. Each one has an instant answer.
const TRY = [":=", "lend", "otherwise", "exceptions", "garbage collector", "concur", "mod"];

function Search({ q, setQ, onSubmit, onFocus, onBlur }) {
  return (
    <form className="search" role="search" onSubmit={(e) => { e.preventDefault(); onSubmit(); }}>
      <Icon name="search" size={17} />
      <input type="search" placeholder="Search CyborgPL — even code" value={q} onChange={(e) => setQ(e.target.value)} onFocus={onFocus} onBlur={onBlur}
        enterKeyHint="go" autoCapitalize="none" autoCorrect="off" spellCheck={false} aria-label="Search CyborgPL" />
    </form>
  );
}

// Asking for the code gets a button straight to it (owner, 2026-09-25).
function SourceHit() {
  return (
    <a className="source-hit" href={REPO_URL} target="_blank" rel="noopener noreferrer">
      <span className="gh"><Icon name="github" size={24} stroke={1.8} /></span>
      <span><b>Source code on GitHub</b><small>Cyborg-Assembly-Manufacturing-Site/CyborgPL</small></span>
      <Icon name="external" size={18} />
    </a>
  );
}

function Answer({ a, html, q, onGo }) {
  return (
    <Link to={a.to} className="answer" prefetch="intent" onClick={onGo} state={a.to.startsWith("/examples/") ? { fromGrid: true } : undefined}>
      {a.foreign?.includes(q) && <p className="foreign"><code>{q}</code> isn’t built in here.</p>}
      <b>{a.title}</b>
      <p>{a.text}</p>
      <Code html={html} />
      <small>examples/{a.from}.cpls <Icon name="chev" size={14} /></small>
    </Link>
  );
}

// The query outlives the page, so coming back from a result finds it still there.
let lastQ = "";

export default function Home() {
  const { index, answers } = useLoaderData();
  const [q, setQState] = useState(lastQ);
  const setQ = (v) => { lastQ = v; setQState(v); };
  // A second tap on this tab, already at the top, clears the search.
  useEffect(() => {
    const clear = () => { setQ(""); document.activeElement?.blur(); };
    addEventListener("cyborgpl:retap", clear);
    return () => removeEventListener("cyborgpl:retap", clear);
  }, []);
  const row = useRef(null);
  const [card, setCard] = useState(0);
  const onScroll = () => {
    const el = row.current;
    if (!el) return;
    const w = el.firstElementChild?.getBoundingClientRect().width ?? 1;
    setCard(Math.min(FEATURES.length - 1, Math.round(el.scrollLeft / (w + 12))));
  };

  const navigate = useNavigate();
  const [focused, setFocused] = useState(false);
  const code = useCodeIndex(focused || !!q);
  const { recent, remember, clear } = useRecent();
  const found = runSearch(index, q, code);
  const go = () => remember(q);
  // Enter opens the first thing on the list.
  const submit = () => {
    if (!found.q) return;
    remember(q);
    if (found.answer) navigate(found.answer.to, { state: found.answer.to.startsWith("/examples/") ? { fromGrid: true } : undefined });
    else if (found.source) window.open(REPO_URL, "_blank", "noopener");
    else if (found.results[0]) navigate(found.results[0].entry.to, { state: found.results[0].entry.kind === "Example" ? { fromGrid: true } : undefined });
  };
  // Blur waits a beat, so a tap on a chip lands before the chips go away.
  const blurTimer = useRef(0);
  const onFocus = () => { clearTimeout(blurTimer.current); setFocused(true); };
  const onBlur = () => { blurTimer.current = setTimeout(() => setFocused(false), 150); };
  const search = <Search q={q} setQ={setQ} onSubmit={submit} onFocus={onFocus} onBlur={onBlur} />;
  // Chips keep the keyboard up: pressing one must not blur the bar.
  const keep = (e) => e.preventDefault();

  if (found.q) {
    const { answer, answerIndex, source, results } = found;
    return (
      <div className="home">
        {search}
        <div className="results">
          {source && <SourceHit />}
          {answer && <Answer a={answer} html={answers[answerIndex]} q={found.q} onGo={go} />}
          {results.map(({ entry: r, line }, i) => (
            <Link key={r.to} to={r.to} onClick={go} state={r.kind === "Example" ? { fromGrid: true } : undefined} className="result" prefetch="intent" style={{ "--c": ACCENTS[i % ACCENTS.length] }}>
              <span className="ri"><Icon name={r.icon} size={18} /></span>
              <span>
                <b><Marked text={r.title} q={found.q} /></b>
                {line ? <code className="hit"><Marked text={around(line, found.q)} q={found.q} /></code> : <small><Marked text={r.text} q={found.q} /></small>}
              </span>
              <em>{r.kind}</em>
            </Link>
          ))}
          {!results.length && !source && !answer && <p className="empty">Nothing matches “{q}”.</p>}
        </div>
      </div>
    );
  }

  if (focused) {
    return (
      <div className="home">
        {search}
        {recent.length > 0 && (
          <>
            <div className="suggest-head"><h2 className="section">Recent</h2><button onMouseDown={keep} onClick={clear}>Clear</button></div>
            <div className="chips">{recent.map((r) => <button key={r} onMouseDown={keep} onClick={() => setQ(r)}>{r}</button>)}</div>
          </>
        )}
        <h2 className="section">Try</h2>
        <div className="chips">{TRY.map((t) => <button key={t} className={/^[a-z ]+$/.test(t) ? "" : "mono"} onMouseDown={keep} onClick={() => setQ(t)}>{t}</button>)}</div>
        <p className="hint">Searches names, pages and every line of code in the examples. Enter opens the top result.</p>
      </div>
    );
  }

  return (
    <div className="home">
      {search}
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
