# Reproducible builds

Decided 2026-09-14. Same source, same compiler, same *named* target →
byte-identical binary, on any machine, any day.

## Why CyborgPL is more exposed than other compilers

In C or Rust, nondeterministic optimisation yields a slower or faster binary
that does the same thing. In CyborgPL, Precog's proofs decide **behaviour**:
a bound proven on one machine and not on another means one binary has no
runtime check and the other has one — two different programs. SMT solvers
are nondeterministic by default (time limits, seeds, memory, clause order),
and "compile for the exact machine" makes the target part of the input.

## Requirements on Precog and the build

1. **No wall-clock anywhere in Precog.** Every budget is a step count —
   solver steps, unroll depth, invariant candidates tried — never seconds.
2. **Solver determinism pinned.** Fixed seeds and configuration; results may
   not depend on thread scheduling. The solver's version is part of the
   compiler's version: a solver upgrade is a compiler change.
3. **Compile-time I/O is always a boundary.** Precog never reads a file,
   clock, or environment variable at compile time. Those are bounded values
   with explicit types like any other input. (Closes the earlier open
   question.)
4. **Measurements are inputs too** (owner, 2026-09-28): `synthia tune` times
   loops once, on purpose, and writes `tune.cplt`; builds read the file
   and never time anything themselves (`gpu.md`).
5. **The target is an explicit, recorded input** — in `Synthia.cplm` or the
   lock file — never sniffed from the host. "Native" is a convenience that
   resolves to a named target and writes it down.
5. **Deterministic ordering** across Limbs and within the analysis: parallel
   inside the compiler is fine only where the result cannot depend on who
   finishes first.
6. **Compile-time arithmetic matches the target.** A `bin16` folded at
   compile time rounds exactly as the target's `bin16` would.
7. **The Unicode version is pinned** (owner, 2026-09-24: "Pin it, but we
   should update when we should update"). Grapheme boundaries, case
   mapping and every other Unicode table come from one recorded version,
   written in the lock file beside the compiler version, so the same
   program counts the same text the same way on every build. Moving to a
   newer Unicode is a deliberate update, like a compiler upgrade, and it
   is made when a new version is worth adopting, not left to stall.

## Making it observable

Synthia records a **proof summary** in the lock file: for every check site,
proven / checked / nocheck. A rebuild whose summary differs is flagged
before anyone wonders why the binary changed. Determinism you cannot observe
is determinism you will not notice losing.
