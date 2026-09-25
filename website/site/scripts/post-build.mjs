// Cloudflare serves /404.html (with status 404) for any path that isn't a file.
import { copyFileSync } from "node:fs";
copyFileSync("build/client/404/index.html", "build/client/404.html");
console.log("post-build: 404.html");
