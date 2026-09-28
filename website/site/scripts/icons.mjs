// App icons from the brand files: the square avatar for home-screen icons (iOS masks
// its own corners), the rounded tile as the browser favicon.
import { Resvg } from "@resvg/resvg-js";
import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

const brand = path.resolve(process.cwd(), "../../brand");
mkdirSync("public/icons", { recursive: true });
const avatar = readFileSync(path.join(brand, "avatar.svg"), "utf8");
for (const [name, size] of [["icon-192.png", 192], ["icon-512.png", 512], ["apple-touch-icon.png", 180]]) {
  const png = new Resvg(avatar, { fitTo: { mode: "width", value: size } }).render().asPng();
  writeFileSync(path.join("public/icons", name), png);
}
copyFileSync(path.join(brand, "logo.svg"), "public/favicon.svg");

// The tab logo as PNG and ICO too: Safari doesn't reliably use an SVG tab icon, and
// browsers ask for /favicon.ico whatever the page links.
const logo = readFileSync(path.join(brand, "logo.svg"), "utf8");
const png = (size) => new Resvg(logo, { fitTo: { mode: "width", value: size } }).render().asPng();
writeFileSync("public/icons/favicon-32.png", png(32));
writeFileSync("public/icons/favicon-96.png", png(96));
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
writeFileSync("public/favicon.ico", Buffer.concat([head, ...images]));
console.log("icons: 192, 512, apple-touch 180, favicon.svg, favicon.ico (16/32/48), favicon 32/96 png");
