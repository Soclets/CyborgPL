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

## Recommended order (Claude)

Build the oracle interpreter first, before the real compiler: it is cheap, it
is the spec, and every syntax/semantics decision from here on can be tested
against it.
