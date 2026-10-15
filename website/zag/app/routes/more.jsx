import { Link, useLoaderData } from "react-router";
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

// The full credits, as the README gives them (owner: "`more` should also have the full credit"). The
// crate and package lists are read from the lockfiles when the site is built, so they can't go stale.
export async function loader() {
  const { readFileSync } = await import("node:fs");
  const path = await import("node:path");
  const read = (f) => readFileSync(path.resolve(f), "utf8");
  // Every package in a package-lock.json, by name and version, once each.
  const npm = (f) => {
    const seen = new Set(), list = [];
    for (const [key, pkg] of Object.entries(JSON.parse(read(f)).packages)) {
      if (!key) continue;
      const name = key.slice(key.lastIndexOf("node_modules/") + 13), id = `${name} ${pkg.version}`;
      if (!seen.has(id)) { seen.add(id); list.push([name, pkg.version]); }
    }
    return list.sort((a, b) => (a[0] < b[0] ? -1 : a[0] > b[0] ? 1 : 0));
  };
  // The crates Zomp pulls in: the Cargo.lock entries with a source (its own have none).
  const crates = read("../../zomp/Cargo.lock").split("[[package]]").slice(1)
    .filter((b) => /^source = /m.test(b))
    .map((b) => [b.match(/^name = "(.*)"/m)[1], b.match(/^version = "(.*)"/m)[1]]);
  return { crates, site: npm("package-lock.json") };
}

// A long list of name and version pairs, run together as the README writes them.
function Versions({ list }) {
  return (
    <p className="versions">
      {list.map(([name, version], i) => (
        <span key={name + version}><code>{name}</code> {version}{i < list.length - 1 ? ", " : "."}</span>
      ))}
    </p>
  );
}


export default function More() {
  useSiteSky();
  const { crates, site } = useLoaderData();
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
          <p>Zag™ is designed by <b>Tankun Sriket</b>, who makes every decision in <a href={`${REPO}/blob/main/DECISIONS.md`} {...out}><code>DECISIONS.md</code></a>. Everything below helped build it.</p>
          <p><b>Claude</b>, by <a href="https://www.anthropic.com" {...out}>Anthropic</a>, wrote most of the code and documents here under the owner’s direction, as five <a href="https://claude.com/claude-code" {...out}>Claude Code</a> sessions: <i>design claude</i> (the design notes, the decision log, the reference interpreter and the benchmarks in <code>bench/</code>), <i>compiler claude</i> (Zomp, in <code>zomp/</code>), <i>std claude</i> (the standard library, written in Zag), <i>check claude</i> (the program generator, differential testing and translation validation in <code>check/</code>) and <i>website claude</i> (<code>website/</code>, this site). Two more work in their own repos: <i>synthia claude</i> on <a href="https://github.com/Cyborg-Assembly-Manufacturing-Site/Synthia" {...out}>Synthia</a>, and <i>RRx2-TV claude</i> on RRx2’s translation validator. Each signs its commits with its house name; the models behind them include Claude Opus 5 and Claude Opus 5.5.</p>

          <h3>Languages and tools</h3>
          <ul className="plain">
            <li><a href="https://www.rust-lang.org" {...out}>Rust</a> and Cargo: the reference interpreter, Zomp and the verification harness.</li>
            <li><a href="https://llvm.org" {...out}>LLVM</a> 22: Zomp’s code generator, through <a href="https://crates.io/crates/llvm-sys" {...out}>llvm-sys</a> and <a href="https://github.com/TheDan64/inkwell" {...out}>Inkwell</a>.</li>
            <li><a href="https://github.com/Z3Prover/z3" {...out}>Z3</a> (Microsoft Research) and <a href="https://cvc5.github.io" {...out}>cvc5</a>: the SMT solvers behind Zomp’s proofs, and the independent check of them.</li>
            <li><a href="https://github.com/AliveToolkit/alive2" {...out}>Alive2</a>: check claude’s first translation validation, proving LLVM’s rewrites of Zomp’s code.</li>
            <li><a href="https://clang.llvm.org" {...out}>Clang</a>: the C programs Zomp is benchmarked against, and the C side of the GPU benchmark.</li>
            <li>Metal and Xcode’s Metal Toolchain, by Apple: the GPU side of the GPU benchmark.</li>
            <li>The compilers and runtimes of the benchmark’s other languages (26 with C): C++, Rust, Go, Zig, Swift, Nim, Fortran (gfortran), OCaml, Dart, Java, Kotlin, C# (.NET), JavaScript and TypeScript (Node), Julia, LuaJIT, PyPy, Elixir, Erlang, Lust, Python, Ruby, Lua, Perl and R.</li>
            <li><a href="https://www.python.org" {...out}>Python</a>: the benchmark runners.</li>
            <li><a href="https://github.com/djrrb/Bungee" {...out}>Bungee</a>, by The Bungee Project Authors (SIL Open Font License 1.1): the letters of the Zag logo, turned into outlines.</li>
            <li><a href="https://github.com/hafontia/Fredoka-One" {...out}>Fredoka</a>, by The Fredoka Project Authors (SIL Open Font License 1.1): this site’s text.</li>
            <li><a href="https://nodejs.org" {...out}>Node.js</a> and npm: this site’s build.</li>
            <li><a href="https://brew.sh" {...out}>Homebrew</a>: LLVM, Z3, cvc5, Alive2 and most of the benchmark’s toolchains on the build machine.</li>
            <li><a href="https://git-scm.com" {...out}>Git</a>, <a href="https://github.com" {...out}>GitHub</a> and the <a href="https://cli.github.com" {...out}>GitHub CLI</a>: hosting and history.</li>
            <li><a href="https://www.cloudflare.com" {...out}>Cloudflare</a> Workers and Wrangler: this site.</li>
            <li>macOS and Xcode’s command-line tools, by Apple.</li>
          </ul>

          <h3>Data</h3>
          <ul className="plain">
            <li>The <a href="https://www.unicode.org/ucd/" {...out}>Unicode Character Database</a> 17.0.0 and <a href="https://www.unicode.org/reports/tr29/" {...out}>UAX #29</a>: grapheme clusters in <code>str</code> and <code>char</code>.</li>
            <li>The comparison on <Link to="/#home">Home</Link> draws on each language’s own documentation, named under the table.</li>
          </ul>

          <h3>Rust crates</h3>
          <p className="small">Zomp’s, from <code>zomp/Cargo.lock</code>. The reference interpreter and the verification harness use no crates.</p>
          <Versions list={crates} />

          <h3>Website packages</h3>
          <p>This site: <a href="https://react.dev" {...out}>React</a>, <a href="https://reactrouter.com" {...out}>React Router</a>, <a href="https://github.com/omrilotan/isbot" {...out}>isbot</a>, <a href="https://vite.dev" {...out}>Vite</a>, <a href="https://shiki.style" {...out}>Shiki</a> (with its TextMate grammar engine), which colours the code, and Cloudflare’s <a href="https://developers.cloudflare.com/workers/wrangler/" {...out}>Wrangler</a>.</p>
          <details className="every">
            <summary>Every package in this site’s <code>package-lock.json</code> ({site.length})</summary>
            <Versions list={site} />
          </details>

          <h3>Ideas taken from other languages</h3>
          <ul className="plain">
            <li><b>Rust</b>: ownership and borrowing, lifetime groups (<code>'a</code> there, <code>@x</code> here), interfaces and <code>{"<T: Ordered>"}</code> bounds (Rust’s traits), diagnostics “helpful like Rust’s”, <code>::</code> for stepping into another strike (Rust’s paths, which took it from C++), and, as spellings of Zag’s words, <code>&amp;</code> and <code>&amp;mut</code> for <code>lend</code> and <code>lend.mut</code> (and <code>lent</code>, <code>lent.mut</code> on a parameter), <code>*const T</code> and <code>*mut T</code> for the pointer types.</li>
            <li><b>C++</b>: <code>incl.C</code>, reading a C header, from <code>{"#include <>"}</code> (owner: “I wanna add C++’s <code>{"#include <>"}</code>”).</li>
            <li><b>C</b>: <code>'i' += 1;</code> and <code>'i'++;</code> as spellings of <code>change 'i' = 'i' plus 1;</code>; with <b>Rust</b> and <b>Go</b>, <code>/</code> and <code>%</code> rounding toward zero (<code>div.trunc</code>, <code>mod.trunc</code>).</li>
            <li><b>Python</b>: <code>**</code> for <code>pow</code>.</li>
            <li><b>Ada</b>: <code>and then</code> / <code>or else</code>, here <code>and.then</code> / <code>or.else</code>.</li>
            <li><b>Swift</b>, <b>Zig</b> and <b>C</b>: points of comparison in the SIMD, wrapping-arithmetic and enum decisions.</li>
            <li><b>IEEE 754</b>: <code>bin16</code>, <code>bin32</code> and <code>bin64</code> arithmetic and rounding.</li>
            <li><b>Google Brain</b>: <code>brain16</code> is bfloat16, the 16-bit format from Google Brain.</li>
            <li><b>Lua</b>: the name <i>Lunae</i>.</li>
            <li><b>Kespar</b>, another of the owner’s languages: the prover’s design notes and its first provisional limits.</li>
            <li><b>Xag</b>, the owner’s C++ language: the reference for everything outside the prover, and the licence.</li>
          </ul>
          <p className="small">Copyright 2026 Tankun Sriket.</p>
        </section>
      </div>
    </main>
  );
}
