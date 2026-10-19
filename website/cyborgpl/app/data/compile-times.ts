// Compile times for the home page's "wait" section (owner, 2026-10-19: "compilation time
// benchmark"). Measured by design claude in bench/ (asked 2026-10-19): the four bench programs built
// from source to a final executable, best of 3, no build cache. Null until those numbers exist: the
// chart shows a placeholder rather than anything made up.
// `total` is RESULTS.md's own total: the per-program times are rounded, so summing them can be off by 0.01.
export type CompileRow = { name: string; ours?: boolean; total: number; seconds: { sum: number; fib: number; lists: number; sieve: number } };
export type CompileRun = { date: string; machine: string; commands: string; caveat: string; rows: CompileRow[] };

// bench/RESULTS.md, "2026-10-19: compile time, source to final executable" (bench/compile_time.py,
// kayo `CyDT f84c8a7a KAYO\n`): best of 3 cold builds per program, wall-clock, one build at a time.
export const COMPILE: CompileRun | null = {
  date: "19 Oct 2026",
  machine: "an Apple Silicon Mac",
  commands: "kayo build --final · zig -O ReleaseFast (empty cache) · go build (empty GOCACHE) · rustc -C opt-level=3 · gfortran -O3 · clang -O3",
  caveat: "Zig and Go come close only because an empty cache makes them rebuild their standard libraries every time; warm, they'd be far faster. KAYO has no cache to warm: its time is Precog proving every check, so it grows with what there is to prove.",
  rows: [
    { name: "CyborgPL", total: 18.18, ours: true, seconds: { sum: 2.11, fib: 0.10, lists: 5.48, sieve: 10.49 } },
    { name: "Zig", total: 17.85, seconds: { sum: 4.38, fib: 4.39, lists: 4.55, sieve: 4.52 } },
    { name: "Go", total: 5.02, seconds: { sum: 1.28, fib: 1.26, lists: 1.24, sieve: 1.25 } },
    { name: "Rust", total: 0.33, seconds: { sum: 0.08, fib: 0.08, lists: 0.10, sieve: 0.08 } },
    { name: "Fortran", total: 0.29, seconds: { sum: 0.07, fib: 0.08, lists: 0.08, sieve: 0.07 } },
    { name: "C", total: 0.16, seconds: { sum: 0.04, fib: 0.04, lists: 0.04, sieve: 0.04 } },
  ],
};
