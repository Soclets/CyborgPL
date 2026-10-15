// Tokenises every examples/*.zg (once *.nrv) with this grammar (through the site's Shiki) and prints
// (1) every non-whitespace span with no scope of its own, and (2) for each scope, the distinct
// words it was given — the list to read by hand for misclassification.
// Needs website/zag's packages installed.  node website/grammar/audit.mjs [scope-filter]
import { readdirSync, readFileSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { createRequire } from "node:module";

const here = path.dirname(fileURLToPath(import.meta.url));
// Shiki lives in the Zag site's node_modules; resolve it from there.
const { createHighlighter } = await import(pathToFileURL(createRequire(path.join(here, "../zag/package.json")).resolve("shiki")));
const examples = path.resolve(here, "../../examples");
const grammar = JSON.parse(readFileSync(path.join(here, "nerve.tmLanguage.json"), "utf8"));
const hl = await createHighlighter({ themes: ["min-light"], langs: [grammar] });
const filter = process.argv[2];

const unscoped = [];
const byScope = new Map(); // scope -> Map(text -> [count, first place])
// Every program file, the program folders' too (their .zg files and Ricochet.zv manifests), since
// the Examples page shows them all.
const files = [];
(function walk(dir, rel) {
  for (const name of readdirSync(dir).sort()) {
    const full = path.join(dir, name), r = rel ? `${rel}/${name}` : name;
    if (statSync(full).isDirectory()) walk(full, r);
    else if (/\.(zg|zv|nrv)$/.test(name)) files.push(r);
  }
})(examples, "");
for (const f of files) {
  const src = readFileSync(path.join(examples, f), "utf8");
  const lines = hl.codeToTokens(src, { lang: "nerve", theme: "min-light", includeExplanation: "scopeName" }).tokens;
  lines.forEach((line, ln) => {
    for (const tok of line) for (const ex of tok.explanation ?? []) {
      const text = ex.content.trim();
      if (!text) continue;
      const scope = ex.scopes.at(-1).scopeName;
      if (scope === "source.nerve") { unscoped.push(`${f}:${ln + 1}  ${JSON.stringify(text)}`); continue; }
      if (scope.startsWith("comment") || scope.startsWith("string") || scope.startsWith("punctuation.definition.string")) continue;
      const m = byScope.get(scope) ?? new Map();
      const e = m.get(text) ?? [0, `${f}:${ln + 1}`];
      e[0]++; m.set(text, e); byScope.set(scope, m);
    }
  });
}
console.log(`${files.length} files. Unscoped spans: ${unscoped.length}`);
for (const u of unscoped) console.log("  " + u);
for (const [scope, m] of [...byScope].sort()) {
  if (filter && !scope.includes(filter)) continue;
  console.log(`\n${scope}`);
  console.log("  " + [...m].sort((a, b) => b[1][0] - a[1][0]).map(([t, [n, at]]) => `${t}×${n}` + (n === 1 ? `(${at})` : "")).join("  "));
}
