// Build time: the good-practice page comes from design/style.md itself (owner, 2026-10-02), so
// the site says exactly what the design says. The doc's references to other design files
// ("(`input.md`)") are dropped: visitors can't open them.
import { readFileSync } from "node:fs";
import path from "node:path";
import { highlight } from "./examples.server.js";

const tidy = (t) => t.replace(/\s*\(`[a-z-]+\.md`[^)]*\)/g, "").replace(/\s+/g, " ").trim();

export async function readPractice() {
  const src = readFileSync(path.resolve(process.cwd(), "../../design/style.md"), "utf8").split("\n");
  // Gather each numbered practice whole (its bold title may wrap onto the next line), and its
  // code block, under the ## group it sits in.
  const groups = [];
  let item = null, code = null;
  for (const raw of src) {
    const line = raw.replace(/^ {3,4}/, "");
    if (code) { if (line.trim().startsWith("```")) { item.codeSrc = code.join("\n"); code = null; } else code.push(line); continue; }
    if (item && line.trim().startsWith("```")) { code = []; continue; }
    const h = line.match(/^## (.+)/);
    if (h) { groups.push({ title: h[1].trim(), items: [] }); item = null; continue; }
    if (/^\d+\. /.test(line) && groups.length) { item = { lines: [line] }; groups.at(-1).items.push(item); continue; }
    if (item && line.trim()) item.lines.push(line.trim());
  }
  for (const g of groups) {
    g.items = await Promise.all(g.items.map(async (it) => {
      const m = it.lines.join(" ").match(/^(\d+)\.\s+\*\*(.+?)\*\*\s*(.*)$/s);
      return { n: +m[1], title: m[2].replace(/\.$/, "").replace(/\s+/g, " "), text: tidy(m[3]), code: it.codeSrc ? await highlight(it.codeSrc) : null };
    }));
  }
  return groups;
}
