import { useState } from "react";
import { Link, Outlet, useLoaderData } from "react-router";
import Icon from "../components/Icon.jsx";
import Marked from "../components/Marked.jsx";
import { ACCENTS } from "../lib/content.js";
import { around, runSearch } from "../lib/search.js";
import { useCodeIndex } from "../lib/useSearch.js";

export const meta = () => [{ title: "Examples · CyborgPL" }];

// Build time: every program in examples/, with a few lines to preview.
export async function loader() {
  const { listExamples } = await import("../lib/examples.server.js");
  return { examples: listExamples() };
}

// The query outlives the page, so coming back from a result finds it still there.
let lastQ = "";

export default function Examples() {
  const { examples } = useLoaderData();
  const [q, setQState] = useState(lastQ);
  const setQ = (v) => { lastQ = v; setQState(v); };
  const [focused, setFocused] = useState(false);
  const code = useCodeIndex(focused || !!q);
  // Same engine as Home: names, summaries, typos, synonyms, and every line of code.
  const found = runSearch(examples.map((e) => ({ ...e, text: e.summary })), q, code);
  const shown = found.q ? found.results.map((r) => ({ ...r.entry, line: r.line })) : examples;
  return (
    <div className="examples">
      <h1 className="h2big">Examples</h1>
      <p className="lede small">Real programs from the repo, each with the output it really prints.</p>
      <label className="search">
        <Icon name="search" size={17} />
        <input type="search" placeholder={`Search ${examples.length} examples and their code`} value={q} onChange={(e) => setQ(e.target.value)} onFocus={() => setFocused(true)}
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
