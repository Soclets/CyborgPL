// Before the build: the home page's three examples (owner, 2026-10-19: "3 examples: Hello World, C
// stuff, and the biggest"), read from the repo's examples/ with what each prints, so the page always
// shows them as they are. Plain text for now: no highlighting (owner: "No syntax highlighting for now").
import { mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const DIR = "../../examples";
const read = (f: string) => readFileSync(join(DIR, f), "utf8").replace(/\n$/, "");
// The biggest is whichever single-file program is longest, so it follows the folder.
const biggest = readdirSync(DIR).filter((f) => f.endsWith(".cy")).sort((a, b) => statSync(join(DIR, b)).size - statSync(join(DIR, a)).size)[0]!;
const picks = [
  { id: "hello", tab: "Hello World" },
  { id: "c_headers", tab: "C" },
  { id: biggest.slice(0, -3), tab: "The biggest" },
];
const examples = picks.map((p) => {
  const source = read(`${p.id}.cy`);
  return { ...p, file: `${p.id}.cy`, lines: source.split("\n").length, source, out: read(`${p.id}.out`) };
});
mkdirSync("app/generated", { recursive: true });
writeFileSync("app/generated/examples.json", JSON.stringify(examples));
console.log(`pre-build: examples ${examples.map((e) => `${e.file} (${e.lines} lines)`).join(", ")}`);
