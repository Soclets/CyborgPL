import { useEffect, useRef, useState } from "react";
import { Link, useLoaderData, useNavigate } from "react-router";
import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import Code from "../components/Code.jsx";
import Comparison from "../components/Comparison.jsx";
import Icon from "../components/Icon.jsx";
import Marked from "../components/Marked.jsx";
import Md from "../components/Md.jsx";
import Warranty from "../components/Warranty.jsx";
import { ANSWERS } from "../lib/answers.js";
import { ACCENTS, BUILT_WITH, DESCRIPTION, FEATURES, REPO_URL, SYNTAX, TRADEOFFS, WHY_NERVE } from "../lib/content.js";
import { plain } from "../lib/md.js";
import { around, runSearch } from "../lib/search.js";
import { useCodeIndex, useRecent } from "../lib/useSearch.js";
import { seo } from "../lib/seo.js";
import { LANGUAGE_ARTWORK } from "../lib/language-artwork.js";

// Have every menu logo ready by the time a comparison picker opens.
export const links = () => Object.values(LANGUAGE_ARTWORK).map((a) => ({ rel: "preload", as: "image", href: a.asset }));

// Structured data naming the site (owner, 2026-09-28): Google takes a result's site name from
// a WebSite block on the home page.
export const meta = () => [
  ...seo({ title: "Nerve", description: DESCRIPTION, path: "/" }),
  { "script:ld+json": { "@context": "https://schema.org", "@type": "WebSite", name: "Nerve", alternateName: "Nerve programming language", url: "https://nerve-lang.com/" } },
];

// Build time: one small index over everything worth finding.
export async function loader() {
  const { highlight, listExamples } = await import("../lib/examples.server.js");
  const pages = [
    { to: "/learn", title: "Learn", text: "Your first program, syntax in five minutes, core features", kind: "Page", icon: "book" },
    { to: "/examples", title: "Examples", text: "Every program in the repo, with its output", kind: "Page", icon: "code" },
    { to: "/download", title: "Download", text: "Nothing to download yet", kind: "Page", icon: "download" },
    { to: "/more", title: "RRx2 and RR", text: "Nerve’s own verifiers: RRx2, a translation validator, and RR, an exact-math language", kind: "Page", icon: "shield" },
    { to: "/commits", title: "Commits", text: "Every change to Nerve, newest first: each commit's message, who wrote it and when", kind: "Page", icon: "book" },
    { to: "/good-practice", title: "Good practice", text: "How Nerve is meant to be written: the practices its rules make easy, advice not rules", kind: "Page", icon: "check" },
    { to: "/licence", title: "Licence", text: "Apache-2.0 with the LLVM exception; text CC BY 4.0", kind: "Page", icon: "licence" },
  ];
  const features = FEATURES.map((f) => ({ to: `/learn#${f.id}`, title: plain(f.title), text: plain(f.text), kind: "Feature", icon: f.icon }));
  const lessons = SYNTAX.map((s, i) => ({ to: `/learn#syntax-${i + 1}`, title: plain(s.title), text: plain(s.text), kind: "Syntax", icon: "book" }));
  const { readPractice } = await import("../lib/practice.server.js");
  const practices = (await readPractice()).flatMap((g) => g.items.map((it) => ({ to: `/good-practice#p-${it.n}`, title: plain(it.title), text: plain(it.text), kind: "Practice", icon: "check" })));
  const examples = listExamples().map((e) => ({ to: `/examples/${e.id}`, title: e.title, text: e.summary || `examples/${e.id}.nrv`, kind: "Example", icon: "code", id: e.id }));
  // An answer's code must be a real line of its example, or the build stops.
  const { readFileSync } = await import("node:fs");
  const answers = await Promise.all(ANSWERS.map(async (a) => {
    if (!a.code) return null;   // an answer with nothing to quote (the compiler's commands)
    const src = readFileSync(`../../examples/${a.from}.nrv`, "utf8");
    for (const line of a.code.split("\n")) if (!src.includes(line)) throw new Error(`answers.js: "${line}" is not in examples/${a.from}.nrv`);
    return highlight(a.code);
  }));
  return { index: [...pages, ...features, ...lessons, ...practices, ...examples], answers };
}

// Tapped on the empty bar: things worth trying. Each one has an instant answer.
const TRY = ["let", "plus", "div.floor", "lend", "otherwise", "garbage collector", "concur"];

function Search({ q, setQ, onSubmit, onFocus, onBlur }) {
  return (
    <form className="search" role="search" onSubmit={(e) => { e.preventDefault(); onSubmit(); }}>
      <Icon name="search" size={17} />
      <input type="search" placeholder="Search Nerve — even code" value={q} onChange={(e) => setQ(e.target.value)} onFocus={onFocus} onBlur={onBlur}
        enterKeyHint="go" autoCapitalize="none" autoCorrect="off" spellCheck={false} aria-label="Search Nerve" />
    </form>
  );
}

// Keep the outgoing view until it has faded, then bring the next one in.
// The search form stays mounted and focused, and typing within results keeps
// the same key so it doesn't restart this transition on every letter.
function HomeView({ kind, className = "", children }) {
  const reducedMotion = useReducedMotion();
  return (
    <AnimatePresence initial={false} mode="wait">
      <motion.div key={kind} className={`home-view ${className}`}
        initial={{ opacity: 0, y: reducedMotion ? 0 : 6 }}
        animate={{ opacity: 1, y: 0, transition: { duration: reducedMotion ? 0 : 0.18 } }}
        exit={{ opacity: 0, y: reducedMotion ? 0 : -4, transition: { duration: reducedMotion ? 0 : 0.12 } }}>
        {children}
      </motion.div>
    </AnimatePresence>
  );
}

// Asking for the code gets a button straight to it (owner, 2026-09-25).
function SourceHit() {
  return (
    <a className="source-hit" href={REPO_URL} target="_blank" rel="noopener noreferrer">
      <span className="gh"><Icon name="github" size={24} stroke={1.8} /></span>
      <span><b>Source code on GitHub</b><small>Soclets/Nerve-lang</small></span>
      <Icon name="external" size={18} />
    </a>
  );
}

function Answer({ a, html, q, onGo }) {
  return (
    <Link to={a.to} className="answer" prefetch="intent" onClick={onGo} state={a.to.startsWith("/examples/") ? { fromGrid: true } : undefined}>
      {a.foreign?.includes(q) && <p className="foreign"><code>{q}</code> isn’t built in here.</p>}
      <b><Md text={a.title} /></b>
      <p><Md text={a.text} /></p>
      {html && <Code html={html} />}
      {a.from && <small>examples/{a.from}.nrv <Icon name="chev" size={14} /></small>}
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
    addEventListener("nerve:retap", clear);
    return () => removeEventListener("nerve:retap", clear);
  }, []);

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
        <HomeView kind="results" className="results">
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
        </HomeView>
      </div>
    );
  }

  if (focused) {
    return (
      <div className="home">
        {search}
        <HomeView kind="suggest">
        {recent.length > 0 && (
          <>
            <div className="suggest-head"><h2 className="section">Recent</h2><button onMouseDown={keep} onClick={clear}>Clear</button></div>
            <div className="chips">{recent.map((r) => <button key={r} onMouseDown={keep} onClick={() => setQ(r)}>{r}</button>)}</div>
          </>
        )}
        <h2 className="section">Try</h2>
        <div className="chips">{TRY.map((t) => <button key={t} className={/^[a-z ]+$/.test(t) ? "" : "mono"} onMouseDown={keep} onClick={() => setQ(t)}>{t}</button>)}</div>
        <p className="hint">Searches names, pages and every line of code in the examples. Enter opens the top result.</p>
        </HomeView>
      </div>
    );
  }

  return (
    <div className="home">
      {search}
      <HomeView kind="hero">
      <h1 className="huge">Wired into your <span>nerves</span></h1>
      <p className="lede">{DESCRIPTION}</p>
      <div className="actions">
        <Link className="btn primary" to="/download" prefetch="intent"><Icon name="download" size={18} stroke={2.3} />Download</Link>
        <Link className="btn" to="/learn" prefetch="intent">Read shit</Link>
        <a className="btn round" href={REPO_URL} target="_blank" rel="noopener noreferrer" aria-label="Source on GitHub"><Icon name="github" size={20} stroke={1.9} /></a>
      </div>
      <Warranty />
      <h2 className="section">Why “Nerve”</h2>
      <blockquote className="why">
        <p>{WHY_NERVE}</p>
        <cite>Tankun Sriket, who designs Nerve</cite>
      </blockquote>
      <h2 className="section">Features &amp; flaws</h2>
      <table className="tradeoffs">
        <thead><tr><th scope="col">What you get</th><th scope="col">What it costs</th></tr></thead>
        <tbody>
          {TRADEOFFS.map((t) => (
            <tr key={t.get}><td><Md text={t.get} /></td><td><Md text={t.cost} /></td></tr>
          ))}
        </tbody>
      </table>
      <h2 className="section">Built with</h2>
      <table className="tradeoffs built-with">
        <thead><tr><th scope="col">Part</th><th scope="col">Made with</th></tr></thead>
        <tbody>
          {BUILT_WITH.map((b) => (
            <tr key={b.part}><td><Md text={b.part} /></td><td><Md text={b.made} /></td></tr>
          ))}
        </tbody>
      </table>
      <Comparison />
      </HomeView>
    </div>
  );
}
