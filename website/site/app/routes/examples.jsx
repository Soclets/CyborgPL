import { useEffect, useMemo, useState } from "react";
import { Link, Outlet, useLoaderData } from "react-router";
import Icon from "../components/Icon.jsx";
import Marked from "../components/Marked.jsx";
import { ACCENTS } from "../lib/content.js";
import { around, runSearch } from "../lib/search.js";
import { seo } from "../lib/seo.js";

export const meta = () => seo({ title: "Examples · CyborgPL", description: "Real CyborgPL programs from the repo, each with the output it really prints.", path: "/examples" });

// Build time: every program, its card preview and its full plain source.
export async function loader() {
  const { listExamples } = await import("../lib/examples.server.js");
  return { examples: listExamples() };
}

// The query outlives the page, so coming back from a result finds it still there.
let lastQ = "";

// The example window's code (and the animation library in it) is fetched once the grid is
// idle, so even the first opening doesn't wait for it. Asked of the route manifest, so it is
// exactly the built chunk.
function useWarmExampleWindow() {
  useEffect(() => {
    const warm = () => {
      const m = window.__reactRouterManifest?.routes?.["routes/example"];
      if (m?.module) import(/* @vite-ignore */ m.module).catch(() => {});
    };
    const id = "requestIdleCallback" in window ? requestIdleCallback(warm, { timeout: 2000 }) : setTimeout(warm, 800);
    return () => ("cancelIdleCallback" in window ? cancelIdleCallback(id) : clearTimeout(id));
  }, []);
}

export default function Examples() {
  useWarmExampleWindow();
  const { examples } = useLoaderData();
  const [q, setQState] = useState(lastQ);
  const setQ = (v) => { lastQ = v; setQState(v); };
  // A second tap on this tab, already at the top, clears the search.
  useEffect(() => {
    const clear = () => { setQ(""); document.activeElement?.blur(); };
    addEventListener("cyborgpl:retap", clear);
    return () => removeEventListener("cyborgpl:retap", clear);
  }, []);
  // The source is already here for instant opening; reuse it for code search.
  const code = useMemo(() => Object.fromEntries(examples.map((e) => [e.id,
    e.source.split("\n").filter((l) => l.trim() && !l.trim().startsWith("#")),
  ])), [examples]);
  // Same engine as Home: names, summaries, typos, synonyms, and every line of code.
  const entries = useMemo(() => examples.map((e) => ({ ...e, text: e.summary })), [examples]);
  const found = runSearch(entries, q, code);
  const shown = found.q ? found.results.map((r) => ({ ...r.entry, line: r.line })) : examples;
  return (
    <div className="examples">
      <h1 className="h2big">Examples</h1>
      <p className="lede small">Real programs from the repo, each with the output it really prints.</p>
      <label className="search">
        <Icon name="search" size={17} />
        <input type="search" placeholder={`Search ${examples.length} examples and their code`} value={q} onChange={(e) => setQ(e.target.value)}
          enterKeyHint="search" autoCapitalize="none" autoCorrect="off" spellCheck={false} />
      </label>
      <div className="grid">
        {shown.map((e, i) => (
          <Link key={e.id} to={e.id} state={{ fromGrid: true }} preventScrollReset prefetch="intent" className="gcard" style={{ "--c": ACCENTS[i % ACCENTS.length] }}>
            {e.line ? <pre className="gcode gline"><Marked text={around(e.line, found.q)} q={found.q} /></pre> : <pre className="gcode">{e.preview}</pre>}
            <b><Marked text={e.title} q={found.q} /></b>
            <small>{e.lines} lines</small>
          </Link>
        ))}
      </div>
      {!shown.length && <p className="empty">Nothing matches “{q}”.</p>}
      <Outlet />
    </div>
  );
}
