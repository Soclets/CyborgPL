# Verifying the compiler

For CyborgPL's own developers — not for users of CyborgPL. Nothing here
ships or is user-configurable. Owner's direction, 2026-09-14: an oracle plus
the optimised generator.

## The two implementations

- **The oracle** — a reference interpreter: slow, small, written for
  obviousness, straight from the spec. No Precog, no LLVM, no optimisation.
  Correct by inspection. It is also the *executable specification*: what
  CyborgPL does in a given case is what the oracle does.
- **The generator** — the real compiler: Precog + LLVM, targeting the exact
  machine and using everything the hardware has.

## The method: differential testing

A program generator produces large numbers of random, valid CyborgPL programs
(as Csmith / YARPGen do for C and C++). Each runs through both
implementations. Any difference in output is a bug in one of them, and either
way it has been found. This runs continuously in the compiler's CI.

## Beyond testing

- **Translation validation** — for a compiled program, use the SMT solver
  (already present for Precog) to prove the LLVM-optimised IR is equivalent
  to the unoptimised IR, as Alive2 does. Covers the LLVM half; Precog's own
  transformations (folding, CTFE) are covered by the oracle. A compiler-CI
  tool, run over the test corpus.
- **Full formal verification** (CompCert-style, in a proof assistant) — the
  strongest guarantee and a decade of expert work. Ruled out for now.

## Three oracles (owner, 2026-09-21)

The interpreter is one of three independent checks, each of a different
kind — a second interpreter would only be the same reading again:

1. **The interpreter oracle** (exists, `oracle/`) — answers "what does this
   program *do*". Differential testing against the compiler on the
   examples and on generated programs.
2. **The Precog oracle** — answers "what can be *proven*". A small, slow,
   brute-force analyser: run a check site over every value its range type
   allows, every input up to a size, and say "safe" or show the
   counterexample. The real Precog is clever and fast; this one is dumb and
   exhaustive, and a disagreement is a Precog bug. It comes with the Precog
   design, which is not yet written.
3. **The translation-validation oracle** — answers "does the machine code
   still mean the source". Checks the compiler's LLVM IR against the
   program (Alive2's territory), as above.

Two of the three are mostly borrowed: translation validation feeds
Alive2; the Precog oracle can sit on an SMT solver (Z3), asking the dumb
literal question where Precog asks the clever one. The interpreter stays
ours — it is the spec.

**4. Twin programs — other languages as oracles** (owner, 2026-09-21).
A Cyborg program and the same algorithm in C, Rust, or Python, expected
to print the same thing (`examples/pi.cpls` has all three). Covers what
both languages define alike — arithmetic, widths, text output, lists —
and cannot share a misreading of our spec, having never read it. Cannot
cover what only Cyborg has: failures, ownership, check sites, range
types. Those stay with the oracle and the cold reads.

Each is authored independently of the compiler. None is user-facing.
Hobby language: an intention, not a schedule.

## Recommended order (Claude)

Build the oracle interpreter first, before the real compiler: it is cheap, it
is the spec, and every syntax/semantics decision from here on can be tested
against it.
