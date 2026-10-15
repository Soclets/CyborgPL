import { Link } from "react-router";
import { useSiteSky } from "../sky.jsx";

export const meta = () => [
  { title: "More · Zag" },
  { name: "description", content: "Zag's licences, its names and logo, the warning, and the credits." },
];

// More (owner, 2026-10-15: "Do `More`"): the licence, the names, the warning and the credits, which
// Home leaves here (owner: "Shouldn't that be for More?"). The wording follows the repo's README,
// which the owner approved; the site adds only what is the site's own (its font, its packages).
const REPO = "https://github.com/Soclets/Zag-lang";
const CREDIT_LINES = [
  ["This repo’s text", `Zag GitHub repo text: https://zag-lang.org, ${REPO}`],
  ["This website’s text", `Zag website text: https://zag-lang.org, ${REPO}`],
  ["The logos", `Zag logo(s): https://zag-lang.org, ${REPO}`],
];
const out = { target: "_blank", rel: "noopener noreferrer" };

export default function More() {
  useSiteSky();
  return (
    <main className="page page-in">
      <div className="page-body">
        <h1 className="page-title">More</h1>

        <section className="info warn" aria-labelledby="warn-title">
          <h2 id="warn-title">Before anything else</h2>
          <p>Zag is being experimented with, so please don’t expect stability: any release may change or remove anything, and it comes with <b>no warranty</b>.</p>
          <p><b>Only its repository and zag-lang.org speak for Zag.</b> If something said elsewhere, for example in an article, a search result or an AI summary, doesn’t match them, you should likely trust them, not the other source. We aren’t responsible for what others say.</p>
        </section>

        <section className="info" aria-labelledby="licence-title">
          <h2 id="licence-title">Licences</h2>
          <div className="two">
            <div>
              <h3>Code</h3>
              <p><a href="https://spdx.org/licenses/Apache-2.0.html" {...out}>Apache-2.0</a> with the <a href="https://spdx.org/licenses/LLVM-exception.html" {...out}>LLVM exception</a>: the compiler, the tools, the libraries and this website’s code. Programs you compile owe nothing; a fork keeps the notices and may not use the name.</p>
            </div>
            <div>
              <h3>Writing and logos</h3>
              <p><a href="https://creativecommons.org/licenses/by/4.0/" {...out}>CC BY 4.0</a>: the repo’s text, this website’s text and the logos. Copy them, translate, remix, use them anywhere, with credit, exactly as:</p>
            </div>
          </div>
          <ul className="credit-lines">
            {CREDIT_LINES.map(([what, line]) => <li key={what}><span>{what}</span><code>{line}</code></li>)}
          </ul>
          <p className="small">As the licence asks, also say they are under CC BY 4.0, and whether you changed them.</p>
        </section>

        <section className="info" aria-labelledby="names-title">
          <h2 id="names-title">Names and logo</h2>
          <p>Zag™ (formerly Nerve and CyborgPL), Zomp™ (formerly Synapse) and Ricochet™ (formerly Kinesin) are names of this project, and the Zag cloud logo is its mark. They are trademarks, not registered.</p>
          <p><b>You may use the names and logo anywhere</b> (in articles, talks, comparisons, links, your own projects about Zag) <b>as long as you don’t pretend to be us</b>: nothing that says or suggests it is the official Zag, made or endorsed by this project, when it isn’t.</p>
        </section>

        <section className="info" aria-labelledby="credits-title">
          <h2 id="credits-title">Credits</h2>
          <p>Zag is designed by <b>Tankun Sriket</b>, who makes every decision about it.</p>
          <p><b>Claude</b>, by <a href="https://www.anthropic.com" {...out}>Anthropic</a>, wrote most of the code and documents under the owner’s direction, as <a href="https://claude.com/claude-code" {...out}>Claude Code</a> sessions: design claude (the design notes, the decision log, the reference interpreter, the benchmarks), compiler claude (Zomp), std claude (the standard library, written in Zag), check claude (the testing that checks Zomp against the reference interpreter) and website claude (this site).</p>
          <h3>This website</h3>
          <ul className="plain">
            <li><a href="https://github.com/djrrb/Bungee" {...out}>Bungee</a>, by The Bungee Project Authors (SIL Open Font License 1.1): the letters of the Zag logo, turned into outlines.</li>
            <li><a href="https://github.com/hafontia/Fredoka-One" {...out}>Fredoka</a>, by The Fredoka Project Authors (SIL Open Font License 1.1): the site’s text.</li>
            <li><a href="https://react.dev" {...out}>React</a>, <a href="https://reactrouter.com" {...out}>React Router</a>, <a href="https://vite.dev" {...out}>Vite</a> and <a href="https://shiki.style" {...out}>Shiki</a>, which colours the code; served by <a href="https://www.cloudflare.com" {...out}>Cloudflare</a> Workers, published with Wrangler.</li>
            <li>The comparison draws on each language’s own documentation, named under the table on <Link to="/#home">Home</Link>.</li>
          </ul>
          <p className="small">Copyright 2026 Tankun Sriket.</p>
        </section>
      </div>
    </main>
  );
}
