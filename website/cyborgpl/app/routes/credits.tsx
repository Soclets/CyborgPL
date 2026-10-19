import { Md } from "../components/md";
import ABOUT from "../generated/about.json";
import type { Route } from "./+types/credits";

export const meta: Route.MetaFunction = () => [
  { title: "Credits · CyborgPL" },
  { name: "description", content: "Who and what built CyborgPL." },
];

type Block = { kind: "p"; md: string } | { kind: "h"; md: string } | { kind: "ul"; items: string[] };

// Credits (owner, 2026-10-19: one of More's three). The README's credits, read when the site is
// built, then what only this site has: its own packages, and the compiler's crates, both from their
// lockfiles.
export default function Credits() {
  return (
    <main className="doc">
      <h1>Credits</h1>
      {(ABOUT.credits as Block[]).map((b, i) =>
        b.kind === "h" ? <h2 key={i}><Md text={b.md} /></h2>
        : b.kind === "p" ? <p key={i}><Md text={b.md} /></p>
        : <ul key={i}>{b.items.map((t, j) => <li key={j}><Md text={t} /></li>)}</ul>)}

      <h2>This website</h2>
      <p>Built with {ABOUT.direct.map((d, i) => <span key={d}>{i ? (i === ABOUT.direct.length - 1 ? " and " : ", ") : ""}<code>{d}</code></span>)}.</p>
      <details className="doc-list">
        <summary>Every package in its lockfile ({ABOUT.packages.length})</summary>
        <p>{ABOUT.packages.join(", ")}.</p>
      </details>

      <h2>Rust crates</h2>
      <p><code>KAYO\n</code>'s, from <code>kayo/Cargo.lock</code>: {ABOUT.crates.join(", ")}. The reference interpreter and the verification harness use none.</p>

      <p className="doc-small">{ABOUT.copyright}</p>
    </main>
  );
}
