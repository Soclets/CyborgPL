const n = Number.parseInt(process.env.N ?? "", 10);
if (!(n >= 0 && n <= 100000000)) process.exit(2);
const xs = [];
for (let i = 1; i <= n; i++) xs.push(i % 1000);
let acc = 0;
for (let r = 0; r < 10; r++) for (let j = 0; j < xs.length; j++) acc += xs[j];
process.exit(acc % 256);
