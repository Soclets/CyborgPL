# cca — the Cyborg Chassis Assembler

The CyborgPL compiler, built by the compiler session ("compiler claude")
from `design/` and the oracle's tests, never from `oracle/src`
(`design/compiler-session.md`).

```
cd cca
cargo build
./target/debug/cca parse ../examples/hello.cpls     # parse only; the first refusal, in the diagnostics.md shape
```

## Layout

- `compiler/` — the `cca` crate: `source` (files, spans), `diag` (the
  labelled message shape), `lexer`, `ast`, `parser`; then the checker,
  MIR, Precog and the LLVM lowering as they arrive.
- `runtime/` — `cca-runtime`, the `no_std` Rust library linked into every
  CyborgPL binary.

## Status

- 2026-09-22: lexer and parser cover every decided spelling; every
  `examples/*.cpls` parses; the syntax-level reject tests are refused with
  a message that says what the `.err` says. No checker, no MIR, no LLVM yet.

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
- **2026-09-22 — Reject-test matching.** CCA's messages are in CCA's
  words, so the `.err` substring is not matched literally; the harness
  (to come) records, per reject test, a phrase of CCA's own message that
  was reviewed once against the `.err`, and fails when that phrase
  disappears. Alternative: match the oracle's substring literally, which
  would make the oracle's wording the template the brief says it is not.
