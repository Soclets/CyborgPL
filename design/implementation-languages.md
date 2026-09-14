# Implementation languages

Decided 2026-09-14. Shaped by the team: Rust engineers as reviewers, AIs
writing most code, nobody who reviews C or OCaml.

| Component | Language | Notes |
|---|---|---|
| Compiler — front end, ownership/borrow checking, Precog, LLVM lowering | **Rust** | LLVM via `llvm-sys`/`inkwell` (the C API). |
| LLVM wrapper | **C++** | The only non-Rust code. A small file exposing, as C functions, whatever the LLVM C API lacks (custom passes, pass-manager internals, new APIs). The rustc model (`RustWrapper.cpp`). Kept minimal. |
| Runtime — linked into every user binary (`deci`/`bin128` software arithmetic, `str`/list memory and drop glue, number↔text, check-failure handler, program entry, OS edge) | **`no_std` Rust** | `deci` written in Rust, not vendored from a C library. No C anywhere. |
| Standard Limb | **CyborgPL** | Not a choice: a Limb must be analysable source for Precog. Sits on a small intrinsic layer over the runtime — the one boundary Precog cannot see through, so each intrinsic declares its contract by hand. |
| JetPack | **Rust** | |
| Program generator and test/translation-validation harness | **Rust** | Shares the compiler's AST so generated programs are valid by construction. |
| Oracle interpreter | **Rust**, by a *different AI model* than the compiler, no shared code | Author independence in place of language independence. OCaml was considered and rejected — nobody on the team would review it. |

## Why not C++ throughout

C++ has zero friction with LLVM and was seriously considered. Rust + wrapper
reaches every LLVM feature (anything the C API lacks goes through the
wrapper), at the cost of a few weeks' lag behind each LLVM major and a small
C++ file; in exchange the 200k-line codebase gets memory safety and is in the
one language the team's humans review. C++ is the nearer neighbour to Rust
for that wrapper; C is not, which is why no C.
