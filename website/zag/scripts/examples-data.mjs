// The Examples page's data (owner, 2026-10-15: "Do Examples"): every program in the repo's
// examples/, read when the site is built, so the page always shows the folder as it is.
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { extname, join, relative, resolve } from "node:path";

const DIR = resolve("../../examples");

// What a program is called: its opening comment's name when it gives one ("# Collections (owner,
// …): …"), else its file name. Then the comment's first sentence, without the design notes'
// sources in brackets (dates, decisions, who found it), which mean something only in the repo.
const UPPER = { c: "C", zag: "Zag", fizzbuzz: "FizzBuzz", wstn: "WSTN", simd: "SIMD", asm: "assembly", utf8: "UTF-8", ascii: "ASCII", os: "OS", ffi: "FFI", bin: "bin", deci: "deci", io: "I/O" };
const fromName = (id) => {
  const words = id.split("_").map((w) => UPPER[w] ?? w);
  const s = words.join(" ");
  return s[0].toUpperCase() + s.slice(1);
};
const NOTE = /\s*\((?:owner|decided|design\/|found by|check claude|check run|compiler claude|ffi\.md|types\.md|operators\.md|[a-z-]+\.md|20\d\d-)[^()]*(?:\([^()]*\)[^()]*)*\)/g;
function describe(id, source) {
  const lines = [];
  for (const line of source.split("\n")) {
    if (!line.startsWith("#")) break;
    lines.push(line.replace(/^#\s?/, ""));
  }
  const text = lines.join(" ").replace(/\s+/g, " ").trim();
  if (!text) return { title: fromName(id), blurb: "" };
  let title = fromName(id), rest = text;
  // A short name before the colon is a title; a longer one is already a sentence, so the file's
  // name stands and the whole comment is the description.
  const head = text.match(/^([A-Z][^:.`(]{2,40}?)(?:\s*\([^)]*\))?:\s+(.*)$/);
  if (head && head[1].trim().length <= 30 && !/^Finding\b|^The root of\b/.test(head[1])) { title = head[1].trim(); rest = head[2]; }
  rest = rest.replace(NOTE, "").replace(/^Finding \d{4}-\d\d-\d\d:?\s*/, "");
  const first = rest.match(/^.*?\.(?=\s|$)/)?.[0] ?? rest;
  let blurb = first.trim();
  if (blurb.length > 180) blurb = blurb.slice(0, 177).replace(/\s+\S*$/, "") + "…";
  // Capitalised only when it starts with a plain word; code keeps its own case.
  if (/^[a-z]+[ ,]/.test(blurb)) blurb = blurb[0].toUpperCase() + blurb.slice(1);
  return { title, blurb };
}

const read = (f) => readFileSync(f, "utf8").replace(/\n$/, "");
const extra = (base) => ({
  out: existsSync(`${base}.out`) ? read(`${base}.out`) : null,
  exit: existsSync(`${base}.exit`) ? read(`${base}.exit`).replace(/^exit:/, "") : null,
  args: existsSync(`${base}.args`) ? read(`${base}.args`).split("\n") : null,
  tests: existsSync(`${base}.test-out`) ? read(`${base}.test-out`) : null,
  noOracle: existsSync(`${base}.oracle`),
});

// Every file of a program folder, the root file first and the manifest second.
function folderFiles(dir) {
  const files = [];
  (function walk(d) {
    for (const name of readdirSync(d).sort()) {
      const full = join(d, name);
      if (statSync(full).isDirectory()) walk(full);
      else files.push(relative(dir, full));
    }
  })(dir);
  const rank = (f) => (f === "strike.zg" ? 0 : f === "Ricochet.zv" ? 1 : f === "build.zg" ? 2 : 3);
  return files.sort((a, b) => rank(a) - rank(b) || a.localeCompare(b));
}

export function readExamples() {
  const items = [];
  for (const name of readdirSync(DIR).sort()) {
    const full = join(DIR, name);
    if (statSync(full).isDirectory()) {
      if (!existsSync(`${full}.out`)) continue; // a folder other programs use, not one that runs
      const files = folderFiles(full).map((f) => ({ name: f, source: extname(f) === ".bin" ? null : read(join(full, f)), bytes: statSync(join(full, f)).size }));
      const root = files.find((f) => f.name === "strike.zg");
      items.push({ id: name, folder: true, ...describe(name, root?.source ?? ""), files, ...extra(full) });
    } else if (name.endsWith(".zg")) {
      const id = name.slice(0, -3), source = read(full);
      items.push({ id, folder: false, ...describe(id, source), files: [{ name, source, bytes: statSync(full).size }], ...extra(join(DIR, id)) });
    }
  }
  return items;
}
