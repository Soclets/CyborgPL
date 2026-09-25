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
console.log("icons: 192, 512, apple-touch 180, favicon.svg");
