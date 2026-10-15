# Distribution: the ZDK and the ZCK

Decided 2026-10-15 (owner): Zag ships **two downloads**, both packaged and versioned (owner: "There would
likely be two downloads, ZDK (Zag Dev Kit), and ZCK (Zag Contribute Kit)", then "(a)": the ZCK a real download,
not just the repository).

- **ZDK, the Zag Dev Kit**: for writing Zag.
- **ZCK, the Zag Contribute Kit**: for working on Zag itself.

## Contents (design claude's sketch, not ruled)

**ZDK**: Zomp (the compiler, the prover built in); the runtime and `std`; Ricochet once it has a command of its
own (2026-10-15: not yet); plugins such as `unicode`, perhaps optional; documentation and examples.

**ZCK**: what the ZDK holds, and: the oracle (the executable specification); the conformance tests (examples,
reject tests, `spellings.zv`); check's tools (the program generator, the differential harness, the Alive2 runs,
kept for good on 2026-10-15); the benchmarks. Tools it needs from elsewhere (Z3, cvc5, Alive2, LLVM): bundled,
or fetched by the kit; open.

## Open

- What exactly each holds, and whether the ZCK contains the ZDK or sits beside it.
- Platforms (Zomp builds for aarch64-macos and aarch64-none today), package formats, and how versions are
  numbered (the language has a date, `zag_version.zv`; Zomp says 0.0.1).
- How a ZCK user's changes get back: the ZCK is packaged, so it is not the repository itself.
