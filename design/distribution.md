# Distribution: the CyDT and the CyCT

Decided 2026-10-15 (owner), for Zag: **two downloads**, both packaged and versioned (owner: "There would
likely be two downloads, ZDK (Zag Dev Kit), and ZCK (Zag Contribute Kit)", then "(a)": the ZCK a real download,
not just the repository). For CyborgPL (2026-10-16) they are tools, not kits, since a kit is a library (owner:
"CDT, we could. So it won't clash with our kits", then "Yup, or we could ad Cy", then "Yup"):

- **CyDT, the CyborgPL Dev Tools**: for writing CyborgPL (was the ZDK).
- **CyCT, the CyborgPL Contribute Tools**: for working on CyborgPL itself (was the ZCK).

**Numbering: whole numbers, no dots** (owner, 2026-10-15: "There are no dots, it's just ZDK 1, the next
ZDK 2"): CyDT 1, then CyDT 2. **The CyCT carries the same number** as the CyDT it goes with, release for
release (owner: "yea"): CyCT 1 with CyDT 1. **The compiler names itself by its tools**, `CyDT 1 KAYO\n` (owner,
of Zag's: "ZDK 1 Zomp"), with no dotted version of its own.

## Contents (design claude's sketch, not ruled)

**CyDT**: `KAYO\n` (the compiler, Precog built in); the runtime and `std`; CyJet once it has a command of its
own (2026-10-15: not yet); limbs such as `unicode`, perhaps optional; documentation and examples.

**CyCT**: what the CyDT holds, and: the oracle (the executable specification); the conformance tests (examples,
reject tests, `spellings.cyr`); check's tools (the program generator, the differential harness, the Alive2 runs,
kept for good on 2026-10-15); the benchmarks. Tools it needs from elsewhere (Z3, cvc5, Alive2, LLVM): bundled,
or fetched by the tools; open.

## Open

- What exactly each holds, and whether the CyCT contains the CyDT or sits beside it.
- Platforms (`kayo` builds for aarch64-macos and aarch64-none today) and package formats.
- What a build between releases calls itself; how the language's date (`cy_ver.cyr`) relates to the
  kit's number.
- How a CyCT user's changes get back: the CyCT is packaged, so it is not the repository itself.
