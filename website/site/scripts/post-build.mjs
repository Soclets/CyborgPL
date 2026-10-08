import { copyFileSync, readdirSync, writeFileSync } from "node:fs";
import path from "node:path";

// Cloudflare serves /404.html (with status 404) for any path that isn't a file.
copyFileSync("build/client/404/index.html", "build/client/404.html");

// The sitemap: every pre-rendered page, by its address (no trailing slash, as served).
const SITE = "https://nerve-lang.com";
const pages = [];
(function walk(dir) {
  for (const e of readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) { if (e.name !== "assets" && e.name !== "icons") walk(p); }
    else if (e.name === "index.html") pages.push("/" + path.relative("build/client", dir).split(path.sep).join("/"));
  }
})("build/client");
const urls = pages.filter((p) => p !== "/404").map((p) => (p === "/" ? "/" : p.replace(/\/$/, ""))).sort();
writeFileSync("build/client/sitemap.xml", `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
${urls.map((u) => `  <url><loc>${SITE}${u}</loc></url>`).join("\n")}
</urlset>
`);
console.log(`post-build: 404.html, sitemap.xml (${urls.length} pages)`);
