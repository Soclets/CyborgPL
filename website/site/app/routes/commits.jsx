import { useEffect, useMemo, useState } from "react";
import { Link, useLoaderData } from "react-router";
import Md from "../components/Md.jsx";
import { seo } from "../lib/seo.js";

export const meta = () => seo({ title: "Commits · Nerve", description: "Every change to Nerve, newest first: each commit's message, who wrote it and when, without the code.", path: "/commits" });

// A mirror of the repo's commit history (owner, 2026-10-12), messages only. The build writes it all
// to /commits.json (scripts/commits.mjs); the page arrives with the newest commits and fetches the
// rest straight after.
const FIRST = 40;
export async function loader() {
  const { readFileSync } = await import("node:fs");
  const all = JSON.parse(readFileSync("public/commits.json", "utf8"));
  return { first: all.slice(0, FIRST), total: all.length };
}

// The house names the sessions sign with, and how many commits each wrote.
const by = (c) => (c.by.length ? c.by : [c.a]);
// "kind.folder.narrowing: sentence" (DECISIONS.md), split so the kind can be shown on its own.
const parts = (s) => s.match(/^([a-z]+)\.([^\s:]+):\s*(.*)$/s);
const day = (d) => d.slice(0, 10);

export default function Commits() {
  const { first, total } = useLoaderData();
  const [all, setAll] = useState(first);
  const [q, setQ] = useState("");
  const [who, setWho] = useState("");
  const [shown, setShown] = useState(60);
  useEffect(() => {
    let live = true;
    fetch("/commits.json").then((r) => r.json()).then((j) => live && setAll(j)).catch(() => {});
    return () => { live = false; };
  }, []);
  useEffect(() => setShown(60), [q, who]);

  const people = useMemo(() => {
    const n = new Map();
    for (const c of all) for (const p of by(c)) n.set(p, (n.get(p) ?? 0) + 1);
    return [...n].sort((a, b) => b[1] - a[1]);
  }, [all]);
  const list = useMemo(() => {
    const t = q.trim().toLowerCase();
    return all.filter((c) => (!who || by(c).includes(who)) && (!t || c.s.toLowerCase().includes(t) || c.b.toLowerCase().includes(t) || c.h.startsWith(t)));
  }, [all, q, who]);

  return (
    <div className="plain commits">
      <Link to="/more" className="crumb">‹ More</Link>
      <h1 className="h2big">Commits</h1>
      <p className="lede small">Every change to Nerve, newest first: what each commit says, who wrote it and when. The code itself isn’t shown. {total} commits.</p>
      <input className="commit-search" type="search" placeholder="Search the messages, or a commit’s id" value={q} onChange={(e) => setQ(e.target.value)} aria-label="Search commits" />
      <div className="chips commit-people">
        <button className={who === "" ? "on" : ""} onClick={() => setWho("")}>Everyone</button>
        {people.map(([p, n]) => <button key={p} className={who === p ? "on" : ""} onClick={() => setWho(who === p ? "" : p)}>{p} <small>{n}</small></button>)}
      </div>
      {all.length < total && <p className="commit-note">Loading the rest of the history…</p>}
      <ol className="commit-list">
        {list.slice(0, shown).map((c) => {
          const m = parts(c.s);
          return (
            <li key={c.h} className="commit">
              <div className="commit-head">
                {m ? <><span className={`commit-kind k-${m[1]}`}>{m[1]}</span><code className="commit-where">{m[2]}</code><b><Md text={m[3]} /></b></> : <b><Md text={c.s} /></b>}
              </div>
              <div className="commit-meta"><time dateTime={c.d}>{day(c.d)}</time><code>{c.h}</code><span>{by(c).join(", ")}</span></div>
              {c.b && <details><summary>Message</summary><p className="commit-body">{c.b}</p></details>}
            </li>
          );
        })}
      </ol>
      {list.length === 0 && <p className="empty">No commit matches.</p>}
      {list.length > shown && <button className="commit-more" onClick={() => setShown(shown + 100)}>Show more ({list.length - shown} left)</button>}
    </div>
  );
}
