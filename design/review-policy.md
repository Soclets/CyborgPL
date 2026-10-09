# Code review policy

Decided 2026-09-14. AI writes most of the code; human review effort follows
risk, not line count.

## Tier 1 — every line human-reviewed (the trusted base)

A bug here does not crash Nerve; it makes Nerve *lie*.

- **Precog** — a wrong proof removes a check that was needed.
- **Borrow checker and lifetime inference** — a wrong "ok" is a
  use-after-free in a user's binary.
- **Proof-feeding into LLVM** — a wrong `nsw`/`nuw`/`!range`/`assume` is a
  silent miscompile.
- **Intrinsic contracts** between the standard Neuron and the runtime —
  hand-declared facts Precog trusts without proof.

## Tier 2 — tested and sampled, not fully reviewed

Bugs here are ordinary: visible, reproducible, caught by the oracle or tests.

- Front end (lexer, parser, name resolution).
- Runtime arithmetic (`deci`, `bin128`) — algorithms reviewed, lines trusted
  to a large test corpus.
- Synthia, the program generator, the test harness.
- The oracle interpreter — its correctness comes from disagreement-hunting
  against the compiler, not from reading.

## Tier 3 — reviewed by Precog

The standard Neuron is Nerve source; the language's own guarantees apply to
it.

## Why

Full review of every line would gate the project to ~1–2k reviewed lines per
reviewer per day. Concentrating human attention on the ~100–150k lines whose
failure is silent keeps correctness where it matters and lets differential
testing carry the rest — which is why the oracle should exist early.
