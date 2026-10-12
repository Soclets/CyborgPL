// Every example's highlighted code and output in one file, which the Examples grid loads in the
// background so an example opens complete, with no wait (owner, 2026-10-12: "there shouldn't be a
// delay for syntax highlighting and the copy button"). About 80 KB compressed; kept out of the
// pages themselves, which would otherwise each carry all of it.
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { exampleIds, highlight } from "../app/lib/examples.server.js";

const all = {};
for (const id of exampleIds()) {
  const src = readFileSync(`../../examples/${id}.nrv`, "utf8").replace(/\n$/, "");
  const exit = existsSync(`../../examples/${id}.exit`) ? readFileSync(`../../examples/${id}.exit`, "utf8").trim() : null;
  all[id] = { html: await highlight(src), source: src, out: readFileSync(`../../examples/${id}.out`, "utf8").replace(/\n$/, ""), exit };
}
writeFileSync("public/examples-full.json", JSON.stringify(all));
console.log(`examples-full: ${Object.keys(all).length} examples`);
