// Compile times for the home page's "wait" section (owner, 2026-10-19: "compilation time
// benchmark"). Measured by design claude in bench/ (asked 2026-10-19): the four bench programs built
// from source to a final executable, best of 3, no build cache. Null until those numbers exist: the
// chart shows a placeholder rather than anything made up.
export type CompileRow = { name: string; ours?: boolean; seconds: { sum: number; fib: number; lists: number; sieve: number } };
export type CompileRun = { date: string; machine: string; commands: string; rows: CompileRow[] };

export const COMPILE: CompileRun | null = null;
