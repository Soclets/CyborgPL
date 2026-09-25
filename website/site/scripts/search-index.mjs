// The code half of search: every example's code lines, fetched by the browser only
// when someone starts searching, so no page carries it up front.
import { readFileSync, writeFileSync } from "node:fs";
import { exampleIds } from "../app/lib/examples.server.js";

const index = {};
for (const id of exampleIds()) {
  index[id] = readFileSync(`../../examples/${id}.cpls`, "utf8").split("\n").filter((l) => l.trim() && !l.trim().startsWith("#"));
}
writeFileSync("public/search-index.json", JSON.stringify(index));
console.log(`search-index: ${Object.keys(index).length} examples`);
