const n = Number.parseInt(process.env.N ?? "", 10);
if (!(n >= 0 && n <= 200000000)) process.exit(2);
const comp = new Uint8Array(n + 1);
let count = 0;
for (let p = 2; p <= n; p++) if (!comp[p]) { count++; for (let m = p * p; m <= n; m += p) comp[m] = 1; }
process.exit(count % 256);
