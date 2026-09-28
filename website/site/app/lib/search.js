// The site's search. Pure functions, used by Home (everything) and Examples (programs).
import { ANSWERS, ANSWER_BY_KEY } from "./answers.js";

export const SOURCE_QUERIES = new Set(["gh", "repo", "source", "source code", "github"]);

// Other words people use for the same idea — only where the idea really is CyborgPL's.
const SYNONYMS = {
  gc: ["ownership"], garbage: ["ownership"], collector: ["ownership"], memory: ["ownership"], borrow: ["lend", "ownership"], move: ["give", "ownership"],
  exception: ["failures", "fail", "otherwise"], exceptions: ["failures", "fail", "otherwise"], error: ["failures", "fail"], errors: ["failures", "fail"],
  try: ["otherwise", "failures"], catch: ["otherwise", "failures"], throw: ["fail"], result: ["failures"],
  async: ["concurrency", "concur"], await: ["concur"], thread: ["concur", "concurrency"], threads: ["concur", "concurrency"], parallel: ["concur", "simd", "concurrency"], spawn: ["concur"],
  license: ["licence"], install: ["download"], release: ["download"],
  tutorial: ["learn", "syntax"], docs: ["learn"], documentation: ["learn"], guide: ["learn"], basics: ["learn", "syntax"],
  inference: [":="], infer: [":="], auto: [":="], var: [":=", "let"],
  overflow: ["precog", "overflow"], bounds: ["precog", "bounded"], safety: ["precog"], safe: ["precog"], prove: ["precog"], proof: ["precog"],
  compiler: ["llvm"], native: ["llvm"], backend: ["llvm"],
  string: ["str.utf8", "strings", "text"], strings: ["str.utf8", "text"],
  print: ["std::print.stdout", "prints"], println: ["std::print.stdout", "prints"], output: ["std::print.stdout", "prints"], stdout: ["std::print.stdout"],
  loop: ["while", "for", "loops"], iterate: ["for", "range"],
  function: ["fn", "functions"], method: ["fn", "functions"], func: ["fn"],
  enum: ["enums", "match"], variant: ["enums"], union: ["enums"], switch: ["match"],
  record: ["structs", "struct"], class: ["structs", "struct"], object: ["structs"],
  template: ["generics"], templates: ["generics"], generic: ["generics"],
  array: ["list", "lists", "array"], vector: ["list", "lists"], vec: ["list"],
  dict: ["map", "maps"], dictionary: ["map", "maps"], hashmap: ["map", "maps"],
  comment: ["#", "comments"],
  assign: ["change"], reassign: ["change"], mutate: ["change", "mut"], mutable: ["mut"], shadow: ["$:=", "sealed"], strict: ["#strict", "strict"],
};

export const norm = (s) => s.trim().toLowerCase().replace(/\s+/g, " ");
const words = (t) => t.toLowerCase().split(/[^a-z0-9:=.#+\\]+/).filter(Boolean);
const isWord = (t) => /^[a-z_][a-z0-9_.]*$/.test(t);
const escape = (s) => s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

// Edit distance, giving up past `max`.
function near(a, b, max) {
  if (Math.abs(a.length - b.length) > max) return false;
  let prev = Array.from({ length: b.length + 1 }, (_, j) => j);
  for (let i = 1; i <= a.length; i++) {
    const cur = [i];
    let rowMin = i;
    for (let j = 1; j <= b.length; j++) {
      cur[j] = Math.min(prev[j] + 1, cur[j - 1] + 1, prev[j - 1] + (a[i - 1] === b[j - 1] ? 0 : 1));
      rowMin = Math.min(rowMin, cur[j]);
    }
    if (rowMin > max) return false;
    prev = cur;
  }
  return prev[b.length] <= max;
}
const fuzz = (t) => (t.length >= 8 ? 2 : t.length >= 4 ? 1 : 0);

// A line of code matches a word where a word starts (not inside a quoted 'name'),
// and a symbol (:=, ++, \n…) anywhere. Comments don't count.
const wordRe = new Map();
const commentless = new WeakMap();
function codeHit(term, lines) {
  if (!lines) return null;
  let re = null;
  if (isWord(term)) {
    re = wordRe.get(term);
    if (!re) wordRe.set(term, (re = new RegExp(`(^|[^A-Za-z0-9_'])${escape(term)}${term.length <= 2 ? "(?![A-Za-z0-9_])" : ""}`, "i")));
  }
  let code = commentless.get(lines);
  if (!code) commentless.set(lines, (code = lines.map((l) => l.replace(/\s+#\s.*$/, ""))));
  for (let i = 0; i < code.length; i++) if (re ? re.test(code[i]) : code[i].includes(term)) return lines[i].trim();
  return null;
}

function scoreTerm(term, entry, lines) {
  const title = entry._title ??= words(entry.title + " " + (entry.id ?? "").replace(/_/g, " "));
  const text = entry._text ??= words(entry.text ?? "");
  const short = term.length <= 2;
  const has = (ws, exactOnly) => ws.some((w) => (exactOnly ? w === term : w.startsWith(term)));
  if (title.includes(term)) return { s: 6 };
  if (!short && has(title)) return { s: 5 };
  const line = codeHit(term, lines);
  if (line) return { s: 3, line };
  if (has(text, short)) return { s: 2 };
  const f = fuzz(term);
  if (f && title.some((w) => near(term, w, f))) return { s: 2, fuzzy: true };
  if (f && text.some((w) => near(term, w, f))) return { s: 1, fuzzy: true };
  return { s: 0 };
}

// entries: [{ to, title, text, kind, icon, id? }]; code: { [exampleId]: string[] } or null.
export function runSearch(entries, query, code) {
  const q = norm(query);
  if (!q) return { q, answer: null, source: false, results: [] };
  const a = ANSWER_BY_KEY.get(q);
  const terms = q.split(" ");
  const results = [];
  entries.forEach((entry, order) => {
    let score = 0, line = null;
    for (const term of terms) {
      let best = { s: 0 };
      for (const [k, alt] of [term, ...(SYNONYMS[term] ?? [])].entries()) {
        const r = scoreTerm(alt, entry, entry.id ? code?.[entry.id] : null);
        const s = r.s - (k ? 1 : 0);
        if (s > best.s) best = { ...r, s };
      }
      if (best.s <= 0) return;
      score += best.s;
      line ??= best.line ?? null;
    }
    results.push({ entry, score, line, order });
  });
  results.sort((x, y) => y.score - x.score || x.order - y.order);
  return { q, answer: a === undefined ? null : ANSWERS[a], answerIndex: a ?? null, source: SOURCE_QUERIES.has(q), results: results.slice(0, 40) };
}

// Split text into [plain, matched, plain, …] for highlighting the query's words.
export function marks(text, q) {
  const terms = norm(q).split(" ").filter((t) => t.length >= 1).sort((x, y) => y.length - x.length);
  if (!terms.length || !text) return [text];
  const re = new RegExp(`(${terms.map(escape).join("|")})`, "ig");
  return text.split(re);
}

// A long line starts a little before the match, so the match is on screen.
export function around(line, q) {
  const terms = norm(q).split(" ").filter(Boolean);
  const at = Math.min(...terms.map((t) => line.toLowerCase().indexOf(t)).filter((i) => i >= 0));
  return Number.isFinite(at) && at > 18 ? "…" + line.slice(at - 10) : line;
}
