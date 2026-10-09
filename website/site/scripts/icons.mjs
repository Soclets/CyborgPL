// App icons from the brand files (brand/nerve/): the Nerve icon, a medical cross carrying a
// nerve impulse. The square avatar for home-screen icons (iOS masks its own corners), the
// rounded tile as the browser favicon and the comparison's logo (always on its tile).
import { Resvg } from "@resvg/resvg-js";
import { createHash } from "node:crypto";
import { copyFileSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";

const brand = path.resolve(process.cwd(), "../../brand");
mkdirSync("public/icons", { recursive: true });
// start clean, so icons from an older drawing (or older names) aren't deployed beside the new ones
for (const f of readdirSync("public/icons")) rmSync(path.join("public/icons", f));
mkdirSync("public/language-logos", { recursive: true });
copyFileSync(path.join(brand, "nerve/icon.svg"), "public/language-logos/nerve.svg");
const avatar = readFileSync(path.join(brand, "nerve/avatar.svg"), "utf8");
// Cache busting: the icons' file names carry a hash of their drawings, so a new icon is a new
// address that no browser (Safari's icon cache included) has seen. root.jsx and the manifest use it.
const tab = readFileSync(path.join(brand, "nerve/icon.svg"), "utf8");
const H = createHash("sha256").update(tab).update(avatar).digest("hex").slice(0, 8);
writeFileSync("app/lib/icon-hash.js", `// Written by scripts/icons.mjs: a hash of brand/nerve/icon.svg and avatar.svg.\nexport const ICON = "${H}";\n`);
for (const [name, size] of [[`app-${H}-192.png`, 192], [`app-${H}-512.png`, 512], [`touch-${H}.png`, 180]]) {
  const png = new Resvg(avatar, { fitTo: { mode: "width", value: size } }).render().asPng();
  writeFileSync(path.join("public/icons", name), png);
}
copyFileSync(path.join(brand, "nerve/icon.svg"), "public/favicon.svg");
copyFileSync(path.join(brand, "nerve/icon.svg"), `public/icons/tab-${H}.svg`);
// RRx2's logo, for its row on More (the light version: RR and heart red, 2 and arrow black).
copyFileSync(path.join(brand, "rrx2/logo.svg"), "public/icons/rrx2.svg");

// The tab logo as PNG and ICO too: Safari doesn't reliably use an SVG tab icon, and
// browsers ask for /favicon.ico whatever the page links.
const logo = readFileSync(path.join(brand, "nerve/icon.svg"), "utf8");
const png = (size) => new Resvg(logo, { fitTo: { mode: "width", value: size } }).render().asPng();
writeFileSync(`public/icons/tab-${H}-32.png`, png(32));
writeFileSync(`public/icons/tab-${H}-96.png`, png(96));
const sizes = [16, 32, 48];
const images = sizes.map(png);
const head = Buffer.alloc(6 + 16 * sizes.length);
head.writeUInt16LE(0, 0); head.writeUInt16LE(1, 2); head.writeUInt16LE(sizes.length, 4);
let offset = head.length;
sizes.forEach((size, i) => {
  const e = 6 + 16 * i;
  head.writeUInt8(size, e); head.writeUInt8(size, e + 1);            // width, height
  head.writeUInt16LE(1, e + 4); head.writeUInt16LE(32, e + 6);         // planes, bits per pixel
  head.writeUInt32LE(images[i].length, e + 8); head.writeUInt32LE(offset, e + 12);
  offset += images[i].length;
});
const ico = Buffer.concat([head, ...images]);
writeFileSync("public/favicon.ico", ico);
writeFileSync(`public/icons/tab-${H}.ico`, ico);
// the manifest names the home-screen icons by the same hash
const manifest = JSON.parse(readFileSync("public/manifest.webmanifest", "utf8"));
manifest.icons = [
  { src: `/icons/app-${H}-192.png`, sizes: "192x192", type: "image/png" },
  { src: `/icons/app-${H}-512.png`, sizes: "512x512", type: "image/png" },
  { src: `/icons/app-${H}-512.png`, sizes: "512x512", type: "image/png", purpose: "maskable" },
];
writeFileSync("public/manifest.webmanifest", JSON.stringify(manifest, null, 2) + "\n");
console.log(`icons ${H}: 192, 512, apple-touch 180, tab svg/ico/32/96, favicon.svg, favicon.ico`);
