# cca — the Cyborg Chassis Assembler

The CyborgPL compiler, built by the compiler session ("compiler claude")
from `design/` and the oracle's tests, never from `oracle/src`
(`design/compiler-session.md`).

```
cd cca
cargo build
./target/debug/cca parse ../examples/hello.cpls     # parse only; the first refusal, in the diagnostics.md shape
./target/debug/cca check ../examples/add.cpls       # parse and check
./target/debug/cca mir   ../examples/add.cpls       # check, then print the flow graph
./target/debug/cca precog ../examples/scores.cpls   # …then prove, and print the report
```

## Layout

- `compiler/` — the `cca` crate: `source` (files, spans), `diag` (the
  labelled message shape), `lexer`, `ast`, `parser`, `resolve` (items and
  modules), `check` (names, types, verbs, failures) producing `hir`, and
  `mir` (the flow graph, its domains, its verifier and its printer).
  Precog and the LLVM lowering come next.
- `runtime/` — `cca-runtime`, the `no_std` Rust library linked into every
  CyborgPL binary.

```
cargo test        # every examples/*.cpls checks; every reject program is refused
```

## Status

- 2026-09-22: lexer, parser, item resolution and the checker. Every
  `examples/*.cpls` loads and checks; 38 of the 39 reject *programs* CCA
  can decide are refused in CCA's own words (`compiler/tests/reject_phrases.txt`),
  and the 38 that need Precog are marked `!pending` there — the test
  insists they are still *accepted*, so the file has to be updated the day
  each one starts being refused. No MIR, no Precog, no LLVM yet.

## Provisional — picks made to keep moving, not decisions

Dated, with the alternative, so each can become a question later.

- **2026-09-22 — `cca/` is a Cargo workspace** (`compiler/` + `runtime/`),
  not one crate as the brief's wording has it: the runtime must be
  `no_std` and `staticlib`, which a second crate is the only way to say.
  Alternative: one crate with the runtime as a module and a build script.
- **2026-09-22 — `;` after a `concur` or `whichever` statement is
  optional.** The examples write it both ways (`whichever.cpls`); the
  oracle accepts both; every other statement needs its `;` and there are
  no empty statements. Alternative: require it always (would break the
  examples) or never (would break them too).
- **2026-09-22 — The target's SIMD shapes are ARM64's 128 bits, in the
  compiler.** `simd.md` says the shapes come from the target description
  in `JetPack.toml`, and JetPack does not exist yet, so
  `types::TARGET_SIMD_BITS` holds 128 — the oracle's fixed target and this
  machine's. It moves into the target description the day there is one.
  Alternative: refuse every `simd` until JetPack exists, which would leave
  `examples/simd.cpls` unbuildable.
- **2026-09-22 — A testing-only `std::` stub table.** `std::` does not
  exist (owner, 2026-09-22), but 55 of the 61 examples print with it. The
  checker has a flag (off by default, on in `cargo test` and with
  `CCA_STD_STUBS=1`) that gives the `std::` names the oracle emulates
  their types and nothing else — no code is generated for them. It exists
  so the front end can be tested against the examples today, and goes away
  when std is written. Alternative: test only the six extern-based
  examples until std exists.
- **2026-09-22 — Reject-test matching.** CCA's messages are in CCA's
  words, so the `.err` substring is not matched literally; the harness
  (to come) records, per reject test, a phrase of CCA's own message that
  was reviewed once against the `.err`, and fails when that phrase
  disappears. Alternative: match the oracle's substring literally, which
  would make the oracle's wording the template the brief says it is not.
