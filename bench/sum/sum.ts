declare const process: any;
const n = Number.parseInt(process.env.N ?? "", 10);
if (!(n >= 0 && n <= 1000000000)) process.exit(2);
let acc = 0;
for (let i = 1; i <= n; i++) { const r = i % 7; acc += (r * r) % 7; }
process.exit(acc % 1000000007 % 256);
