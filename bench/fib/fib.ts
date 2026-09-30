declare const process: any;
const n = Number.parseInt(process.env.N ?? "", 10);
if (!(n >= 0 && n <= 92)) process.exit(2);
function fib(k: number): number { return k < 2 ? k : fib(k - 1) + fib(k - 2); }
process.exit(fib(n) % 256);
