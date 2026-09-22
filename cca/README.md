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

./target/debug/cca build ../examples/… -o prog       # …then generate, link, and run
```

The build needs LLVM 22 where `cca/.cargo/config.toml` says it is, and the
runtime linked into every binary: `cargo build -p cca-runtime`.

```
cargo test        # every examples/*.cpls checks; every reject program is
                  # refused; CCA's own programs build, run and exit right
```

## Status

- 2026-09-22: the whole pipeline runs — lexer, parser, item resolution,
  the checker, HIR, MIR, the ownership walk, Precog (layers 1, 2 and 3)
  and LLVM, out to an object file and a linked binary. Every
  `examples/*.cpls` loads, checks and lowers to a well-formed graph, and
  none is refused by Precog. Of the 358 reject programs, 344 are refused
  in CCA's own words (`compiler/tests/reject_phrases.txt`); 9 are marked
  `!pending` and 5 `!by-design`. The test insists a `!pending` one is
  still *accepted*, so the file has to be updated the day each starts
  being refused.
- Over the examples, Precog settles 398 of 422 sites: layer 1 folds what
  it can, layer 2 proves most of the rest, and layer 3 is asked what is
  left. 21 sites keep a check at run time and 3 are `nocheck`.
- What has no code generated yet: the heap types (list, map, text
  building), failures as values, `concur`, `channel` and `inline.ASM`.
  CCA checks, proves and reports those programs; it refuses to write
  machine code for them rather than writing something wrong.

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
- **2026-09-22 — a `list` is 24 bytes even where Precog could prove it
  smaller.** `lists.md` says a sequence nothing appends to becomes an
  `array`, and a list whose final size is proven needs no capacity field.
  Neither is implemented, so a `SeqKind::Free` is lowered as the shape that
  can do both. Sound, and larger than it needs to be. The same is true of
  `map`: `MapOrder::Free` is lowered as the ordered shape. Alternative:
  refuse to lower until Precog settles the variant, which would block
  programs for a size question.
- **2026-09-22 — nothing frees a text yet.** Every text a program can build
  today is a constant, so nothing is on the heap and `Drop` emits nothing.
  The sentinel `types.md` describes — the high bit of the length, tested by
  the free where a name may hold either — is not written, because no name
  can yet hold a built text. It arrives with `++` at run time.
- **2026-09-22 — `bin16` and `bin128` constants are not rounded to their
  width at compile time.** A `bin32` constant is rounded exactly (through
  `f32`) and a `bin64` needs none, so both meet
  `reproducible-builds.md`'s "compile-time arithmetic matches the target".
  The other two widths keep the value as written, and it is only ever used
  to answer range questions — never folded into a binary — so nothing is
  wrong today. It has to be real arithmetic before either width ships.
  Alternative: refuse a `bin16` or `bin128` constant until then, which
  would make `examples/print_vs_printu.cpls` unbuildable for no gain.
- **2026-09-22 — `cca/.cargo/config.toml` points at Homebrew's LLVM 22.**
  The compiler's own dependency has to be found somewhere, and JetPack does
  not exist to say where. This is not the *target* being sniffed from the
  host (`tooling.md` forbids that, and nothing here reads it): it is where
  the build gets LLVM, as Kespar's config points at Z3. Alternative: an
  environment variable every build has to set.
- **2026-09-22 — A `lent` parameter crosses as a pointer.** A borrow is
  temporary access without ownership, and an address is what that is on
  the machine. `ownership-syntax.md` says what a lend *means* and not what
  it is made of, so this is a pick. Alternative: pass small lent values by
  copy and rely on Precog to prove nothing observes the difference — which
  is true today and stops being true the moment a `lend.mut` writes back.
- **2026-09-22 — A drop whose value is gone on *some* paths keeps its
  `Drop`.** MIR emits a drop per owned local at its scope's end; the
  ownership walk removes the ones definitely already given away, and
  counts the rest (`CCA_DROPS=1` prints both numbers). A conditional one
  needs a flag at run time saying whether the value is still there —
  nothing generates code yet, so nothing is wrong today, but the lowering
  must carry that flag when it arrives. Alternative: split the block so
  each path drops what it owns, which trades a flag for more blocks.
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
- **2026-09-22 — Layer 3's unrolling bounds.** One loop is unrolled 16
  times into a query and a whole query holds 64 unrollings
  (`UNROLL_PER_LOOP`, `UNROLL_TOTAL` in `compiler/src/precog/smt.rs`).
  `precog.md` leaves the numbers open and says a site inside a loop may
  be proven for the first trips and unproven after, which is exactly what
  these do: when the unrolling had to cut a path, an `unsat` is reported
  as *unproven*, never as a proof. Deeper made Z3 quadratic on nested
  loops. Alternative: prove loops by induction over an invariant instead
  of unrolling, which proves whole loops but needs invariants nothing
  writes yet.
- **2026-09-22 — The solver budget is 10,000,000 `rlimit`.** A step
  count, never a clock, so the same program gives the same answer on
  every machine (`precog.md`). It rides in the query text as
  `(set-option :rlimit …)`, which both Z3 and CVC5 honour, so no member
  of the portfolio is held to a budget the others were not. Alternative:
  a wall-clock timeout, which would make a proof depend on what else the
  machine was doing.
- **2026-09-22 — CVC5 installed by hand.** Homebrew has no `cvc5`
  formula, so the official release binary (cvc5 1.4.0,
  `cvc5-macOS-arm64-static.zip`, sha256
  `6560851586d3aabb2a97e0c19f5f75e28da88149dc916a373debe5f2e435d71c`) is
  at `~/.local/bin/cvc5`. The portfolio reports which members it found,
  because a portfolio missing a member is not the portfolio the design
  decided on. Alternative: build CVC5 from source, which pins the exact
  commit at the cost of a long build.
- **2026-09-22 — 128-bit whole numbers are refused.** Every number Precog
  reasons about is held in `i128`, so a `uint128` has values it cannot
  write down and an `int128` has sums it cannot; a proof about a number
  the prover cannot represent would not be a proof. `int128`/`uint128`
  are *proposed* in `design/types.md`, not decided, so CCA says so in a
  message instead of carrying them badly (it used to panic). Alternative:
  a wider value model throughout Precog's domains, which is real work for
  a width that may not survive the design.
- **2026-09-22 — An enum is a tag and the room beside it.** The tag is the
  smallest whole number that can name every case, so a three-case enum
  pays one byte for it; the room is an array whose element type carries
  the alignment the widest payload needs, which is what puts a payload at
  an address it can be stored at. `enums.md` decided what an enum
  *means* and left the shape open. Alternative: the payload as an LLVM
  union-by-bitcast of the largest case's own type, which is the same size
  and harder to read.
- **2026-09-22 — A lent local that is not a parameter holds an address.**
  A lent *parameter* already is one. Anything else — the name a `match`
  arm binds a payload to, a `lend` held in a variable — needs somewhere
  to keep the address it borrows, so its slot holds a pointer and reads
  go through it. Alternative: copy the borrowed value into the local,
  which is wrong the moment a `lend.mut` writes back.

