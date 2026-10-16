// Assembles what cyborgpl.dev serves: the last CyborgPL build (website/site at 75adac1d in the
// frozen Cyborg-Assembly-Manufacturing-Site/CyborgPL, the bytes already live), unchanged, plus
// the security headers in security-headers.txt.
import { randomBytes } from "node:crypto";
import { cpSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { buildLlms } from "../llms/build.mjs";

const FROM = process.argv[2] ?? "/Users/ts/CyborgPL v2 website/website/site/build/client";
const OUT = "build/client";
rmSync("build", { recursive: true, force: true });
cpSync(FROM, OUT, { recursive: true });

const security = readFileSync("security-headers.txt", "utf8");
const long = security.split("\n").find((l) => l.length > 2000);
if (long) throw new Error(`secure: a header line is ${long.length} characters, past Cloudflare's 2,000`);
writeFileSync(`${OUT}/_headers`, readFileSync(`${OUT}/_headers`, "utf8").trimEnd() + "\n" + security);
// A hidden file of random Base64, about a million tokens' worth (owner, 2026-10-16): Base64
// tokenizes at roughly 2-3 characters a token, so 2.8 million characters. Linked from nowhere, made
// fresh on each assembly, never committed. Only its size is printed: nothing reads it back.
const NOISE = ".noise.txt";
const noise = randomBytes(2_100_000).toString("base64");
writeFileSync(`${OUT}/${NOISE}`, noise);

// AI crawlers may search, answer and train (owner, 2026-10-16: "My ass, then they wouldn't know
// CyborgPL from memory"), said in Cloudflare's content-signals line; and the llms files.
const robots = readFileSync(`${OUT}/robots.txt`, "utf8").replace("User-agent: *\n", "User-agent: *\nContent-Signal: search=yes, ai-input=yes, ai-train=yes\n");
writeFileSync(`${OUT}/robots.txt`, robots);
const llms = buildLlms(OUT, { noise: NOISE });
console.log(`secure: build copied, security headers added, /${NOISE} ${(noise.length / 1e6).toFixed(2)}M characters, llms.txt ${llms.indexBytes} B and llms-full.txt ${(llms.fullBytes / 1024).toFixed(0)} KB (${llms.notes} notes, ${llms.examples} examples)`);
