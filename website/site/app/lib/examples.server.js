// Build-time only (the .server suffix keeps it out of the browser bundle): reads the
// real programs from examples/ and highlights them with the repo's own grammar.
import { existsSync, readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { createHighlighter } from "shiki";
import { solarizedDark, solarizedLight } from "./theme.js";

const REPO = path.resolve(process.cwd(), "../..");
const EXAMPLES = path.join(REPO, "examples");

let highlighter;
function hl() {
  highlighter ??= createHighlighter({
    themes: [solarizedLight, solarizedDark],
    langs: [JSON.parse(readFileSync(path.join(REPO, "website/grammar/cyborgpl.tmLanguage.json"), "utf8"))],
  });
  return highlighter;
}

// Both themes in one pass: each span carries the light colour, and --shiki-dark for the
// dark theme, which the desktop site switches to (app.css).
export async function highlight(code) {
  return (await hl()).codeToHtml(code, { lang: "cyborgpl", themes: { light: solarizedLight.name, dark: solarizedDark.name }, defaultColor: "light" });
}

// A few names read better by hand; the rest come from the file name.
// Code in a title is marked `like this` (shown as code; plain in a browser tab).
const TITLES = {
  hello: "Hello, World", fizzbuzz: "FizzBuzz", enums: "Enums & `match`", concur: "Concurrency",
  ffi: "Talking to C", simd: "SIMD", pi: "Digits of pi", whichever: "`whichever`",
  and_runs_both: "`and` runs both sides", and_then_or_else: "`and.then` / `or.else`", no_checks: "`NO_FUCKING_CHECKS`",
  strict: "`#strict`", print_vs_print_useful: "`print` vs `print.useful`", print_format: "`print.format`", c_enum: "C enums", list_array: "`list` and `array`",
  match_value: "`match` gives a value", strict_match: "`match` in `#strict`", operator_values: "Operators as values",
  lifetimes: "Lifetimes: `@x`", for_position: "Position and item",
  for_no_cheat: "`for !c`: no cheating", for_para: "`for para` and `for seq`", print_weird: "`print.weird`",
  print_auto_newline: "`print.useful.auto_newline`", testing: "Tests: `test fn` and `must`", list_fill: "`list.fill`",
  list_fill_strict: "`list.fill` in `#strict`", ptr_cast: "`ptr.cast`", deci: "`deci`: exact decimals", deci_more: "`deci`, continued",
  deci_context: "`deci` beside other numbers", to_ascii: "To ASCII", nested_match: "Nested `match`",
  strict_nested_match: "Nested `match` in `#strict`", return_in_match: "`return` inside a `match`",
  match_data_type: "A `match` arm's data", for_lend_mut: "`for` over `lend.mut`", for_lend_mut_read: "Reading during a `lend.mut` walk",
};
export const titleOf = (id) => TITLES[id] ?? id.replace(/_/g, " ").replace(/^./, (c) => c.toUpperCase()).replace(/^Simd\b/, "SIMD");

const read = (id, ext) => readFileSync(path.join(EXAMPLES, `${id}.${ext}`), "utf8").replace(/\n$/, "");
export const exampleIds = () => readdirSync(EXAMPLES).filter((f) => f.endsWith(".cpls")).map((f) => f.slice(0, -5)).sort();

// The opening sentence of the first comment, without its "(design/…, date)" bookkeeping. A
// sentence may run over several comment lines; an indented line (a list) or a blank one ends it.
function summaryOf(src) {
  const lines = src.split("\n");
  const start = lines.findIndex((l) => l.trim().startsWith("#") && !l.trim().startsWith("#strict"));
  if (start < 0) return "";
  let text = "";
  for (let i = start; i < lines.length; i++) {
    const m = lines[i].match(/^\s*#(?![0-9]*l|wl|strict)( ?)(.*)$/);
    if (!m || (i > start && (/^\s/.test(m[2]) || !m[2].trim()))) break;
    text += (text ? " " : "") + m[2].trim();
    if (/\.$/.test(m[2].trim())) break;
  }
  text = text.replace(/\s*\((?:[^()]*?(?:design\/|\d{4}-\d{2}-\d{2}|owner)[^()]*)\)/g, "").trim();
  const end = text.search(/\.(\s|$)/); // only a full stop: `!` and `?` are also operator characters
  return end >= 0 ? text.slice(0, end + 1) : text;
}

export function listExamples() {
  return exampleIds().map((id) => {
    const src = read(id, "cpls");
    const code = src.split("\n").filter((l) => l.trim() && !l.trim().startsWith("#"));
    return { id, title: titleOf(id), summary: summaryOf(src), lines: src.split("\n").length, preview: code.slice(0, 6).join("\n") };
  });
}

export async function getExample(id) {
  if (!exampleIds().includes(id)) throw new Response("Not found", { status: 404 });
  const src = read(id, "cpls");
  const exit = existsSync(path.join(EXAMPLES, `${id}.exit`)) ? read(id, "exit").trim() : null;
  return {
    id, title: titleOf(id), summary: summaryOf(src), file: `examples/${id}.cpls`,
    lines: src.split("\n").length, html: await highlight(src), out: read(id, "out"), exit,
  };
}
