import { useState } from "react";
import { Link, Outlet, useLoaderData } from "react-router";
import Icon from "../components/Icon.jsx";
import { ACCENTS } from "../lib/content.js";

export const meta = () => [{ title: "Examples · CyborgPL" }];

// Build time: every program in examples/, with a few lines to preview.
export async function loader() {
  const { listExamples } = await import("../lib/examples.server.js");
  return { examples: listExamples() };
}

export default function Examples() {
  const { examples } = useLoaderData();
  const [q, setQ] = useState("");
  const needle = q.trim().toLowerCase();
  const shown = needle ? examples.filter((e) => (e.title + " " + e.summary + " " + e.id).toLowerCase().includes(needle)) : examples;
  return (
    <div className="examples">
      <h1 className="h2big">Examples</h1>
      <p className="lede small">Real programs from the repo, each with the output it really prints.</p>
      <label className="search">
        <Icon name="search" size={17} />
        <input type="search" placeholder={`Search ${examples.length} examples`} value={q} onChange={(e) => setQ(e.target.value)} enterKeyHint="search" />
      </label>
      <div className="grid">
        {shown.map((e, i) => (
          <Link key={e.id} to={e.id} state={{ fromGrid: true }} preventScrollReset prefetch="intent" className="gcard" style={{ "--c": ACCENTS[i % ACCENTS.length] }}>
            <pre className="gcode">{e.preview}</pre>
            <b>{e.title}</b>
            <small>{e.lines} lines</small>
          </Link>
        ))}
      </div>
      {!shown.length && <p className="empty">Nothing matches “{q}”.</p>}
      <Outlet />
    </div>
  );
}
