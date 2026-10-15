// Before the build: the data the pages show, worked out once and written to app/generated/, so
// each page's code carries its own data and the loading screen can fetch every page whole (owner,
// 2026-10-15: "make it a real loading screen"). Safari ignores prefetch hints, so a page whose
// data came from a loader would still wait on a request when clicked.
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { createHighlighter } from "shiki";
import { HOME_EXAMPLES } from "../app/content.js";
import { ZAG_THEME } from "../app/zag-theme.js";

const read = (f) => readFileSync(resolve(f), "utf8");
const out = "app/generated";
mkdirSync(out, { recursive: true });

// Home: the examples, highlighted with the site's grammar, with their expected output.
const grammar = JSON.parse(read("../grammar/nerve.tmLanguage.json"));
const hl = await createHighlighter({ themes: [ZAG_THEME], langs: [grammar] });
const dir = resolve("../../examples");
const examples = HOME_EXAMPLES.map((e) => {
  const source = readFileSync(join(dir, `${e.id}.zg`), "utf8").replace(/\n$/, "");
  const lines = source.split("\n").length;
  return { ...e, note: e.note.replace("{lines}", String(lines)), file: `${e.id}.zg`, html: hl.codeToHtml(source, { lang: "nerve", theme: "zag-night" }), out: readFileSync(join(dir, `${e.id}.out`), "utf8").replace(/\n$/, "") };
});
writeFileSync(`${out}/home.json`, JSON.stringify({ examples }));

// More: the crate and package lists, read from the lockfiles so they can't go stale.
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
writeFileSync(`${out}/more.json`, JSON.stringify({ crates, site: npm("package-lock.json") }));

console.log(`pre-build: ${examples.length} examples, ${crates.length} crates, home.json and more.json`);
