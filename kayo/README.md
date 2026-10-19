# KAYO\n — the CyborgPL compiler

The compiler for CyborgPL (Nerve and then Zag for a while, CyborgPL again
since 2026-10-15), built by the compiler session ("compiler claude") from
`design/` and the oracle's tests, never from `oracle/src`
(`design/compiler-session.md`).

**The name** (owner, 2026-10-15): **KAYO\n**, written plainly `kayo` for
the command, the folder and anything a machine types, as C# is `csharp`.
It was CCA, then Synapse, then Zomp: `cca`, `synapse`, `syn`, `sng`, `suc`
and `zomp` are refused, naming `kayo`, as any old spelling is, and so are
the old `CCA_*`, `SYNAPSE_*` and `ZOMP_*` switches, naming their `KAYO_*`
ones. The crates take the full name (owner, 2026-10-08: "Yeah"):
`kayo-compiler` (its library `kayo`), `kayo-base`, `kayo-runtime` and
`kayo-limb-*`. It holds the base (the language's lowest level,
`compiler/`), the limbs (`limbs/`), Precog, the prover
(`compiler/src/precog`), and the runtime (`runtime/`). It names itself by
its tools (owner, 2026-10-16): a release by its number, `CyDT 1 KAYO\n`,
and a build between releases by the commit it was built from,
`CyDT 5eac982a KAYO\n`, with a `+` after it when what it is built from
has changes not committed (`CyDT 5eac982a+ KAYO\n`; "Yeah, add the +").

```
cd kayo
cargo build
./target/debug/kayo parse ../examples/hello.cy     # parse only; the first refusal, in the diagnostics.md shape
./target/debug/kayo check ../examples/add.cy       # parse and check
./target/debug/kayo mir   ../examples/add.cy       # check, then print the flow graph
./target/debug/kayo prove ../examples/scores.cy   # …then prove, and print the report
```

## Layout

Split as `LIMBS.md` lays out (step 1, 2026-10-07; the owner's answers
are in it):

- `compiler/` — the `kayo-compiler` crate (library `kayo`): the base, the lowest level of the
  language since there is no base and no codextras (owner, 2026-10-07),
  still holding the whole language and the `kayo` binary: `source` (files,
  spans), `diag` (the labelled message shape), `lexer`, `ast`, `parser`,
  `resolve` (items and modules), `check` (names, types, verbs, failures)
  producing `hir`, `mir` (the flow graph, its domains, its verifier and
  its printer), `precog` and `codegen`. Its folder keeps its name, since
  other sessions read `kayo/compiler/tests`.
- `base/` — `kayo-base`, the base's power ledger (step 2), kept a crate
  of its own.
- `limbs/<name>/` — one crate per official limb (a core until 2026-10-09; `kayo-limb-ieg`, …),
  each its name, registry ID and version for now; each limb's features
  move in at its step.
- `runtime/` — `kayo-runtime`, **the runtime** (the runtime's name, owner
  2026-10-09; the crate keeps its name until the code-name step), the
  `no_std` Rust library linked into every
  CyborgPL binary.

./target/debug/kayo build ../examples/… -o prog       # …then generate, link, and run
```

The build needs LLVM 22 and lld 22 where `kayo/.cargo/config.toml` says they
are (Homebrew's `llvm@22` and `lld@22` for now), a C++ compiler for the lld
shim (`compiler/lld/shim.cpp`), and the runtime linked into every binary:
`cargo build -p kayo-runtime`. KAYO\n links with lld through its library
(owner, 2026-10-08: "We'll use `lld`", then "`a`", Zig's way), so LLVM is
linked as one shared library, the one lld's libraries use; a self-contained
release will need LLVM and lld built as static libraries. Linking is built
for macOS hosts so far (the SDK found by `xcrun`).

**Of std, `std::print` and `std::exit`** (owner, 2026-10-09: "`b`, not
the entire fucking neuron"; "No `std::` for now" stands for the rest).
`print.useful` and `print.stored`, each to `.stdout` (the default) or
`.stderr`, `print.useful.auto_newln` and the two `.weird` forms build: each
value becomes its text as `to.str.utf8` makes it (`compiler/src/codegen/
print.rs`), and the pieces go out in one `writev`; `std::exit[code]` is the
OS's `exit`, and as an `otherwise` it leaves. Every other print
(`print.format`, `format[…]`) is refused by name, and the rest of std as
before. Every example is built and run against the oracle's `.out` and
`.exit` (`examples_run_as_the_oracle_prints`); the ones that cannot build
yet are in `tests/examples_unbuilt.txt` with their refusal's words.

**A chip with no OS: `aarch64-none`** (owner, 2026-10-09). Built from any
host, run on the bare chip (tested in QEMU's `virt` machine,
`compiler/tests/bare/`). The kit names its own linker script on the
target, its path from the kit's folder: `cyjet::Target[name =
"aarch64-none", layout = "boards/virt.ld"]` ("`c` now", then "`b`"); a lone
file is not built for it. The script keeps `_start` first (`ENTRY(_start)`,
"`a`") and defines `__stack_top`, `__bss_start` and `__bss_end` ("Whatever's
accurate"), each refused by name when missing. KAYO\n writes `_start`:
only core 0 goes on, the FP/SIMD unit is turned on, a vector table whose
entries wait forever is installed, the stack is set and the zeroed memory
cleared, then MAIN runs; when it ends the CPU sleeps forever (`wfe`, "`a`").
A failed run-time check is `brk #1` (2026-10-08: "yea, freeze there"): the
CPU stops in the table, `ELR_EL1` at the trap and `ESR_EL1` saying it was
one. `memcpy`, `memset`, `memmove` and `memcmp` are KAYO\n's own, each kept
only if used (`--gc-sections`). the runtime is not built for it, so a program
needing it, or C, is refused before linking, naming the function.

```
cargo test        # every examples/*.cy checks; every reject program is
                  # refused; KAYO\n's own programs build, run and exit right
```

## Status

- 2026-09-22: the whole pipeline runs — lexer, parser, item resolution,
  the checker, HIR, MIR, the ownership walk, the prover (layers 1, 2 and 3)
  and LLVM, out to an object file and a linked binary. The heap is in:
  lists, text built at run time, maps, enums and `match`, and failures as
  values, all of them freeing what they take. Every
  `examples/*.cy` loads, checks and lowers to a well-formed graph, and
  none is refused by the prover. Of the 358 reject programs, 344 are refused
  in KAYO\n's own words (`compiler/tests/reject_phrases.txt`); 9 are marked
  `!pending` and 5 `!by-design`. The test insists a `!pending` one is
  still *accepted*, so the file has to be updated the day each starts
  being refused.
- Over the examples, the prover settles 398 of 422 sites: layer 1 folds what
  it can, layer 2 proves most of the rest, and layer 3 is asked what is
  left. 21 sites keep a check at run time and 3 are `nocheck`.
- 2026-09-24: every base-language example builds; what has no code yet
  is `std::` (none for now, owner). KAYO\n refuses to write machine code it
  has no shape for rather than writing something wrong.
- 2026-09-27: the 2026-09-25 syntax (no `let`, `change`, `!$`,
  `and.then` / `or.else`); `layout.C enum` and ranged `layout.C` fields,
  checked at the door when a call returns them. A `layout.C` struct or C
  enum crosses to C as C passes it (`codegen/cabi.rs`, AAPCS64: packed
  registers, float HFAs, `sret` over 16 bytes); another machine's
  convention is not written, and KAYO\n refuses there. C's numbers are
  checked wherever they come in (owner, 2026-09-27): a return, a
  `ptr.read`, and every element of a list lent to C through a `ptr.mut`,
  checked when the call ends; `to.Key[…]` converts a number; a callback's
  parameter is never a C enum; a C variable holds C's plain number, and
  `to.int32[Mode.vi]` converts back (2026-09-28); `NO_FUCKING_CHECKS`
  trusts C instead. Not built: a callback that takes or returns a struct
  by value (it needs a thunk in C's convention).
- 2026-09-27: `pos` removed; `#strict` (the written-out syntax, and each
  site proven from its own function, in every build).
- 2026-09-29: a `match` gives a value; `for 'i', 'x'` gives the position;
  a function hands back a borrow (`-> lent@x T`), and borrows are checked
  Rust-style — while one is in use (to its last use), what it borrows is
  not given away, changed or lent `mut`, and a `lent.mut` one is used by
  nobody else (`mir/borrows.rs`).

- `kayo build --assert-proofs file.cy -o out` (2026-09-23): the build that
  checks the prover (`precog.md`, attack (d)). Every site the prover proved keeps
  a run-time check, nothing proven is promised to LLVM, and layer 1's
  folded answers are computed at run time instead. A wrong proof prints
  `proof failed: <the site's sentence>` and `where: file, line N, column
  M`, and exits **104** (an ordinary check that fires exits 101). `nocheck`
  sites proved nothing and are not asserted. The verification session's
  harness runs every generated program this way.

## Provisional — picks made to keep moving, not decisions

Dated, with the alternative, so each can become a question later.

- **2026-09-22 — `kayo/` is a Cargo workspace** (`compiler/` + `runtime/`),
  not one crate as the brief's wording has it: the runtime must be
  `no_std` and `staticlib`, which a second crate is the only way to say.
  Alternative: one crate with the runtime as a module and a build script.
- **2026-09-22 — `;` after a `concur` or `whichever` statement is
  optional.** The examples write it both ways (`whichever.cy`); the
  oracle accepts both; every other statement needs its `;` and there are
  no empty statements. Alternative: require it always (would break the
  examples) or never (would break them too).
- **2026-09-22 — The target's SIMD shapes are ARM64's 128 bits, in the
  compiler.** `simd.md` says the shapes come from the target description
  in `Synthia.cplm`, and Synthia does not exist yet, so
  `types::TARGET_SIMD_BITS` holds 128 — the oracle's fixed target and this
  machine's. It moves into the target description the day there is one.
  Alternative: refuse every `simd` until Synthia exists, which would leave
  `examples/simd.cy` unbuildable.
- **2026-09-22 — a `list` is 24 bytes even where the prover could prove it
  smaller.** `lists.md` says a sequence nothing appends to becomes an
  `array`, and a list whose final size is proven needs no capacity field.
  Neither is implemented, so a `SeqKind::Free` is lowered as the shape that
  can do both. Sound, and larger than it needs to be. The same is true of
  `map`: `MapOrder::Free` is lowered as the ordered shape. Alternative:
  refuse to lower until the prover settles the variant, which would block
  programs for a size question.
- **2026-09-22 — `kayo/.cargo/config.toml` points at Homebrew's LLVM 22.**
  The compiler's own dependency has to be found somewhere, and Synthia does
  not exist to say where. This is not the *target* being sniffed from the
  host (`tooling.md` forbids that, and nothing here reads it): it is where
  the build gets LLVM, as Kespar's config points at Z3. Alternative: an
  environment variable every build has to set.
- **2026-09-22 — A `lent` parameter crosses as a pointer.** A borrow is
  temporary access without ownership, and an address is what that is on
  the machine. `ownership-syntax.md` says what a lend *means* and not what
  it is made of, so this is a pick. Alternative: pass small lent values by
  copy and rely on the prover to prove nothing observes the difference — which
  is true today and stops being true the moment a `lend.mut` writes back.
- **2026-09-22 — A drop whose value is gone on *some* paths asks a flag.**
  MIR emits a drop per owned local at its scope's end; the ownership walk
  removes the ones definitely already given away (`KAYO_DROPS=1` counts
  both). Where a value is gone on some paths only, each owning local has an
  "alive" flag at run time — set when it gets a value, cleared when it is
  given away — and the drop tests it. Alternative: split the block so each
  path drops what it owns, which trades a flag for more blocks.
- **2026-09-22 — A testing-only `std::` stub table.** `std::` does not
  exist (owner, 2026-09-22), but 69 of the 74 examples use it. The
  checker has a flag (off by default, on in `cargo test` and with
  `KAYO_STD_STUBS=1`) that gives the `std::` names the oracle emulates
  their types and nothing else — no code is generated for them. It exists
  so the front end can be tested against the examples today, and goes away
  when std is written. Alternative: test only the six extern-based
  examples until std exists.
- **2026-09-22 — Reject-test matching.** KAYO\n's messages are in KAYO\n's
  words, so the `.err` substring is not matched literally:
  `compiler/tests/reject_phrases.txt` records, per reject test, a phrase
  of KAYO\n's own message, reviewed against the `.err`, and the test fails
  when that phrase disappears — and, since 2026-09-24 (owner), when the
  `.err` is no longer the one it was reviewed against (an FNV-1a
  fingerprint beside each phrase). Alternative: match the oracle's
  substring literally, which would make the oracle's wording the template
  the brief says it is not.
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
  `(set-option :rlimit …)`, which both Z3 and CVC5 honour. CVC5 counts
  far coarser steps than Z3 — a query Z3 answered in 0.04 s at 10,000,000
  kept CVC5 working for minutes at that number — so since 2026-09-30 it
  is asked with its own 1,000,000, about the same work, still a step
  count. Both members are always asked, to their own budgets. Alternative:
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
- **2026-09-22 — 128-bit whole numbers are refused.** Every number the prover
  reasons about is held in `i128`, so a `uint128` has values it cannot
  write down and an `int128` has sums it cannot; a proof about a number
  the prover cannot represent would not be a proof. `int128`/`uint128`
  are *proposed* in `design/types.md`, not decided, so KAYO\n says so in a
  message instead of carrying them badly (it used to panic). Alternative:
  a wider value model throughout the prover's domains, which is real work for
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
- **2026-09-22 — A function that can fail hands back a variant.** Case 0
  is the answer, each later case is one of the reasons, and the caller
  reads the tag: the same shape as an enum, because it is the same idea.
  `failures.md` decided the model — "the failure is a value the caller
  must handle" — and said nothing about the machine. Alternative: an
  out-parameter for the answer and the reason in the return register,
  which is smaller for a wide answer and worse for every other case.
- **2026-09-22 — Every text handle carries the mark, for now.** The owner
  decided (c): the prover knows which memory a text is in and a free is
  emitted only where it can be the heap, with the sentinel — the high bit
  of the length — as the fallback for a name that may hold either. the prover
  does not carry that fact yet, so KAYO\n pays the fallback everywhere: a
  literal's length has the bit set, `str.bytes` masks it off, and the free
  tests it. This is the decided mechanism at the decided fallback, not a
  different design; what is missing is the analysis that would make it
  free. Alternative: leak every text until the analysis exists, which is
  what the previous commit did.
- **2026-09-22 — A map's table is the runtime's; its types are the
  compiler's.** `maps.md` decided the shape — one flat table, open
  addressing, three handle fields unordered and four ordered, insertion
  order for the walks — and the runtime implements exactly that, taking
  the key and value sizes and the functions that hash, compare and drop
  one of them. Those are generated per type, by structure, which is what
  "`map[Point, str.utf8]` just works" asks for. `map.keys`/`values`/
  `pairs` build an array of *copies*: the array owns what is in it and is
  given back at the end of the walk, so it must not take the map's own
  keys with it. Alternative: a cursor that borrows, which is cheaper and
  needs a borrow that outlives the loop body to be proven safe.
- **2026-09-23 — Tasks are green, on one worker thread per core.**
  `concurrency.md` says tasks are "lighter tasks multiplexed over" the
  pool, so a task is a stack of its own and a saved stack pointer, and a
  task that waits is parked rather than blocking a thread
  (`runtime/src/tasks.rs`, with the context switch in assembly for
  arm64 and x86-64, both tested — x86-64 under Rosetta). A program with
  no `concur` starts no threads at all. The picks inside it:
  one run queue under one lock (not work stealing); **256 KiB of stack
  per task** and 8 MiB for MAIN when it runs as one, virtual, with a
  64 KiB guard below that stops a runaway recursion instead of letting it
  write over the next thing; the worker count is the number of online
  cores. Alternatives: an OS thread per task (simple, and exactly what the
  design ruled out); stackless tasks through LLVM's coroutine lowering
  (no stack per task, but every function a task might wait in would
  have to be compiled twice).
- **2026-09-23 — A block that leaves early waits for its tasks; it does
  not yet cancel them.** `concurrency.md`: "a block that fails cancels
  the tasks it still has running". Cancelling a task mid-body needs points
  where it can stop and the drops that run when it does, and what a
  cancelled task *sees* is open in the design. Until then an early
  `return`, `fail` or `break` waits for the block's tasks exactly as the
  block's end does — the same result whenever the tasks would have
  finished anyway, and a hang where one would have waited forever.
- **2026-09-23 — Layer 3 stands in for the trips it did not unroll.** At
  the last level a loop head is copied at, everything the loop writes is
  forgotten and everything it leaves alone is kept, and layer 2's facts
  for that head (the join of every state it ever saw arrive, over every
  trip and call) narrow what was forgotten. That copy is every later trip
  at once, so a loop no longer makes an `unsat` into a proof about a
  prefix; `UNROLL_TOTAL` running out still does. The facts are used only
  when layer 2 followed every call (the same `gave_up` gate as layer-1
  folding). Alternative: loop invariants found by induction for every
  loop, not only the template shapes (the sum-over-length one proves
  gradebook since 2026-09-24).
- **2026-09-23 — Channels: the counting, none of the prover's three parts yet.**
  The layout is the owner's (one block, slots inline, never growing; the
  program holds a pointer to it) and so is `closed` (count the tasks that
  can reach it; a `take` on an empty channel fails `closed` when every
  holder is itself waiting to take from it). What is built is the plain
  form: every channel on the heap, every slot the type allows, and the
  counting everywhere. Of the prover's half of `closed`, one part is built
  (2026-09-23, `mir/channels.rs`): the task that made a channel lets go
  of it where it provably never names it again — directly or through a
  borrow held in another name — rather than at the end of its block, so
  MAIN lending one channel to a producer and a consumer no longer keeps
  the consumer's last `take` waiting for ever (the oracle says `closed`,
  and so does KAYO\n now). A channel ever given away, or a name assigned
  twice, is left to the counting. Still not built: deleting the counting
  where the close is proven, and the three layout parts — the stack, fewer
  slots than the bound, no channel at all for one hand-off. Those are
  additions, not requirements. `whichever` tries its arms in the order written, so of several
  ready at once the first wins and a run is reproducible.

- **2026-09-23 — Which C calls end the program, for "reached on every
  run".** A certain failure is an error only where it runs on every run
  (owner, 2026-09-23). A call into C normally returns, but `exit`,
  `_exit`, `_Exit`, `abort` and `quick_exit` do not, so a line after one
  is not reached on every run: those five names are treated as ways out.
  C does not say which functions return, so this is a list by name.
  Alternative: treat every call into C as one that might not return,
  which would turn nearly every certain failure after a `write` into a
  note. This is a stopgap for the time before std: once `std::exit`
  exists it is the language's own way out, KAYO\n knows it ends the program
  without a list, and these names stop carrying any weight — not a
  permanent rule about C.
- **2026-09-23 — The exact answer is a 65-bit value.** Plain math is
  exact and checked where it lands (owner, `operators.md`); KAYO\n types
  every `+ - x xx` on integers as one internal type whose range is
  `int64`'s lowest to `uint64`'s highest (spelled `integer`), holds it
  in 65 bits, and computes `+ - x` in 128 so the only question in
  between is that window. Alternative: the narrowest type the prover proves
  (`operators.md` allows it, "especially for SIMD"); later, as an
  optimisation, since the answer is the same.
- **2026-09-23 — `:=` of an exact answer keeps the exact type, for now.**
  The width is decided (`precog.md`, "`:=` widths"): the prover picks the
  narrowest type holding every value the name can have, then adjusts for
  speed. KAYO\n does not choose it yet; it holds the exact type, which holds
  the same values, so only the representation differs.
- **2026-09-23 — Bitwise operators and shifts on an exact answer.**
  Wrapping on mixed sizes is decided (owner, b): an error until written
  `to.int16['a' + 'b'] +~ 1`, and KAYO\n refuses it. The bitwise operators
  take the one size an exact answer's typed operands share; mixed sizes,
  a `:=` name holding an exact answer, and any shift of one say not yet,
  since none of those was asked.
- **2026-09-24 — Induction on arguments covers at most 4096 tuples.** A
  recursive function is proven by induction on its arguments (owner:
  "teach the prover to prove bounds on recursive functions by induction"): its
  body is analysed once per argument tuple, from each parameter type's
  lowest value up to the highest the call can pass, and a recursive call
  reads the answers for the tuples before it. `INDUCTION_CAP` bounds how
  many tuples, since each is one analysis of the body. Past it, recursion
  is followed eight calls deep as before. Alternative: a larger cap (slower
  builds on wide ranges), or symbolic summaries (a formula in the argument
  rather than a table), which is a bigger piece of work.
- **2026-09-24 — `xx~` wraps at the base's size alone.** Its exponent is
  a count, like a shift's, not a value being wrapped, so a `uint32` base
  with an `int32` exponent is not mixing sizes. The oracle's reading
  (`operators.md`), not ruled. Alternative: the mixed-size rule for
  wrapping covers the exponent too, and `to.…` says which. (A negative
  exponent being a check site for `xx` and `xx~` was provisional here; the
  owner decided it, 2026-09-24: a.)
- **2026-09-24 — A `char.byte` as text is the code point of its number.**
  `to.str.utf8[…]` of byte 200 is two bytes of UTF-8 (U+00C8), as the
  oracle has it, while a point converts to a byte only below 128 and text
  of one byte must be ASCII. Nothing decides what a byte past 127 is as
  text. Alternatives: the raw byte (text that is not UTF-8), or refusing
  it (`not_utf8`).
- **2026-09-24 — A character whose unit is not written is a point.**
  `types.md` leaves the unit to the prover "from the content and its use";
  KAYO\n picks `char.point` (flat, 32 bits) when the content is one code
  point; a literal of several points can only be a `char.grapheme`, and
  is one. Not a semantic choice: the unit changes no value a program can
  see, only its size.
- **2026-09-24 — A function value is its code and a block of what it
  captured.** Two pointers: the code, and the captures — null for a named
  function or a literal that captured nothing, whose code is then the
  plain function (what C's callback slot takes). A literal that captured
  gets the block as its first parameter; a lend's block lives in the frame
  that made it (a lend cannot outlive what it borrows). A held call
  branches on the null. Nothing in `design/` says how a function value is
  laid out. Alternatives: always an env parameter (C callbacks then need a
  second entry per function); defunctionalising — every function value's
  possible targets are known to the prover, so a tag and a switch, no indirect
  call. The block begins with how to give it back and how to copy it
  (null for one that only lends): a function carrying a `copy`/`give`
  capture is owned (owner, 2026-09-24: a), and its block is on the heap.
- **2026-09-24 — A `:=` list lent across takes the parameter's widths.**
  `types.md` leaves a `:=` collection's width to the prover, from use; KAYO\n
  gives a list of bare numbers the width of the parameter it is lent,
  given, or unpacked into. (A lend whose written widths differ is a type
  error, owner 2026-09-24: a — "a lend hands over the value where it is, so
  the sizes must match".)
- **2026-09-24 — `kayo precog --json`: a line per check site, for the
  the prover oracle.** Check claude asked for KAYO\n's claims in a form a program
  reads (`precog.md`, attacks b and c). One JSON object per site: file,
  start and end (line, column) of its expression, the function instance,
  the kind of check and the construct, the type and range it must fit, the
  verdict (proven / checked / refused / nocheck), the layer that proved it
  (1, 2, 3, induction), and the value and operands as layers 1 and 2 saw
  them ("solver" for layer 3); integers as decimal strings. An interface
  between two sessions, not the language's JSON schema, which the brief
  leaves to the owner; its shape can change when that is decided.
- **2026-09-24 — Unicode 17.0.0, pinned.** Grapheme clusters are UAX #29
  (extended) through unicode-segmentation =1.13.3, whose tables are Unicode
  17.0.0 — earlier notes of this session said 16, wrongly. The owner ruled
  one Unicode version per build, recorded in the lock file beside the
  compiler version, moved on deliberately ("we should update when we should
  update"; reproducible-builds.md, item 7). No lock file exists yet
  (Synthia's); `kayo version` prints both, for it to carry.
- **2026-09-24 — `inline.ASM` says memory and the flags are written.**
  `ffi.md` has the bindings and the clobbers a program writes; KAYO\n adds
  `memory` and `cc` to every block, since a syscall may write through a
  pointer it was handed and the braces are not read. That costs LLVM some
  freedom around the block, never correctness. A binding uses the whole
  register (`xN` 64 bits, `wN` 32), the value widened by its own sign.
  Alternative: a `clobber memory` word the programmer writes.
- **2026-09-29 — `deci` is software BID, and every operation is a runtime
  call** (`runtime/src/deci.rs`, `compiler/src/codegen/deci.rs`): a value
  is its layout in an integer of its width. An answer past the largest
  value, a division or `mod` by zero, and zero to a negative power stop the
  program where they happen (exit 101, as a check does), as the oracle
  stops; they are not the prover sites, since the prover follows nothing about a
  decimal yet (every `deci` domain is "any", a range type's too).
  Alternative: sites the prover could prove, once it follows decimals.
- **2026-09-29 — `deci` `xx` is exact while the power has at most p + 40
  digits**, then carried at p + 40 digits, cut toward zero and marked
  inexact: only an answer within 10^-40 of a tie could round differently.
  The oracle stops instead ("past the oracle's limit of exact digits"), so
  `1.0000001 xx 10000000` is 2.718281692544966 here and a stop there.
  Alternative: an exact power always (slow and large), or a stop.
- **2026-09-29 — "A range is guaranteed" is decided for the whole program
  at once** (for the owner's option c: `nsw`, and a call's answer of a
  range type clamped to its range for LLVM). A range is handed to LLVM only
  when nothing in the program took one on anyone's word: no `nocheck`
  narrowing into a range type, no C door trusted under `NO_FUCKING_CHECKS`,
  no C global of a range type. One such site anywhere turns the clamps off
  everywhere. Alternative: follow each value to where its range was made,
  which is more work and hands LLVM more.
- **2026-09-29 — `kayo run <program>` builds as `kayo build` does (the working
  build, or `--final` / `--assert-proofs`), into a folder of its own under
  the system's temporary folder, runs it on this machine with the
  terminal's input and output, removes the folder, and exits with the
  program's exit (a signal as 128 + its number, as a shell has it). KAYO\n's
  own words while building go to stderr, so the program's stdout is only
  its own. The words after `--` are the program's `argv` (owner,
  2026-09-29), as the oracle takes them. `--input` and a cross target's
  runner wait for the owner.
  Alternative: keep the binary beside the source, as `kayo build` does.
- **2026-09-29 — A vector nothing gives a lane width has 64-bit lanes.**
  `let 'a' := simd.fill[5]` has no type to take its lanes from; KAYO\n gives it
  the natural width a lone number has (`core-features.md`), 64 bits, so
  two lanes in a 128-bit register. Before, it had no shape at all.
  Alternative: refuse it and ask for a type, as `simd.shuffle` does.
- **2026-09-29 — kits with dependencies: picks.** KAYO\n builds the
  default target (owner, 2026-09-29: b: `default = true`, or the only
  one), and writes code for `aarch64-macos` only so far, so any other
  default is refused as "not yet" rather than built for the wrong
  machine. The targets KAYO\n writes code for are listed in `kayo/targets.txt`,
  the one list (Synthia copies it, and the oracle reads it); any target
  not in it is refused (owner, 2026-09-30: "refuse"), and a default that
  is not this machine's is "not yet". A `CyJet.cyr` without a 'name', a 'prefix', a
  'version' or a target is refused (owner, 2026-09-29), a dependency's
  too; a folder with no `CyJet.cyr` is a plain program, and a dependency
  must have one. Its 'name' and 'prefix' both follow the module-name rule
  (owner, 2026-09-30: "strict"), with the oracle's reasons. A dependency is reached by its own 'prefix' and names no
  word; two different folders with one prefix, or a dependency whose
  prefix is `std` or `cca`, are refused, as the oracle does; the root
  kit's own prefix is checked against neither, as in the oracle. A
  dependency's own 'targets' are checked and not otherwise used. Using a
  runner is open. Two kits that depend on the same folder share one
  copy of it. A `version` dependency is refused until the registry
  exists, as the oracle does.
- **2026-09-30 — Parallel `for`: how KAYO\n builds it.** The rules are the
  owner's (`parallelism.md`); these are KAYO\n's picks within them, all
  provisional. A loop runs in slices when every iteration writes only
  what is its own — locals it makes, the element at its own position (the
  counter plus one constant, the same one for every read of that list),
  or a sum into one integer — and nothing leaves the loop early.
  - *Worth it*: decided as the loop starts, from its actual trips times
    the statements of one trip (an inner loop's weighed sixteen times, a
    call ten): at 64,000 or more, slices. Never from timing the machine;
    `tune.cplr` will override it once `kayo tune` exists.
  - *Sums and products* (`+` or `x`, one of them throughout, into an
    integer, a bin or a deci): each slice folds its own trips from 0 (or
    1), and the parent folds the partials into the variable in slice
    order, once. An integer fold whose every site is proven folds at the
    variable's own width, wrapping (arithmetic modulo 2^width lands on the
    true total, which fits); otherwise its partials are 128 bits, checked
    once at the end, and a product's also remember going past 128 bits and
    meeting a 0. A
    bin or deci partial is the variable's own type, so the rounding moves
    with the slicing (owner, 2026-09-30: c), and a bin total is always
    asked once whether it is still a real number. Every site on the fold
    must be proven, so the loop in order never stops — except an integer
    sum whose terms keep one sign, whose store may stay checked: the one
    check at the end stops exactly when the loop would have.
  - *In order*, silently (an error under `for para`): `perm` loops, a
    `break`, `return` or `fail` out of a trip, tasks, channels, `must`, a
    call through a held function, and any conflict. A loop that prints or
    calls C stays in order unless `para`.
  - *The pool*: threads made at the first sliced loop, one per core (the
    caller is one), at most 64 slices; a loop sliced inside another runs
    inline. `THREAD_CAP` in the environment caps the threads (owner,
    2026-10-15: a neutral name; `CYBORG_THREADS` is no longer read); the
    tests set 3, this session's share of the machine.
- **2026-09-30 — `--target <name>` names the target to build (ruled:
  owner, 2026-09-30, "yea").** A lone
  target saying `default = false` leaves the kit no default (owner,
  2026-09-30), and a compile (`kayo build`, `kayo run`, `kayo test`) with no
  target named is then refused; checking (`kayo check`, `kayo mir`, `kayo
  precog`) is not. A target is named `--target aarch64-macos`, the
  spelling cargo and clang use, once per command (proposed by KAYO\n, ruled
  by the owner). A named target must be one the kit's `CyJet.cyr` lists (a
  plain `.cy` file: one in `kayo/targets.txt`), and is built only on its
  own machine so far, as the default is.
- **2026-09-29 — `kayo prepare`: picks around the owner's rulings.** The
  rulings (the current folder's kit; names are kits' `name`s, its own or
  its dependencies', comma-separated with or without a space; a name two of
  them share is refused with their prefixes listed, and `--prefix geo2`
  picks one; a `build.cy` runs in its own kit's folder; `--all` runs
  dependencies first, each once, stopping at the first failure) are built
  as ruled. Also ruled (owner, 2026-09-30, "yea" to each, as KAYO\n had
  built them): named kits run in the order written, each once, whether
  asked for by name or by prefix; the names come before `--prefix`, and
  every word after it is a prefix, so `kayo prepare --prefix geo2 money`
  and a second `--prefix` are refused. Picks beside them, still
  provisional: a named kit with no `build.cy` is refused, while `--all` passes
  over those; and the failing one's exit is kayo's. Ruled (owner,
  2026-09-30): each `build.cy` is an ordinary program with no limits
  and no channel into the build, built as a program of its own with no
  language access to its kit's code — it reads a kit's files as any
  program would — and given no command-line words: its settings come as
  any program's do, from the environment or files; a kit whose prefix differs from its name
  is shown as `geometry (prefix `geo2`)`. `--prefix` takes a comma list
  and mixes with names (owner, 2026-09-30): `kayo prepare money --prefix
  geo2, geo3`.
  A folder is a word with a `/`, or `.` or `..` (owner, 2026-09-30), and
  comes first, as a program does in `kayo run` (owner: "yea"); a folder
  given later is refused. `kayo prepare` with no names, or a folder alone, is
  refused, pointing to names or `--all` (owner, 2026-09-30: "`a`").
- **2026-09-29 — How `kayo test` runs tests.** One test binary holds every
  `test fn` (and no `MAIN`); `kayo test` runs it once per test, the test's
  number its argument, so a check that fires ends that test only. A `must`
  that does not hold ends its test with exit 1 and `line N: …` on stderr.
  `must.stop` runs its expression in a forked copy of the process (POSIX
  `fork`) and the test carries on in the original. A fired check is
  reported in KAYO\n's own words (`the position can be past the end`), not
  the oracle's. A comparison shows its sides when both have a text form
  (numbers, `bool`, characters, text); other values say only that `must`
  does not hold. Alternative: `setjmp`/`longjmp` in one process.
- **2026-10-09 — The size report's shape** (the owner asked for "size with a
  breakdown"; the breakdown is design claude's reading: by kit, by
  function, and the copies each generic produced). Every `kayo build`
  and `kayo run` ends with it, after the sites line: the file's size on
  disk, the program's own code and data, and each function's code by
  kit, largest first, a generic's copy named with its types
  (`biggest[int32]`, "a copy of the generic `biggest` (2 in all)"). It is
  measured in the object file before linking, each function running from
  its symbol to the next; one LLVM inlined everywhere has no room of its
  own and is named on one line. The entry, `main`, is counted as the
  program's, since `MAIN` is often inlined into it. A blank a bare literal
  filled is named as built, 64 bits wide. Everything else on disk is the runtime,
  the runtime, and the linker's. Alternative: measure the linked file,
  which needs its symbol table kept.
- **2026-10-09 — Where the base's `optional` lives.** It is declared once,
  in a module of its own, and a bare `optional` falls back to it when the
  module has none, which no kit can have (`optional` is the base's). A
  type's own function (`fn Counter.next`) without `pub` is seen across its
  kit, as the oracle has it (not asked). A `for` over an iterator takes
  no `pos`, and plain `lend` is refused (`next` changes it).
- **2026-10-10 — A const array read in place is the program's own.**
  `incl.bytes` is a const's value only (owner, 2026-10-10: "(c)"), and a
  const array — it or one of literals — is laid out once in the program.
  Read where it stands (an element, `array.length`, lent to a call) it is
  viewed there, nothing copied, no heap, so with no OS too: a local set up
  as its body starts, never freed. Taken as a value (`let mut 'b' :=
  'logo'`) it is a fresh array, copied once, as "every use is a fresh
  value" says. Read inside a task it is still refused as a place (the
  task's captures are all it sees). `incl.str.*` is a text literal.
- **2026-10-10 — How KAYO\n reads a C header.** An `incl.C` line is the
  `extern.C` block KAYO\n writes (`compiler/src/cheader.rs`): the Clang
  beside the LLVM KAYO\n is built with (else `clang` on `PATH`) is run
  three times at once on the line's headers (Clang is CNC's, owner
  2026-10-10: borrowed so until the self-contained build, when it ships
  inside the CNC limb, fetched only by a build listing CNC) — its AST as JSON, its
  `#define`s, and its own `#define`s to leave out — and what comes through
  is written as CyborgPL and parsed at the line. What comes through follows
  the oracle's README ("What of a C header comes through"); where it says
  nothing, KAYO\n matches what the oracle does (found by running it): a
  `#define` number may carry C's `u`/`l` suffixes and be hex or octal, but
  not `+5`, `-(3)` or `0b101`; an empty `""` text comes through; a pointer
  is `ptr` when `const` stands next to its target (`const T *`, or `T
  *const *` for the outer one, so `execv`'s `char *const *` is
  `ptr[ptr.mut[C.char]]`), and a `const` after the last `*` leaves its
  function out; only globals C declares `extern` come
  through, `mut` unless C's own type is `const`; C's `...` is a last
  parameter `...'pN'`. Clang 22's own `__size_t` is `uint64`, and
  `__ptrdiff_t` and `__signed_size_t` are `int64`. The headers are read
  for the machine KAYO\n runs on, not the target. A C struct (owner,
  2026-10-10) is written as a `layout.C struct` when whole, and as
  `layout.C struct FILE;` when opaque, a form only KAYO\n's own text may
  use; an opaque struct is refused wherever it would be held rather than
  pointed at (a variable, a parameter, a field), not only when built or
  sized. A bit field, or a struct or union written inside a struct, makes
  it opaque. A struct with no fields is opaque.
- **2026-10-12 — C's types as KAYO\n reads them.** The `C.` types are
  the target's (owner, 2026-10-10): on `aarch64-macos` Apple's ABI (`char`
  and `wchar_t` signed, `long double` a `bin64`); on `aarch64-none` Arm's
  AAPCS64 (`char` and `wchar_t` unsigned, `long double` 128 bits, refused
  as not yet until `bin128` is built). Each is read as the CyborgPL type it
  is, as the oracle does. A header's type is read as C writes it, its type
  names followed until one CyborgPL keeps (`size_t` and its kind, `int8_t` …
  `uint64_t`) or a struct's or enum's: so `fread` takes a `C.size_t`, as
  its header says, where the oracle reads Clang's own built-in
  declaration of it (`C.unsigned_long`). A struct with a function pointer
  field comes through whole, as the oracle does; a union is opaque. A
  typedef of a pointer to a struct (`pthread_t`) is that pointer
  (`ptr.mut[_opaque_pthread_t]`), not a struct of its own name. C's `...`
  is a last parameter `...'args'`, as the oracle writes it. A C enum with
  two names for one number keeps the first as its case and makes the
  later a const (owner, 2026-10-12: "a"). A struct packed or aligned its own
  way (`#pragma pack`, `__attribute__((packed))`, `aligned`) is opaque, as
  the oracle has `log2phys`: `layout.C` lays out as C does by default.
- **2026-10-12 — What a `:=` whole number can be, for a `deci`.** The
  owner's (a): converting a whole number into a `deci` asks for
  `otherwise` only when it could have more digits than the `deci` holds,
  worked out from its operands. The checker walks the value: a literal is
  itself, a typed value its type's range, a `:=` local the range it was
  first given (a later `change` is not followed, as the oracle has it),
  and `plus`, `minus`, `times`, `div.X`, `mod.X` and negation what their
  operands allow. Where nothing bounds it, KAYO\n asks for `otherwise`,
  where the oracle takes it as fitting, so no run meets a failure the
  program has no way to handle.
- **2026-10-13 — How KAYO\n builds a collection.** A collection is the
  functions it is: its own function `fn cum`, each member `fn cum.double`
  (a function may own functions, as 2026-10-09's `fn cum.quick` says), and
  each preset a function calling the own one with its arguments filled
  and the rest passed as the own function takes them, a failure passed up
  as it comes. A collection may have no function of its own (owner,
  2026-10-13: "(a)"; decided, no longer a pick): its name only groups its
  members, so they are a module of its name, reached the same way
  (`zero.of_int64[]`, and no `zero[…]`), open to the kit, or to all
  under the mark when the collection is `pub`. A template is read as text
  (`compiler/src/template.rs`), as the oracle reads it, and its generated
  part is parsed as ordinary members and compared with what the template
  makes on every build; `kayo generate` writes it, byte for byte as
  `oracle --generate` does. `named` runs as a small function returning
  text: the file's own functions may be called there (`named
  size_name['n'];`, owner 2026-10-13), run on whole numbers, `bool` and
  text with `if`, `while`, `loop`, `let`, `change` and `return`, a million
  steps at most.
- **2026-10-14 — How KAYO\n reads the symbols and the argument words.**
  Each symbol is parsed as its word (`/` as `div.trunc`, `==` as
  `equal_to`, `&&` as `and.then`), so its level, its checks and the words
  in a message are the word's; `op +`, `op -`, `op *` declare `op plus`,
  `op minus`, `op times`, and the other symbols are refused in `op`. A
  written `==` is told from the word `equal_to`, which the lexer reads as
  the same token, by its length. `*const T` / `*mut T` are read as
  `ptr[T]` / `ptr.mut[T]`, `*'p'` as `ptr.read['p']` and `change *'p' = v;`
  as `ptr.write['p', value v]`. The argument words are checked on the
  tokens before parsing (`compiler/src/argwords.rs`, from `spellings.cyr`'s
  `calls`, as the oracle reads them): each listed call's words are taken
  out, so the checker sees the call as it always was; a position the table
  leaves "" (SIMD's `pos`, which was the call's own syntax already) stays.
  A call after `.`, or after a kit's `::` other than std's, is not
  checked.
  The cursed `+` and `*`, the `jokes` power and `KAYO_TODAY` are gone
  (owner, 2026-10-14).
- **2026-10-14 — How KAYO\n loads the CyborgPL std.** A dependency listed
  with std's ID and a `path` (no `version`) is the CyborgPL std: its prefix
  must be `std`, it is exempt from the "list std to use lists, maps and
  text" gate, and it is not a dependency by prefix but the program's one
  std (`Items::nerve_cord`), whichever kit listed it; a second one is
  refused. `std::x` (and a bare name the CyborgPL std marks `bare.optional`
  or `bare.only`) is looked up there first, as a `pub` item, and what it
  does not have falls back to the built-ins, `print` and `exit` (owner,
  2026-10-14, "(a)"), name by name. Inside the CyborgPL std its own names
  are its own. A program that lists no CyborgPL std is unchanged.
- **2026-10-14 — Layer 3 assumes the landings on the way, and knows a
  counting loop's start.** Reaching a site means every check before it
  passed, since one that fails stops the program and a proven one cannot
  fail (by induction on the first site to fail): so a query assumes each
  narrowing site walked before the asked one passed, unless it is trusted
  (`no_check`). Only narrowings, and not the one that steps a counting
  loop's counter (which lands where the loop just saw room): the others
  add weight to every unrolled trip and turned proofs into spent budgets.
  At a counting loop's last unrolled head, its counter is at least its
  first value going up (at most, going down), when the loop writes
  neither that value nor the step (`ForLoop::start`). Both from std's
  `str.cy` (std claude): `'pos' plus 'm'` wrapped round to 0 on an
  unrolled trip and then indexed, and `for 'k' from 'pos'` was given a
  `'k'` below `'pos'`.
- **2026-10-14 — A module may be named after a built-in type's word.**
  `str.cy` (std's), `int32`, `bool`, `char`: only the table's keywords and
  the old words kept inside (`kit`, `nocheck`, `var`, `func`) are refused,
  as the oracle has it.
- **2026-10-14 — `str.empty`.** The parser reads the path `str.empty` as
  the empty text literal, so it goes wherever `""` went, `spellings.cyr`'s
  own lists included; a written `""` (two quotes) is refused from the
  table's special row. A text of at least 1 byte for `part`, `at` and
  `from` is std's to say, in its types: KAYO\n builds no std text
  function, and `str_split_empty` waits on the std session (`!pending`).
- **2026-10-14 — Commas in braces, and `level N { … } left_first`.** Enum
  cases, struct fields (`layout.C` ones too) and a level's operators are
  separated by `,`, one after the last allowed (`item_comma`); `incl.C`
  writes its `layout.C` declarations with `, ` between items, as KAYO\n
  reads them back. A level is `level 1 { plus, minus } left_first`, the old
  `level.when_chained.…` refused from the table's special row.
- **2026-10-14 — A length bound is a `LengthBound` site wherever a value
  lands.** Handing a value to a length-bounded parameter, field or return
  was a `Narrow` site, which checks numbers only, so an empty text reached
  a `str.utf8[min 1 bytes]` unchecked (std claude); it is a
  `LengthBound` now, and a lend into a bounded parameter is a site too.
- **2026-10-14 — Examples against the CyborgPL std.** An example listed in
  `tests/examples_cyborgpl_std.txt` is also built in a temporary kit that
  lists `std/` by std's ID and its path, and must print the oracle's
  `.out` (`examples_run_against_the_cyborgpl_std`); built as a lone file,
  with the built-in std, it stays on the not-yet-built list.
- **2026-10-14 — A final build's debug info beside it, and the no-OS
  image's start.** `--final` runs LLVM's `dsymutil` (macOS, `out.dSYM`) or
  `llvm-objcopy --only-keep-debug` and `--strip-debug
  --add-gnu-debuglink` (Linux and no OS, `out.debug`) after linking, before
  the size is measured; a working build keeps the info inside. With no OS
  the vector table opens `.text._start`, 2 KiB aligned, and `_start` sits
  in its first slot (taken only by an exception on `SP_EL0`, which the
  program never runs on): no padding before the table. A const array read
  in place (`incl.bytes`) has its length in the prover's domain.
- **2026-10-15 — No proof leans on an assumption that cannot hold.**
  A whole-number constant is read at a width that holds it (2^63 was read
  at 64 bits as -2^63, so a landing assumed passed was false and every
  later site of the function was "proven"; check claude, corpus
  `f4d99bd1`). And a layer 3 `unsat` that assumed the landings before its
  site counts only if the site can still be reached with them (`reach`):
  if not, the site is asked again without them.
- **2026-10-15 — A widened loop head's entries are a choice of their own.**
  When a loop's last unrolled head copy stands for the later trips
  (widened, because an inner loop used the levels up), each entry into
  the loop also arrives there; it did on the entry's own condition, so the
  walked trip and the later ones held together, the join took the walked
  trip's values, and the later trips were never asked (`'i'` of -2 on an
  outer loop's third trip "proven"; check claude, corpus `71a8d5a2`). The
  entry is now split by a fresh selector, one way or the other.
- **2026-10-15 — `--emit-llvm-tv <dir>`, for translation validation.**
  Beside a build that is otherwise the same, `dir/tv-before.ll` and
  `dir/tv-after.ll` hold a copy of the module before and after KAYO\n's own
  passes (`PASSES`, the same machine, the prover's `nsw`/`nuw` and facts),
  with every function that has a body (but `main`) marked `noinline` and
  given weak linkage, so no pass deletes a function, drops an argument,
  carries a constant in or replaces a call by what the body returns: a
  weak body is not the one that must run (check claude: external still let
  `ret ptr %0` replace its callers' calls). Alive2 (check
  claude's `check tv`) can then compare them function by function (owner,
  2026-10-15: "(a)"). The functions keep their signatures, and the
  attributes the passes infer on a function (`nofree`, `noundef`, …) are
  taken off again, since they are about calls and Alive2 cannot read all
  of them; lifetime markers are check claude's to strip, on both sides.
- **2026-10-15 — `concat`, and `'i'++;`, `'i' += e;`.** `concat` is the
  word for joining text, at `++`'s level, and messages name it. A name at
  the start of a statement followed by `++` / `--` and `;`, or by `+=`,
  `-=`, `*=`, `/=`, `%=`, `**=` and a value, is read as the long `change`
  (`change 'i' = 'i' plus (e);`, the value bracketed); nothing else
  changes in the checker. A field is not one (`'p'.x += 2` is refused as
  before).
- **2026-10-15 — A counting loop by a step the program computes comes
  round into its body.** The step is taken only when there is room for it
  before the stop, so the head's test — is the counter within the stop —
  is already answered on the way round; the back edge skips it, leaving
  one test a trip (bench/sieve's inner `by 'p'` loop, 5% faster). Not for
  a step written as a number, where LLVM merges the two tests itself and
  did worse with the room's alone (bench/lists 3% slower); not in a
  slice whose room is measured to a stop folded to a constant, the whole
  loop's rather than the slice's. Whether a loop runs in slices is asked
  with a multiply (`span ≥ k × step`), not a 128-bit division on every
  entry. The idea to look was design claude's (`llvm.loop.unroll.disable`;
  KAYO\n's loop was never unrolled, and that changed nothing).
- **2026-10-15 — `kayo version` says `CyDT 1 KAYO\n`** (owner: KAYO\n is
  named by its kit, distribution.md), followed by "a build before CyDT 1's
  release": what such a build calls itself is not ruled yet.
- **2026-10-16 — Symbols only where they read correctly** (owner, DECISIONS
  "Symbols only where they read correctly"). `+`, `-`, `x` (`times`), `xx`
  (std's `pow`), `<`, `>` and a leading `-` are the good-practice spellings;
  `less_than`, `greater_than` and `negate` are new words (`neg` refused).
  Every other symbol is refused before parsing, naming its word from
  spellings.cyr's row: `== != <= >= / % && || ** ++`, the compound changes
  (gone), and `*` and `&` by what stands around them, as the oracle reads
  them (after a value `*` is `x` and `&` is `and`; `*19.99`, `*const`,
  `*mut`, `*'p'`, `change *'p' =`, `&mut`, `&'x'`). `op x` declares
  `times`; `op xx` is refused, `pow` being std's.
- **2026-10-16 — Percentages** (owner, DECISIONS "Percentages"). The lexer
  reads `80%` and `80 percent` as the bin `0.80`: the number's digits with
  the point two places left, so `exactly 80%` is the deci. The `%` touches
  the number and is the old `%` of `mod` when an operator character, a
  digit, `'` or `(` follows (`7%2`); `percent` comes after a space and
  runs into no letter.
- **2026-10-17 — Recursion is inlined into itself** (owner, DECISIONS
  "Beating GCC on recursion": "Oh hell yeah, we're doing 1 and 2"; C++
  allowed for kayo's own LLVM passes). `llvm-passes/` holds them, C++20
  with `-Wall -Wextra -Werror`, one static library called through
  `extern "C"`: `pipeline.cpp` builds `default<O3>` as `LLVMRunPasses`
  does, and `recursive_inline.cpp` runs after the early simplification,
  ahead of the inliner. A function that calls itself has its body copied
  to a snapshot; each self-call is pointed at the snapshot and inlined,
  `RECURSION_DEPTH` (4) levels while the function stays under
  `RECURSION_SIZE` (1000) instructions; `noinline` functions (what
  `--emit-llvm-tv` marks) and mutual recursion are left alone.
  `KAYO_RECURSION_DEPTH=0` turns it off to compare. LLVM then also folds the
  repeated pure calls the levels hold, so bench fib(35) went from 0.0150 s
  to 0.0020 s (GCC 16 -O3: 0.0085 s). `tests/llvm_passes.rs` runs the pass
  under AddressSanitizer and UBSan on hand-written IR.
- **2026-10-17 — Typed literals, `5: uint8`** (owner: "yea", "(a)": no
  brackets). The parser reads a `: T` right after a literal (a number,
  `-128`, `exactly 0.1`) as a `Typed` node, and after a name too, for the
  checker to refuse naming `to.T['x']`. The checker takes only literals,
  of a number type, that fit the type and its limits, with the oracle's
  sentences: `300: uint8` does not fit, `0: uint64[min 1]` is outside the
  limits, `1.5: int32` is not whole, `0.1: deci64` points at `exactly`.
- **2026-10-17 — A file with no CyJet.cyr gets std** (owner, 2026-10-06:
  what the machine's package installed; with the std package, std). Until
  an installer writes where it is, the repository's `std/` stands in
  (`KAYO_STD` names another), loaded as a manifest's std dependency would
  be; what it has not written yet says "not available yet", and the
  built-in print and exit stay where std has none. std's functions no start
  of the program reaches are emptied after lowering (`drop_unreached`), so
  nothing of them is proven or generated. std's own `str.at.X` and
  `str.slice.X` ask each position to be 1 or more before their parameter's
  bound, so a computed 0 says "0 is no position".
- **2026-10-18 — `no_cheat` in front of any statement or block, calls
  included** (owner: "(b)"; and "(a)": `no_cheat for`, `for no_cheat`
  refused from its row). The blocks it writes are marked, its loops are
  `no_cheat` loops, and where it starts the locals it touches pass through
  the barrier; every function it calls, and everything those call, is a
  `no_cheat` copy (`nocheat_clones`) whose parameters pass through it at
  entry, so its work is neither folded, dropped, nor shared with a
  look-alike call, while inlining stays. Precog folds nothing in any of
  them. Beside `check` / `no_check` it is refused, the order not decided.
- **2026-10-19 — A `list[bool]` held as bits** (owner, 2026-10-18: "Pack
  a `list[bool]` into bits"). `mir::bits` finds each owned `list[bool]` a
  function makes with `list.fill` and then only reads and writes by
  position, asks the length of, and drops; anything else (a call, a lend, a
  copy, an append, a walk, a capture, a return) keeps it a byte an element.
  Such a list is a byte for every eight elements (`codegen/bits.rs`), its
  length and capacity still counted in elements; a write sets or clears its
  own bit, and inside a parallel `for`'s slices, where the slice beside it
  may hold the same byte, it is one atomic `or` or `and` (two for a value
  not known as written) and a read is an atomic load. `KAYO_BITS=0` turns it
  off to compare. bench sieve to 200,000,000: 192 MB to 25 MB, 0.49 s to
  0.35 s on 3 threads, 0.77 s to 0.62 s on 1.
- **2026-10-19 — `no_cheat` covers everything the statement runs** (owner:
  "(a)"). A task started inside runs a `no_cheat` copy of its body. A call
  through a held function cannot be pointed at a copy before it runs, so
  once `no_cheat` code makes one, every function the program holds (`fn`
  by name, or a literal) gets a copy, and the generated code compares the
  held code with each and calls the copy (`nocheat_held`); where LLVM knows
  which function is held, the comparisons fold to a direct call. A copy is
  kept wherever its function is reached.
- **2026-10-19 — The check word comes before `no_cheat`** (owner: "(b)",
  then "(a)"): `check no_cheat …`, `no_check no_cheat …`,
  `NO_FUCKING_CHECKS no_cheat …`. `no_cheat` in front of a check word is
  refused with the order to write.
- **2026-10-19 — `table_bytes`, read** (owner: "(c)", "(a)", "(a)"):
  `cyjet::target[…, table_bytes = N]` per target and `table_bytes N fn …`
  in front of a function (after `pub` and `bare`), each a whole number of
  bytes; anything else is refused as the oracle refuses it. Kept for the
  tables, which come next, and listed in the build report. A no-OS target
  listed without a `layout` is now refused when it is built, not when it is
  listed (`examples/table_limits` lists `aarch64-none` beside the Mac).
- **2026-10-19 — The build report** (owner; `design/tooling.md` § The build
  report). Every build writes `<program>.report.md` beside the program
  (`src/report.rs`): a contents line, then Summary, Inputs, Proofs, Your
  markers, Types Precog picked, Optimizations, Removed, C, Sizes, Timing,
  and Warnings and notes, each a `##` heading. The terminal keeps errors,
  Precog's notes and one line: `built build/fib (final build,
  aarch64-macos): 22 sites: 22 proven, 0 checked at run time, 461,392
  bytes, in 14.71 s; report build/fib.report.md`. What it reads: every
  file and kit loaded (`Items::report_kits`), every `:=` (`Function::picked`)
  with every value Precog saw it take (`Analysis::held`, joined at each
  store), the struct layouts and const sizes the code generator worked out
  (`Lowered`), the recursive inliner's count and the functions LLVM left
  vector code in (`ObjectFacts`), each `incl.C` line (`cheader::read_lines`),
  and each phase's time. Precog's tables are not built yet, so the report
  says none were made.
- **2026-10-19 — Precog's tables** (owner, 2026-10-17; limits
  2026-10-19, `design/precog.md` § Tables; `precog/tables.rs`). A function
  whose parameters are whole numbers with ranges, which returns a whole
  number or a `bool` and fails with no reason, and which with everything it
  calls does nothing but compute on plain values (no C, no held function,
  no task, no assembly, no list, text or other memory), gets every answer
  worked out while building: Precog runs it on each argument tuple, dry, on
  the graph as written (before any fold: `square` called only with 200 was
  already folded to 40000, and every entry read 40000). A table is kept when
  every answer is one exact value, it fits its limit (`table_bytes N fn`,
  else the target's in `CyJet.cyr`, else 65,536; 0 is never), and every
  check in the function and in what it calls is proven. The code generator
  writes the function as a lookup in a private constant array. Never a
  `no_cheat` copy, never with `--assert-proofs`; `KAYO_TABLES=0` turns it
  off to compare. bench fib is a 93-entry table of 744 bytes, made in 7.8 ms
  of Precog. The report lists each table under Optimizations and Sizes, and
  the near misses with why.
- **2026-10-19 — A slice ran to the loop's end** (check claude: 8000 ones
  summed to 16001 after a long write loop). Precog knew the list's length,
  folded the sum loop's read of its stop to it, and the step round in each
  parallel slice compared with that constant instead of the slice's own
  stop. Inside a counted loop, an instruction reading its stop, counter or
  step is no longer folded (`tests/programs/par_sum_after_write.cy`).
- **2026-10-19 — A simd's lane count says `count`** (owner):
  `simd[int32, count 4]`, as `simd.load.some[…, count 3]` says it; the bare
  `simd[int32, 4]` is refused from `spellings.cyr`'s row, and types print
  with the word. kayo's own simd test programs are rewritten.
- **2026-10-19 — `simd.fill` into a declared vector** (owner: "yea" to
  (a)): the declared lanes win, and the value is fitted into one as a
  typed `let` fits it, so `simd.fill[value 5: uint8]` into a
  `simd[int32, count 4]` is four `int32` lanes of 5 (`expect_store`); a value
  that does not fit, `-1` into `uint32` lanes, is refused as a `let` would be.
- **2026-10-19 — A dependency says its kind** (owner: "(a)", "dep.reg"):
  `cyjet::dep.path["../kits/areas"]`, one text, and
  `cyjet::dep.reg[id = …, version = …]`. The old `cyjet::dep[…]` is refused
  from `spellings.cyr`'s two rows, naming whichever its fields meant; a
  field on `dep.path`, or a `path` on `dep.reg`, is refused as the oracle
  refuses it. std by its ID and the folder it is written in,
  `cyjet::dep[id = <std's>, path = "…"]`, has no new spelling yet (asked):
  it is still taken as it was.
- **2026-10-19 — std from its folder is `cyjet::dep.path.std["…"]`**
  (owner: "(b)", then "Shouldn't it be `cyjet::dep.path.std`?"): one text,
  the folder, taking over `std::` as before; the old
  `cyjet::dep[id = <std's>, path = …]` and the day-old `cyjet::dep.std` are
  each refused from their own row.
- **2026-10-19 — `coll.fn`, `coll.handler`, full member names, members
  written elsewhere** (owner). `collection.` is refused from its
  `spellings.cyr` row, a bare `collection c {` too. Every member line writes
  the collection's name in front, in a `coll.fn` (`area.ring […]`, presets
  `area.unit ['w' = 1];`) and in a `coll.handler` (`h.as_zero { … }`); a short
  name is refused naming the full one, and `kayo generate` writes templates'
  members with it. A member may point at a function written elsewhere,
  `area.square = shapes.square.area;` (`ItemKind::FnPointer`): the resolver
  reads the path from where the collection is written (the module around it,
  for a collection with no own function), and makes the member a copy of
  that function under its own name and visibility, once every function is
  resolved, a pointer to a pointer a round later. The own function pointing
  elsewhere is refused. A `pub` collection's one `bare.` mark is written in its
  block, `pub coll.fn area { bare.never; … }`; in front it is refused.
- **2026-10-19 — Layer 3 asks three sites at once** (std claude: a
  30-line argv program took 78 s to build). The time was CVC5: three
  questions of about 120 KB took 26, 18 and 8 s one after another. Each
  site's questions (`precog::layer3`) are now asked on one of three threads,
  each running one solver at a time, and the answers are applied in site
  order, so the build says the same whichever finished first: 72 s to 34 s,
  the same verdicts. `KAYO_SMT_TIMES=1` prints each question's size, time
  and answer. Two report slips from the same program: a program reading
  memory through `ptr.read`, or taking C's `argc`/`argv`, was said to read
  nothing from outside; and a length bound's counterexample showed the
  largest length (`9223372036854775807 bytes`) where the empty text was the
  one out of range: it now shows the end that is out.
