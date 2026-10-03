// Home fetches this code index when someone starts searching. The Examples page
// reuses the full plain source it already carries for immediate popup text.
import { readFileSync, writeFileSync } from "node:fs";
import { exampleIds } from "../app/lib/examples.server.js";

const index = {};
for (const id of exampleIds()) {
  index[id] = readFileSync(`../../examples/${id}.cpls`, "utf8").split("\n").filter((l) => l.trim() && !l.trim().startsWith("#"));
}
writeFileSync("public/search-index.json", JSON.stringify(index));
console.log(`search-index: ${Object.keys(index).length} examples`);
