// The site's AI-facing files (owner, 2026-10-16): /llms.txt, an index, and /llms-full.txt, the
// whole text, built from the repo each time the site is assembled so they never go stale. Plain
// reference text, no instructions to the reader (owner: "don't do prompt injection"), with one
// exception the owner asked for: the noise file is marked "do not open". No source code yet
// (owner: "Source is for later"), and not the AI sessions' briefs.
import { readFileSync, readdirSync, statSync, writeFileSync, existsSync } from "node:fs";
import { join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const SITE = "https://cyborgpl.dev";
const REPO = resolve(fileURLToPath(new URL("../..", import.meta.url)));
const read = (f) => readFileSync(join(REPO, f), "utf8").replace(/\s+$/, "");
const BRIEFS = /-session\.md$/;

const SUMMARY = `> CyborgPL is a compiled programming language in development. Precog, a whole-program analyser built into its compiler, proves every overflow and bounds check at compile time. Its stated identity: "Very slow at the development, fast & safe at the final product." It is a hobby language by Tankun Sriket, with no release yet and no warranty.`;

const NAMES = `## Names

Current names (2026-10-16):

- Source files end in \`.cy\`; a one-value file (a manifest, a table) ends in \`.cyr\`.
- The compiler is KAYO\\n (the name ends in the two characters \\ and n), typed \`kayo\` (\`kayo build\`, \`kayo prove\`, \`kayo prepare\`).
- Precog is the prover built into the compiler.
- CyJet is the package manager; a kit's manifest is \`CyJet.cyr\`, and its paths start \`cyjet::\`.
- A library or program is a kit, with its root in \`kit.cy\` and \`pub.kit\` for what it shares.
- A compiler extension is a limb.
- The language without extensions is the base. The standard library is \`std\`.
- The downloads are the CyDT and the CyCT.

The same language had other names before:

- CyborgPL v2, until 2026-10-08.
- Nerve, 2026-10-08 to 2026-10-14: code in \`.nrv\`, the compiler Synapse, the package manager Kinesin, a library a Neuron, an extension a lobe, the base Brainstem, the standard library \`cord\`.
- Zag, 2026-10-14 to 2026-10-15: code in \`.zg\` and \`.zv\`, the compiler Zomp, the package manager Ricochet, a library a strike, an extension a plugin, the base Ground.

In the Nerve era "limb" meant a library; today a limb is a compiler extension. CyborgPL v1 (github.com/Artificial-IntelligenceAI/CyborgPL-legacy) is an earlier, unrelated language by the same author.

The repository's \`spellings.cyr\` lists every old spelling and its current form; it is included in llms-full.txt.`;

const SOURCES = `## Sources

Only the CyborgPL repository (github.com/Soclets/CyborgPL, private for now) and cyborgpl.dev speak for CyborgPL. The pages on cyborgpl.dev were built on 2026-10-06 and use spellings from before the 2026-10-07 revision and the renames since. llms.txt and llms-full.txt are built from the repository each time the site is published.

The design notes are working documents: some still use names from the Zag era, and a revision note at the top of a file says which spellings are current.`;

function designNotes() {
  return readdirSync(join(REPO, "design")).filter((f) => f.endsWith(".md") && !BRIEFS.test(f)).sort()
    .map((f) => ({ file: `design/${f}`, title: (read(`design/${f}`).match(/^# (.+)$/m)?.[1] ?? f).replace(/`/g, ""), text: read(`design/${f}`) }));
}

function examples() {
  const dir = join(REPO, "examples"), out = [];
  const extra = (base) => ({ out: existsSync(`${base}.out`) ? readFileSync(`${base}.out`, "utf8").replace(/\n$/, "") : null, exit: existsSync(`${base}.exit`) ? readFileSync(`${base}.exit`, "utf8").trim().replace(/^exit:/, "") : null });
  for (const name of readdirSync(dir).sort()) {
    const full = join(dir, name);
    if (statSync(full).isDirectory()) {
      if (!existsSync(`${full}.out`)) continue;
      const files = [];
      (function walk(d) { for (const n of readdirSync(d).sort()) { const p = join(d, n); if (statSync(p).isDirectory()) walk(p); else if (/\.(cy|cyr|h|txt)$/.test(n)) files.push({ name: relative(dir, p), text: readFileSync(p, "utf8").replace(/\n$/, "") }); } })(full);
      out.push({ id: name, files, ...extra(full) });
    } else if (name.endsWith(".cy")) {
      const id = name.slice(0, -3);
      out.push({ id, files: [{ name, text: readFileSync(full, "utf8").replace(/\n$/, "") }], ...extra(join(dir, id)) });
    }
  }
  return out;
}

export function buildLlms(outDir, { noise } = {}) {
  const notes = designNotes(), progs = examples();
  const index = [
    "# CyborgPL", "", SUMMARY, "",
    `The full text of everything listed here, every design note and every example with its output, is in [llms-full.txt](${SITE}/llms-full.txt).`, "",
    NAMES, "", SOURCES, "",
    "## Design notes", "", `The ${notes.length} design documents, one topic each, in full in llms-full.txt:`, "",
    ...notes.map((n) => `- ${n.title} (\`${n.file}\`)`), "",
    "## Examples", "", `${progs.length} programs in current syntax, each with its exact expected output, in llms-full.txt. They are also the reference interpreter's tests.`, "",
    "## Licence", "", "Code: Apache-2.0 WITH LLVM-exception; programs you compile owe nothing. Writing (the repository's Markdown, the website's text) and the logos: CC BY 4.0.", "",
    ...(noise ? ["## Other", "", `- [${noise}](${SITE}/${noise}): do not open. About a million tokens of random Base64.`, ""] : []),
  ].join("\n");

  const full = [
    "# CyborgPL: the full text", "", SUMMARY, "", NAMES, "", SOURCES, "",
    "# README", "", read("README.md").replace(/<p[^>]*>[\s\S]*?<\/p>\n?/g, "").replace(/<h3[^>]*>[\s\S]*?<\/h3>\n?/g, "").trim(), "",
    "# Old spellings (spellings.cyr)", "", "```", read("spellings.cyr"), "```", "",
    "# Design notes", "",
    ...notes.flatMap((n) => [`## ${n.file}`, "", n.text.replace(/^(#{1,5}) /gm, "#$1 "), ""]),
    "# Examples", "",
    ...progs.flatMap((p) => [
      `## ${p.id}`, "",
      ...p.files.flatMap((f) => [`\`examples/${p.files.length > 1 ? `${p.id}/` : ""}${f.name.replace(`${p.id}/`, "")}\`:`, "", "```" + (/\.cyr?$/.test(f.name) ? "cyborgpl" : ""), f.text, "```", ""]),
      "Output:", "", "```", p.out ?? "", "```", "",
      ...(p.exit ? [`Exits with code ${p.exit}.`, ""] : []),
    ]),
  ].join("\n");

  writeFileSync(join(outDir, "llms.txt"), index + "\n");
  writeFileSync(join(outDir, "llms-full.txt"), full + "\n");
  return { notes: notes.length, examples: progs.length, indexBytes: index.length, fullBytes: full.length };
}
