# Verifying the compiler

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

For Nerve's own developers — not for users of Nerve. Nothing here
ships or is user-configurable. Owner's direction, 2026-09-14: an oracle plus
the optimised generator.

## The two implementations

- **The oracle** — a reference interpreter: slow, small, written for
  obviousness, straight from the spec. No Cortex, no LLVM, no optimisation.
  Correct by inspection. It is also the *executable specification*: what
  Nerve does in a given case is what the oracle does.
- **The generator** — the real compiler: Cortex + LLVM, targeting the exact
  machine and using everything the hardware has.

## The method: differential testing

A program generator produces large numbers of random, valid Nerve programs
(as Csmith / YARPGen do for C and C++). Each runs through both
implementations. Any difference in output is a bug in one of them, and either
way it has been found. This runs continuously in the compiler's CI.

## Beyond testing

- **Translation validation** — for a compiled program, use the SMT solver
  (already present for Cortex) to prove the LLVM-optimised IR is equivalent
  to the unoptimised IR, as Alive2 does. Covers the LLVM half; Cortex's own
  transformations (folding, CTFE) are covered by the oracle. A compiler-CI
  tool, run over the test corpus.
- **Full formal verification** (CompCert-style, in a proof assistant) — the
  strongest guarantee and a decade of expert work. Ruled out for now.

## Printing while there is no std (owner, 2026-09-23)

A generated program must print something, or the harness has nothing to
compare but the exit code. Synapse refuses every `std::` call until std
exists (owner, 2026-09-22), so **each generated program carries a small
printer written in the base language**: the number turned into ASCII
digits in a `list[uint8]`, handed to C's `write` under `!check`, as
`examples/ffi.nrv` does. Both implementations already agree on it.
The printer is ordinary Nerve, so a bug in *it* is a finding too.
When std has a print, the generator switches to `std::print.useful` and the
printer goes. Proposed alternative: wait for std, and compare only the
exit code and where a check fired — which would delay every output bug
until then.

## Input while there is no std (owner, 2026-09-23)

A program that reads nothing is run to the end at compile time
(`precog.md`, layer 1), so its binary is only its output: testing it
exercises Cortex's compile-time execution and nothing else — not the
machine code, not the runtime checks, not layers 2 and 3. Generated
programs therefore need a value the compiler cannot see, and until std
has input they get it as **the length of an environment variable**,
through `extern.C fn getenv` and `strlen`, which both implementations
already run. The harness sets the variable to N characters; the program
builds whatever value it wants from N, bounded by a range type or a
conversion. It switches to `std::input` when std has one. Proposed
alternative: wait for std, and test only what a fully known program
tests.

This channel found three Synapse bugs on the day it was proposed, probed by
hand — the worst a **wrong proof**: `'n' times 10000000000000000000` into a
`uint64` reported proven, with the binary silently overflowing at n = 2
where the oracle stops. That is the class of bug the proofs-as-
assertions mode exists for, and the reason the verification session
does not read `synapse/`.

## Three oracles (owner, 2026-09-21)

The interpreter is one of three independent checks, each of a different
kind — a second interpreter would only be the same reading again:

1. **The interpreter oracle** (exists, `oracle/`) — answers "what does this
   program *do*". Differential testing against the compiler on the
   examples and on generated programs.
2. **The Cortex oracle** — answers "what can be *proven*". Three attacks
   on every claim in Cortex's report (decided 2026-09-22, `precog.md`):
   exhaustive/random execution over the domains; an independent simple
   interval analysis; and the negation of every solver-proven site,
   re-encoded by a second author. A disagreement is a Cortex bug.
3. **The translation-validation oracle** — answers "does the machine code
   still mean the source". Checks the compiler's LLVM IR against the
   program (Alive2's territory), as above.

Two of the three are mostly borrowed (owner, 2026-09-22: "Borrow it").
Translation validation uses **Alive2** for the LLVM-IR half — ~37k lines
of C++ (2026-09-21), of which ~20k is a formal semantics of LLVM IR kept
in step with LLVM by people who work on LLVM, run in LLVM's own CI and
credited with hundreds of LLVM bugs; its trust is that history, which a
rewrite could not have. Ours is the Cyborg half: that MIR-with-proofs
means what the source meant, and that the IR handed to LLVM matches
MIR — a few hundred to a few thousand lines. The Cortex oracle can sit
on an SMT solver (Z3), asking the dumb literal question where Cortex
asks the clever one. The interpreter stays ours — it is the spec.

**Our own: `RRx2-TV`** (owner, 2026-09-30: "Nah, `RRx2-TV` seems more
fun"). Translation validation for the LLVM half becomes ours, not
borrowed: a checker named RRx2-TV, which can check a whole program where
Alive2 checks one function at a time (Synapse folds most of a program into
`main`), and can read Cortex's facts directly. Check claude ran
Alive2 first (93c6e49); Nerve is paused until RRx2-TV exists (owner). The paragraph above is why "Borrow it"
was chosen, and it still holds as the cost: most of Alive2 is an exact
meaning for every LLVM instruction, and a mistake there says "proven"
where nothing is. Like check claude's tools, it must not share Cortex's
author. Open: who writes it, in what language, in which repo, and
whether it starts from Alive2's code (MIT) or from nothing.

**Sessions** (owner, 2026-09-22): (1) this one — design and the
oracle, never a line of the compiler; (2) a fresh session — Synapse
(front end, checker, MIR, Cortex, lowering, runtime) and Synthia,
reading `design/` and the oracle's tests, never its source; (3) later,
another fresh session — verification: the Cortex oracle, translation
validation, the generator, the differential harness, by a different
author than Cortex's.

**4. Twin programs — other languages as oracles** (owner, 2026-09-21).
A Cyborg program and the same algorithm in C, Rust, or Python, expected
to print the same thing (`examples/pi.nrv` has all three). Covers what
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
