// The repo's commit history for the Commits page (owner, 2026-10-12: "a mirror of commits, minus
// the code part"): each commit's message, author, co-authors and date, never its changes. A commit
// that touches secrets/, or whose subject is house.secrets…, is left out: those are the owner's
// private notes.
import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";

const git = (...args) => execFileSync("git", ["-C", "../..", ...args], { maxBuffer: 64 << 20 }).toString();
const secret = new Set(git("log", "--format=%H", "--", "secrets").split("\n").filter(Boolean));
const commits = [];
for (const rec of git("log", "--no-merges", "--format=%H%x1f%aI%x1f%an%x1f%B%x1e").split("\x1e")) {
  const [hash, date, author, msg] = rec.replace(/^\n+/, "").split("\x1f");
  if (!hash || secret.has(hash)) continue;
  const lines = msg.trim().split("\n");
  const subject = lines[0].trim();
  if (/^house\.secrets\b/.test(subject)) continue;
  const by = [], body = [];
  for (const l of lines.slice(1)) {
    const m = l.match(/^Co-Authored-By:\s*(.+?)\s*<[^>]*>\s*$/i);
    if (m) by.push(m[1]); else body.push(l);
  }
  commits.push({ h: hash.slice(0, 8), d: date, a: author, by, s: subject, b: body.join("\n").trim() });
}
writeFileSync("public/commits.json", JSON.stringify(commits));
console.log(`commits: ${commits.length} (${secret.size} touching secrets/ left out)`);
