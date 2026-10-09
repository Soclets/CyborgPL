# Implementation languages

Decided 2026-09-14. Shaped by the team: Rust engineers as reviewers, AIs
writing most code, nobody who reviews C or OCaml.

| Component | Language | Notes |
|---|---|---|
| Compiler — front end, ownership/borrow checking, Cortex, LLVM lowering | **Rust** | LLVM via `llvm-sys`/`inkwell` (the C API). |
| LLVM wrapper | **C++** | The only non-Rust code. A small file exposing, as C functions, whatever the LLVM C API lacks (custom passes, pass-manager internals, new APIs). The rustc model (`RustWrapper.cpp`). Kept minimal. |
| Runtime — linked into every user binary (`deci`/`bin128` software arithmetic, `str`/list memory and drop glue, number↔text, check-failure handler, program entry, OS edge) | **`no_std` Rust** | `deci` written in Rust, not vendored from a C library. No C anywhere. |
| Standard Neuron | **Nerve** | Not a choice: a Neuron must be analysable source for Cortex. Sits on a small intrinsic layer over the runtime — the one boundary Cortex cannot see through, so each intrinsic declares its contract by hand. |
| Synthia | **Go** (owner, 2026-09-29; Rust until then) | A separate tool since the two package managers (2026-09-29): it reads `Synthia.ttsr`, fetches Neurons and runs `cca build …`, sharing no code with CCA. Go's standard library covers it; one cross-compiling binary. |
| Program generator and test/translation-validation harness | **Rust** | Shares the compiler's AST so generated programs are valid by construction. |
| Oracle interpreter | **Rust**, by a *different AI model* than the compiler, no shared code | Author independence in place of language independence. OCaml was considered and rejected — nobody on the team would review it. It was tried once, on 2026-09-17: a subagent wrote a 1,287-line OCaml interpreter (`oracle-ocaml/`, never committed), and one run against the Rust oracle found 7 disagreements, two of which became decisions (`1,000` is an error; verbs on a plain value are an error). The owner then had it deleted. |

## Talking to LLVM — three layers (decided 2026-09-21)

1. **`inkwell`** — the safe Rust wrapper. Nearly all lowering code.
2. **`llvm-sys`** — the raw C API, `unsafe`. Only for a call the C API has
   but inkwell has not wrapped yet. Same LLVM version as inkwell (inkwell
   brings its own llvm-sys; the two cannot point at different LLVMs).
3. **The C++ wrapper** — only for what the C API does not expose at all.

Each layer is reached only when the one above cannot do the job, so the
`unsafe` and the C++ stay as small as possible. The safety difference is
in the compiler's own code (misusing LLVM through raw pointers crashes the
compiler; inkwell's lifetimes catch most of that at compile time), never
in the binaries produced — both paths hand LLVM the same IR.

**The LLVM version follows inkwell.** On 2026-09-21: LLVM 23.1.1 was out,
`llvm-sys` 231 already on it, `inkwell` 0.10 on LLVM 22.1 — the usual lag
of a few weeks to months. The 22 → 23 differences that would touch Cyborg:
a C-API change to branch opcodes (inkwell's problem), a new IR byte type
(not needed), and an AArch64 inline-asm fix (dodged, see `ffi.md`). Owner
chose inkwell over bare llvm-sys after asking about the safety difference
and the version lag.

## Why not C++ throughout

C++ has zero friction with LLVM and was seriously considered. Rust + wrapper
reaches every LLVM feature (anything the C API lacks goes through the
wrapper), at the cost of a few weeks' lag behind each LLVM major and a small
C++ file; in exchange the 200k-line codebase gets memory safety and is in the
one language the team's humans review. C++ is the nearer neighbour to Rust
for that wrapper; C is not, which is why no C.
