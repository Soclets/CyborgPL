// After the build: /preload.json, the list the loading screen downloads (app/loading/preload.ts),
// every file a visitor's browser would need: the code, styles, fonts and images, each with its
// size, so the bar can count real bytes. Pages' HTML is left out (moving between pages needs only
// their code), and so are the files only crawlers read.
import { readdirSync, statSync, writeFileSync } from "node:fs";
import { join, relative } from "node:path";

const OUT = "build/client";
const SKIP = /\.(html|txt|xml|json|map|webmanifest)$|^_headers$|^_redirects$/;
const list: { url: string; size: number }[] = [];
(function walk(dir: string): void {
  for (const name of readdirSync(dir)) {
    const full = join(dir, name);
    if (statSync(full).isDirectory()) { walk(full); continue; }
    if (SKIP.test(name) || name.startsWith(".")) continue;
    list.push({ url: "/" + relative(OUT, full).split("\\").join("/"), size: statSync(full).size });
  }
})(OUT);
list.sort((a, b) => a.url.localeCompare(b.url));
writeFileSync(join(OUT, "preload.json"), JSON.stringify(list));
const kb = list.reduce((s, e) => s + e.size, 0) / 1024;
console.log(`post-build: preload.json, ${list.length} files, ${kb.toFixed(0)} KB`);
