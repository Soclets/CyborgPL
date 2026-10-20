# Reproducible builds

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

Decided 2026-09-14. Same source, same compiler, same *named* target →
byte-identical binary, on any machine, any day.

## Why CyborgPL is more exposed than other compilers

In C or Rust, nondeterministic optimisation yields a slower or faster binary
that does the same thing. In CyborgPL, the prover's proofs decide **behaviour**:
a bound proven on one machine and not on another means one binary has no
runtime check and the other has one — two different programs. SMT solvers
are nondeterministic by default (time limits, seeds, memory, clause order),
and "compile for the exact machine" makes the target part of the input.

## Requirements on the prover and the build

1. **No wall-clock anywhere in the prover.** Every budget is a step count —
   solver steps, unroll depth, invariant candidates tried — never seconds.
2. **Solver determinism pinned.** Fixed seeds and configuration; results may
   not depend on thread scheduling. The solver's version is part of the
   compiler's version: a solver upgrade is a compiler change.
3. **Compile-time I/O is always a boundary.** the prover never reads a file,
   clock, or environment variable at compile time. Those are bounded values
   with explicit types like any other input. (Closes the earlier open
   question.) **Revised 2026-10-08** (owner: "We should have env vars"):
   a build may read any environment variable, freely, as Rust's `env!`
   (owner: "`c`"); a build that reads one is repeatable only with the same
   values, which nothing records. **Revised 2026-10-10**: a build may also
   put a file's bytes into the program (include bytes), from any path; the
   registry refuses a published kit that reads a file outside its own
   folder (owner: "(c)"), so a kit built from source elsewhere finds
   every file it includes. The file counts as source: its bytes are part
   of what the build depends on. It is spelled `incl.bytes["logo.png"]`, is
   the base's (no limb, no `std`), and gives an `array[u8]` held
   in the program file; it is a const's value only (`const 'logo' :=
   incl.bytes[…];`), read where it sits and copied only where something
   takes it to keep or change. Its text twin names the type, `incl.str.utf8[…]`
   (or `.ascii`, or `.utf16` / `.utf32` with UniTect, which always say their
   byte order: `incl.str.utf16.low_first[…]` / `.high_first`), and the build refuses
   a file not valid in it; it is a const's value only, as bytes are. A path is from the kit's root, as `my.` is.
   DECISIONS 2026-10-10 has the rulings.
4. **Measurements are inputs too** (owner, 2026-09-28): `synapse tune` times
   loops once, on purpose, and writes `tune.cyr` (`synthia tune` runs it); builds read the file
   and never time anything themselves (`gpu.md`).
5. **The target is an explicit, recorded input** — in the manifest (`Synthia.ttsr` / `CyJet.cyr`) or the
   lock file — never sniffed from the host. "Native" is a convenience that
   resolves to a named target and writes it down.
6. **Deterministic ordering** across kits and within the analysis: parallel
   inside the compiler is fine only where the result cannot depend on who
   finishes first.
7. **Compile-time arithmetic matches the target.** A `f16` folded at
   compile time rounds exactly as the target's `f16` would.
8. **The Unicode version is pinned** (owner, 2026-09-24: "Pin it, but we
   should update when we should update"). Grapheme boundaries, case
   mapping and every other Unicode table come from one recorded version,
   written in the lock file beside the compiler version, so the same
   program counts the same text the same way on every build. Moving to a
   newer Unicode is a deliberate update, like a compiler upgrade, and it
   is made when a new version is worth adopting, not left to stall.

## Making it observable

Synthia records a **proof summary** in the lock file: for every check site,
proven / checked / `no_check`. A rebuild whose summary differs is flagged
before anyone wonders why the binary changed. Determinism you cannot observe
is determinism you will not notice losing.
