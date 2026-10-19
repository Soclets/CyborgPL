import type { BenchRun } from "./compile-times";

// Run times for the home page's "above average" section (owner, 2026-10-19: "Ask design claude to
// run a fresh one, with cheat and no_cheat", then "every" for which CyborgPL rows to show). From
// bench/RESULTS.md, "2026-10-19, later: run time with cheat and no_cheat" (e91e1993; kayo
// `CyDT f84c8a7a KAYO\n`): final builds, best of 3, every answer agreed with C. A row's total is its
// four times added up.
const row = (name: string, s: [number, number, number, number], extra: Partial<{ detail: string; ours: boolean; threads: boolean }> = {}) => ({
  name, ...extra, total: Math.round((s[0] + s[1] + s[2] + s[3]) * 1000) / 1000,
  seconds: { sum: s[0], fib: s[1], lists: s[2], sieve: s[3] },
});

export const RUN: BenchRun | null = {
  date: "19 Oct 2026",
  machine: "an Apple Silicon Mac",
  commands: "python3 bench/run.py cyborgpl cyborgpl1 cyborgpl_nc cyborgpl1_nc c fortran go zig rust",
  caveat: "no_cheat makes every call the program writes really run; plain CyborgPL may do less work (fib becomes a table Precog fills while building). Striped rows use 3 cores; every other language uses one. Fortran's fib also gets its answers without the work.",
  rows: [
    row("CyborgPL", [0.033, 0.002, 0.039, 0.166], { detail: "3 threads", ours: true, threads: true }),
    row("CyborgPL", [0.091, 0.002, 0.041, 0.302], { detail: "1 thread", ours: true }),
    row("CyborgPL", [0.050, 0.012, 0.040, 0.204], { detail: "no_cheat · 3 threads", ours: true, threads: true }),
    row("CyborgPL", [0.142, 0.012, 0.079, 0.341], { detail: "no_cheat · 1 thread", ours: true }),
    row("C", [0.094, 0.017, 0.042, 0.413]),
    row("Rust", [0.174, 0.017, 0.053, 0.397]),
    row("Go", [0.145, 0.024, 0.111, 0.386]),
    row("Zig", [0.148, 0.016, 0.071, 0.367]),
    row("Fortran", [0.135, 0.003, 0.091, 0.541]),
  ],
};
