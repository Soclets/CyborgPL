// After the build: the pieces Cloudflare and search engines read from the site's root.
import { copyFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const out = "build/client";
const SITE = "https://zag-lang.org";

// Cloudflare shows 404.html for an address it has nothing for, so the built /404 page goes there.
copyFileSync(`${out}/404/index.html`, `${out}/404.html`);

// The picture a shared link shows (app/seo.js), kept in brand/zag/ with the logo.
copyFileSync("../../brand/zag/social-preview.png", `${out}/social-preview.png`);

// Every prerendered page goes in the sitemap, except the 404 page.
const pages = [];
(function walk(dir, path) {
  for (const name of readdirSync(dir)) {
    const full = join(dir, name);
    if (statSync(full).isDirectory()) { if (name !== "assets" && name !== "404") walk(full, `${path}${name}/`); }
    else if (name === "index.html") pages.push(path === "/" ? "/" : path.replace(/\/$/, ""));
  }
})(out, "/");
pages.sort();
writeFileSync(`${out}/sitemap.xml`, `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${pages.map((p) => `  <url><loc>${SITE}${p}</loc></url>`).join("\n")}\n</urlset>\n`);
writeFileSync(`${out}/robots.txt`, `User-agent: *\nAllow: /\n\nSitemap: ${SITE}/sitemap.xml\n`);

console.log(`post-build: 404.html, social-preview.png, sitemap.xml (${pages.join(", ")}), robots.txt`);
