// Cloudflare shows 404.html for an address it has nothing for, so the built /404 page goes there.
import { copyFileSync } from "node:fs";
copyFileSync("build/client/404/index.html", "build/client/404.html");
console.log("post-build: 404.html");
