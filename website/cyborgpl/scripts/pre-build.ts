// Before the build: the home page's three examples (owner, 2026-10-19: "3 examples: Hello World, C
// stuff, and the biggest"), read from the repo's examples/ with what each prints, so the page always
// shows them as they are. Highlighted with the site's grammar (website/grammar, owner: "do syntax
// highlighting"): each piece of code is stored with its kind, not a colour, so the page and the
// fullscreen Solarized view can colour the same code their own way (app.css).
import { mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { createHighlighter } from "shiki";

const DIR = "../../examples";
const read = (f: string) => readFileSync(join(DIR, f), "utf8").replace(/\n$/, "");
// The biggest is whichever single-file program is longest, so it follows the folder.
const biggest = readdirSync(DIR).filter((f) => f.endsWith(".cy")).sort((a, b) => statSync(join(DIR, b)).size - statSync(join(DIR, a)).size)[0]!;
const picks = [
  { id: "hello", tab: "Hello World" },
  { id: "c_headers", tab: "C" },
  { id: biggest.slice(0, -3), tab: "The biggest" },
];
// The kind of a piece of code, from the most specific scope the grammar gave it.
const KINDS: [RegExp, string][] = [
  [/^invalid/, "old"], [/comment/, "comment"], [/^constant\.character\.escape/, "escape"], [/^(string|punctuation\.definition\.string)/, "string"],
  [/^constant/, "number"], [/^keyword\.operator/, "operator"], [/^(keyword|storage|variable\.language)/, "keyword"],
  [/^(entity\.name\.type|support\.type)/, "type"], [/^entity\.name\.function/, "function"], [/^entity\.name\.namespace/, "namespace"],
  [/^entity\.name\.tag/, "tag"], [/^variable\.other\.(member|enummember)/, "member"], [/^variable/, "name"], [/^punctuation/, "punct"],
];
const kindOf = (scope: string) => KINDS.find(([re]) => re.test(scope))?.[1] ?? "";
const grammar = JSON.parse(readFileSync("../grammar/nerve.tmLanguage.json", "utf8"));
const hl = await createHighlighter({ themes: ["min-light"], langs: [grammar] });
type Piece = [string, string];
const highlight = (source: string): Piece[][] =>
  hl.codeToTokens(source, { lang: "nerve" as never, theme: "min-light", includeExplanation: "scopeName" }).tokens.map((line) => {
    const out: Piece[] = [];
    for (const token of line) for (const ex of token.explanation ?? []) {
      const kind = kindOf(ex.scopes.at(-1)?.scopeName ?? ""), last = out[out.length - 1];
      if (last && last[1] === kind) last[0] += ex.content; else out.push([ex.content, kind]);
    }
    return out;
  });

const examples = picks.map((p) => {
  const source = read(`${p.id}.cy`);
  return { ...p, file: `${p.id}.cy`, lines: source.split("\n").length, code: highlight(source), out: read(`${p.id}.out`) };
});
mkdirSync("app/generated", { recursive: true });
writeFileSync("app/generated/examples.json", JSON.stringify(examples));
console.log(`pre-build: examples ${examples.map((e) => `${e.file} (${e.lines} lines)`).join(", ")}`);
