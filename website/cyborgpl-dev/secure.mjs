// Assembles what cyborgpl.dev serves: the last CyborgPL build (website/site at 75adac1d in the
// frozen Cyborg-Assembly-Manufacturing-Site/CyborgPL, the bytes already live), unchanged, plus
// the security headers in security-headers.txt.
import { cpSync, readFileSync, rmSync, writeFileSync } from "node:fs";

const FROM = process.argv[2] ?? "/Users/ts/CyborgPL v2 website/website/site/build/client";
const OUT = "build/client";
rmSync("build", { recursive: true, force: true });
cpSync(FROM, OUT, { recursive: true });

const security = readFileSync("security-headers.txt", "utf8");
const long = security.split("\n").find((l) => l.length > 2000);
if (long) throw new Error(`secure: a header line is ${long.length} characters, past Cloudflare's 2,000`);
writeFileSync(`${OUT}/_headers`, readFileSync(`${OUT}/_headers`, "utf8").trimEnd() + "\n" + security);
console.log("secure: build copied, security headers added");
