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

// Legal and Credits (owner, 2026-10-19: More holds "Legal" (licences, trademarks and logo, privacy)
// and "Credits"): their official wording is read from the README, which the owner approved and
// design claude keeps, so the pages always say what it says. Blocks are kept as Markdown; the page
// renders the little it uses (app/components/md.tsx).
type Block = { kind: "p"; md: string } | { kind: "h"; md: string } | { kind: "ul"; items: string[] };
const README = readFileSync("../../README.md", "utf8").split("\n");
const paragraphs = (lines: string[]) => lines.join("\n").split(/\n\s*\n/).map((p) => p.replace(/\s*\n\s*/g, " ").trim()).filter(Boolean);
const warning = paragraphs(README.filter((l) => l.startsWith(">")).map((l) => l.replace(/^>\s?/, "")).filter((l) => !l.startsWith("[!WARNING]")));
const licences = README.filter((l) => /^- \[`LICENSE/.test(l)).map((l) => l.replace(/^- /, ""));
const creditLines = README.filter((l) => /^ {2}- (this repo's|the website's|the logos)/.test(l)).map((l) => l.replace(/^ {2}- /, ""));
const asked = README.find((l) => /^ {2}As the licence asks/.test(l))?.trim() ?? "";
const trademark = README.find((l) => l.includes("names of this project")) ?? "";
const copyright = README.find((l) => /^Copyright /.test(l)) ?? "";
// The Credits <details>: paragraphs, bold headings and lists, without the README's crate and
// website-package lists (this page builds its own from the lockfiles).
const from = README.findIndex((l) => l.includes("<summary><b>Credits</b></summary>"));
const to = README.findIndex((l, i) => i > from && /^Copyright /.test(l));
const credits: Block[] = [];
let skip = false;
for (const raw of README.slice(from + 1, to)) {
  const line = raw.trim();
  if (!line || /^<\/?(details|summary)/.test(line)) continue;
  // A heading is a bold line on its own, maybe with a note in brackets: "**Rust crates** (…)".
  const heading = line.match(/^\*\*([^*]+)\*\*(\s*\(.*\))?$/);
  if (heading) {
    skip = /^(Rust crates|Website packages)/.test(heading[1]!);
    if (!skip) credits.push({ kind: "h", md: heading[1]! });
    continue;
  }
  if (skip) continue;
  if (line.startsWith("- ")) {
    const last = credits[credits.length - 1];
    if (last?.kind === "ul") last.items.push(line.slice(2)); else credits.push({ kind: "ul", items: [line.slice(2)] });
  } else credits.push({ kind: "p", md: line });
}
const crates = readFileSync("../../kayo/Cargo.lock", "utf8").split("[[package]]").slice(1)
  .filter((b) => /^source = /m.test(b)).map((b) => `${b.match(/^name = "(.*)"/m)![1]} ${b.match(/^version = "(.*)"/m)![1]}`);
const lock = JSON.parse(readFileSync("package-lock.json", "utf8")).packages as Record<string, { version?: string }>;
const packages = [...new Set(Object.entries(lock).filter(([k]) => k).map(([k, v]) => `${k.slice(k.lastIndexOf("node_modules/") + 13)} ${v.version}`))].sort();
const direct = Object.keys({ ...JSON.parse(readFileSync("package.json", "utf8")).dependencies, ...JSON.parse(readFileSync("package.json", "utf8")).devDependencies }).sort();
writeFileSync("app/generated/about.json", JSON.stringify({ warning, licences, creditLines, asked, trademark, copyright, credits, crates, packages, direct }));
console.log(`pre-build: about.json, ${credits.length} credit blocks, ${crates.length} crates, ${packages.length} packages`);
