# Synapse — the Nerve compiler

The compiler for Nerve, the language once called CyborgPL, built by the
compiler session ("compiler claude") from `design/` and the oracle's tests,
never from `oracle/src` (`design/compiler-session.md`).

**The name** (owner, 2026-10-09: "Yea"): CCA, the Cyborg Chassis Assembler,
is **Synapse**, where Nerve's parts meet: the Brainstem (the language's lowest
level, `compiler/`), the lobes (`lobes/`), Cortex (the prover,
`compiler/src/cortex`) and Axon (the runtime, `runtime/`). The command is
`synapse`, and `syn` is the same command (owner: "Both"), and so is `sng`,
for SNG, "Synapse Not GCC" (owner, 2026-10-10: "work too"); the folder and the
crates take the full name (`synapse-brainstem`, `synapse-runtime`,
`synapse-lobe-*`; "Yeah"). The old `cca` command is refused, naming
`synapse`, as any old spelling is.

```
cd synapse
cargo build
./target/debug/synapse parse ../examples/hello.nrv     # parse only; the first refusal, in the diagnostics.md shape
./target/debug/syn check ../examples/add.nrv           # parse and check (`syn` is `synapse`)
./target/debug/synapse mir   ../examples/add.nrv       # check, then print the flow graph
./target/debug/synapse cortex ../examples/scores.nrv   # …then prove, and print the report
```

## Layout

Split as `PLUGINS.md` lays out (step 1, 2026-10-07; the owner's answers
are in it):

- `compiler/` — the `signatar` crate, Brainstem, the lowest level of the
  language since there is no base and no codextras (owner, 2026-10-07),
  still holding the whole language and the `synapse` binary: `source` (files,
  spans), `diag` (the labelled message shape), `lexer`, `ast`, `parser`,
  `resolve` (items and modules), `check` (names, types, verbs, failures)
  producing `hir`, `mir` (the flow graph, its domains, its verifier and
  its printer), `precog` and `codegen`. Its folder keeps its name, since
  other sessions read `synapse/compiler/tests`.
- `base/` — `synapse-base`, Brainstem's power ledger (step 2), kept a crate
  of its own; "the base" it was named for is Brainstem's since 2026-10-07.
- `lobes/<name>/` — one crate per official lobe (a core until 2026-10-09; `synapse-lobe-ieg`, …),
  each its name, registry ID and version for now; each lobe's features
  move in at its step.
- `runtime/` — `synapse-runtime`, **Axon** (the runtime's name, owner
  2026-10-09; the crate keeps its name until the code-name step), the
  `no_std` Rust library linked into every
  CyborgPL binary.

./target/debug/synapse build ../examples/… -o prog       # …then generate, link, and run
```

The build needs LLVM 22 and lld 22 where `synapse/.cargo/config.toml` says they
are (Homebrew's `llvm@22` and `lld@22` for now), a C++ compiler for the lld
shim (`compiler/lld/shim.cpp`), and Axon, the runtime, linked into every binary:
`cargo build -p synapse-runtime`. Synapse links with lld through its library
(owner, 2026-10-08: "We'll use `lld`", then "`a`", Zig's way), so LLVM is
linked as one shared library, the one lld's libraries use; a self-contained
release will need LLVM and lld built as static libraries. Linking is built
for macOS hosts so far (the SDK found by `xcrun`).

**Of cord, `cord::print` and `cord::exit`** (owner, 2026-10-09: "`b`, not
the entire fucking neuron"; "No `std::` for now" stands for the rest).
`print.useful` and `print.stored`, each to `.stdout` (the default) or
`.stderr`, `print.useful.auto_newln` and the two `.weird` forms build: each
value becomes its text as `to.str.utf8` makes it (`compiler/src/codegen/
print.rs`), and the pieces go out in one `writev`; `cord::exit[code]` is the
OS's `exit`, and as an `otherwise` it leaves. Every other print
(`print.format`, `format[…]`) is refused by name, and the rest of cord as
before. Every example is built and run against the oracle's `.out` and
`.exit` (`examples_run_as_the_oracle_prints`); the ones that cannot build
yet are in `tests/examples_unbuilt.txt` with their refusal's words.

**A chip with no OS: `aarch64-none`** (owner, 2026-10-09). Built from any
host, run on the bare chip (tested in QEMU's `virt` machine,
`compiler/tests/bare/`). The Neuron names its own linker script on the
target, its path from the Neuron's folder: `kin::Target[name =
"aarch64-none", layout = "boards/virt.ld"]` ("`c` now", then "`b`"); a lone
file is not built for it. The script keeps `_start` first (`ENTRY(_start)`,
"`a`") and defines `__stack_top`, `__bss_start` and `__bss_end` ("Whatever's
accurate"), each refused by name when missing. Synapse writes `_start`:
only core 0 goes on, the FP/SIMD unit is turned on, a vector table whose
entries wait forever is installed, the stack is set and the zeroed memory
cleared, then MAIN runs; when it ends the CPU sleeps forever (`wfe`, "`a`").
A failed run-time check is `brk #1` (2026-10-08: "yea, freeze there"): the
CPU stops in the table, `ELR_EL1` at the trap and `ESR_EL1` saying it was
one. `memcpy`, `memset`, `memmove` and `memcmp` are Synapse's own, each kept
only if used (`--gc-sections`). Axon is not built for it, so a program
needing it, or C, is refused before linking, naming the function.

```
cargo test        # every examples/*.nrv checks; every reject program is
                  # refused; Synapse's own programs build, run and exit right
```

## Status

- 2026-09-22: the whole pipeline runs — lexer, parser, item resolution,
  the checker, HIR, MIR, the ownership walk, Cortex (layers 1, 2 and 3)
  and LLVM, out to an object file and a linked binary. The heap is in:
  lists, text built at run time, maps, enums and `match`, and failures as
  values, all of them freeing what they take. Every
  `examples/*.nrv` loads, checks and lowers to a well-formed graph, and
  none is refused by Cortex. Of the 358 reject programs, 344 are refused
  in Synapse's own words (`compiler/tests/reject_phrases.txt`); 9 are marked
  `!pending` and 5 `!by-design`. The test insists a `!pending` one is
  still *accepted*, so the file has to be updated the day each starts
  being refused.
- Over the examples, Cortex settles 398 of 422 sites: layer 1 folds what
  it can, layer 2 proves most of the rest, and layer 3 is asked what is
  left. 21 sites keep a check at run time and 3 are `nocheck`.
- 2026-09-24: every base-language example builds; what has no code yet
  is `std::` (none for now, owner). Synapse refuses to write machine code it
  has no shape for rather than writing something wrong.
- 2026-09-27: the 2026-09-25 syntax (no `let`, `change`, `!$`,
  `and.then` / `or.else`); `layout.C enum` and ranged `layout.C` fields,
  checked at the door when a call returns them. A `layout.C` struct or C
  enum crosses to C as C passes it (`codegen/cabi.rs`, AAPCS64: packed
  registers, float HFAs, `sret` over 16 bytes); another machine's
  convention is not written, and Synapse refuses there. C's numbers are
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

- `synapse build --assert-proofs file.nrv -o out` (2026-09-23): the build that
  checks Cortex (`precog.md`, attack (d)). Every site Cortex proved keeps
  a run-time check, nothing proven is promised to LLVM, and layer 1's
  folded answers are computed at run time instead. A wrong proof prints
  `proof failed: <the site's sentence>` and `where: file, line N, column
  M`, and exits **104** (an ordinary check that fires exits 101). `nocheck`
  sites proved nothing and are not asserted. The verification session's
  harness runs every generated program this way.

## Provisional — picks made to keep moving, not decisions

Dated, with the alternative, so each can become a question later.

- **2026-09-22 — `synapse/` is a Cargo workspace** (`compiler/` + `runtime/`),
  not one crate as the brief's wording has it: the runtime must be
  `no_std` and `staticlib`, which a second crate is the only way to say.
  Alternative: one crate with the runtime as a module and a build script.
- **2026-09-22 — `;` after a `concur` or `whichever` statement is
  optional.** The examples write it both ways (`whichever.nrv`); the
  oracle accepts both; every other statement needs its `;` and there are
  no empty statements. Alternative: require it always (would break the
  examples) or never (would break them too).
- **2026-09-22 — The target's SIMD shapes are ARM64's 128 bits, in the
  compiler.** `simd.md` says the shapes come from the target description
  in `Synthia.cplm`, and Synthia does not exist yet, so
  `types::TARGET_SIMD_BITS` holds 128 — the oracle's fixed target and this
  machine's. It moves into the target description the day there is one.
  Alternative: refuse every `simd` until Synthia exists, which would leave
  `examples/simd.nrv` unbuildable.
- **2026-09-22 — a `list` is 24 bytes even where Cortex could prove it
  smaller.** `lists.md` says a sequence nothing appends to becomes an
  `array`, and a list whose final size is proven needs no capacity field.
  Neither is implemented, so a `SeqKind::Free` is lowered as the shape that
  can do both. Sound, and larger than it needs to be. The same is true of
  `map`: `MapOrder::Free` is lowered as the ordered shape. Alternative:
  refuse to lower until Cortex settles the variant, which would block
  programs for a size question.
- **2026-09-22 — `synapse/.cargo/config.toml` points at Homebrew's LLVM 22.**
  The compiler's own dependency has to be found somewhere, and Synthia does
  not exist to say where. This is not the *target* being sniffed from the
  host (`tooling.md` forbids that, and nothing here reads it): it is where
  the build gets LLVM, as Kespar's config points at Z3. Alternative: an
  environment variable every build has to set.
- **2026-09-22 — A `lent` parameter crosses as a pointer.** A borrow is
  temporary access without ownership, and an address is what that is on
  the machine. `ownership-syntax.md` says what a lend *means* and not what
  it is made of, so this is a pick. Alternative: pass small lent values by
  copy and rely on Cortex to prove nothing observes the difference — which
  is true today and stops being true the moment a `lend.mut` writes back.
- **2026-09-22 — A drop whose value is gone on *some* paths asks a flag.**
  MIR emits a drop per owned local at its scope's end; the ownership walk
  removes the ones definitely already given away (`SYNAPSE_DROPS=1` counts
  both). Where a value is gone on some paths only, each owning local has an
  "alive" flag at run time — set when it gets a value, cleared when it is
  given away — and the drop tests it. Alternative: split the block so each
  path drops what it owns, which trades a flag for more blocks.
- **2026-09-22 — A testing-only `std::` stub table.** `std::` does not
  exist (owner, 2026-09-22), but 69 of the 74 examples use it. The
  checker has a flag (off by default, on in `cargo test` and with
  `SYNAPSE_CORD_STUBS=1`) that gives the `std::` names the oracle emulates
  their types and nothing else — no code is generated for them. It exists
  so the front end can be tested against the examples today, and goes away
  when std is written. Alternative: test only the six extern-based
  examples until std exists.
- **2026-09-22 — Reject-test matching.** Synapse's messages are in Synapse's
  words, so the `.err` substring is not matched literally:
  `compiler/tests/reject_phrases.txt` records, per reject test, a phrase
  of Synapse's own message, reviewed against the `.err`, and the test fails
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
- **2026-09-22 — 128-bit whole numbers are refused.** Every number Cortex
  reasons about is held in `i128`, so a `uint128` has values it cannot
  write down and an `int128` has sums it cannot; a proof about a number
  the prover cannot represent would not be a proof. `int128`/`uint128`
  are *proposed* in `design/types.md`, not decided, so Synapse says so in a
  message instead of carrying them badly (it used to panic). Alternative:
  a wider value model throughout Cortex's domains, which is real work for
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
  decided (c): Cortex knows which memory a text is in and a free is
  emitted only where it can be the heap, with the sentinel — the high bit
  of the length — as the fallback for a name that may hold either. Cortex
  does not carry that fact yet, so Synapse pays the fallback everywhere: a
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
- **2026-09-23 — Channels: the counting, none of Cortex's three parts yet.**
  The layout is the owner's (one block, slots inline, never growing; the
  program holds a pointer to it) and so is `closed` (count the tasks that
  can reach it; a `take` on an empty channel fails `closed` when every
  holder is itself waiting to take from it). What is built is the plain
  form: every channel on the heap, every slot the type allows, and the
  counting everywhere. Of Cortex's half of `closed`, one part is built
  (2026-09-23, `mir/channels.rs`): the task that made a channel lets go
  of it where it provably never names it again — directly or through a
  borrow held in another name — rather than at the end of its block, so
  MAIN lending one channel to a producer and a consumer no longer keeps
  the consumer's last `take` waiting for ever (the oracle says `closed`,
  and so does Synapse now). A channel ever given away, or a name assigned
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
  exists it is the language's own way out, Synapse knows it ends the program
  without a list, and these names stop carrying any weight — not a
  permanent rule about C.
- **2026-09-23 — The exact answer is a 65-bit value.** Plain math is
  exact and checked where it lands (owner, `operators.md`); Synapse types
  every `+ - x xx` on integers as one internal type whose range is
  `int64`'s lowest to `uint64`'s highest (spelled `integer`), holds it
  in 65 bits, and computes `+ - x` in 128 so the only question in
  between is that window. Alternative: the narrowest type Cortex proves
  (`operators.md` allows it, "especially for SIMD"); later, as an
  optimisation, since the answer is the same.
- **2026-09-23 — `:=` of an exact answer keeps the exact type, for now.**
  The width is decided (`precog.md`, "`:=` widths"): Cortex picks the
  narrowest type holding every value the name can have, then adjusts for
  speed. Synapse does not choose it yet; it holds the exact type, which holds
  the same values, so only the representation differs.
- **2026-09-23 — Bitwise operators and shifts on an exact answer.**
  Wrapping on mixed sizes is decided (owner, b): an error until written
  `to.int16['a' + 'b'] +~ 1`, and Synapse refuses it. The bitwise operators
  take the one size an exact answer's typed operands share; mixed sizes,
  a `:=` name holding an exact answer, and any shift of one say not yet,
  since none of those was asked.
- **2026-09-24 — Induction on arguments covers at most 4096 tuples.** A
  recursive function is proven by induction on its arguments (owner:
  "teach Cortex to prove bounds on recursive functions by induction"): its
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
  `types.md` leaves the unit to Cortex "from the content and its use";
  Synapse picks `char.point` (flat, 32 bits) when the content is one code
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
  possible targets are known to Cortex, so a tag and a switch, no indirect
  call. The block begins with how to give it back and how to copy it
  (null for one that only lends): a function carrying a `copy`/`give`
  capture is owned (owner, 2026-09-24: a), and its block is on the heap.
- **2026-09-24 — A `:=` list lent across takes the parameter's widths.**
  `types.md` leaves a `:=` collection's width to Cortex, from use; Synapse
  gives a list of bare numbers the width of the parameter it is lent,
  given, or unpacked into. (A lend whose written widths differ is a type
  error, owner 2026-09-24: a — "a lend hands over the value where it is, so
  the sizes must match".)
- **2026-09-24 — `synapse precog --json`: a line per check site, for the
  Cortex oracle.** Check claude asked for Synapse's claims in a form a program
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
  (Synthia's); `synapse version` prints both, for it to carry.
- **2026-09-24 — `inline.ASM` says memory and the flags are written.**
  `ffi.md` has the bindings and the clobbers a program writes; Synapse adds
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
  stops; they are not Cortex sites, since Cortex follows nothing about a
  decimal yet (every `deci` domain is "any", a range type's too).
  Alternative: sites Cortex could prove, once it follows decimals.
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
- **2026-09-29 — `synapse run <program>` builds as `synapse build` does (the working
  build, or `--final` / `--assert-proofs`), into a folder of its own under
  the system's temporary folder, runs it on this machine with the
  terminal's input and output, removes the folder, and exits with the
  program's exit (a signal as 128 + its number, as a shell has it). Synapse's
  own words while building go to stderr, so the program's stdout is only
  its own. The words after `--` are the program's `argv` (owner,
  2026-09-29), as the oracle takes them. `--input` and a cross target's
  runner wait for the owner.
  Alternative: keep the binary beside the source, as `synapse build` does.
- **2026-09-29 — A vector nothing gives a lane width has 64-bit lanes.**
  `let 'a' := simd.fill[5]` has no type to take its lanes from; Synapse gives it
  the natural width a lone number has (`core-features.md`), 64 bits, so
  two lanes in a 128-bit register. Before, it had no shape at all.
  Alternative: refuse it and ask for a type, as `simd.shuffle` does.
- **2026-09-29 — Neurons with dependencies: picks.** Synapse builds the
  default target (owner, 2026-09-29: b: `default = true`, or the only
  one), and writes code for `aarch64-macos` only so far, so any other
  default is refused as "not yet" rather than built for the wrong
  machine. The targets Synapse writes code for are listed in `synapse/targets.txt`,
  the one list (Synthia copies it, and the oracle reads it); any target
  not in it is refused (owner, 2026-09-30: "refuse"), and a default that
  is not this machine's is "not yet". A `Kinesin.nrp` without a 'name', a 'prefix', a
  'version' or a target is refused (owner, 2026-09-29), a dependency's
  too; a folder with no `Kinesin.nrp` is a plain program, and a dependency
  must have one. Its 'name' and 'prefix' both follow the module-name rule
  (owner, 2026-09-30: "strict"), with the oracle's reasons. A dependency is reached by its own 'prefix' and names no
  word; two different folders with one prefix, or a dependency whose
  prefix is `std` or `cca`, are refused, as the oracle does; the root
  Neuron's own prefix is checked against neither, as in the oracle. A
  dependency's own 'targets' are checked and not otherwise used. Using a
  runner is open. Two Neurons that depend on the same folder share one
  copy of it. A `version` dependency is refused until the registry
  exists, as the oracle does.
- **2026-09-30 — Parallel `for`: how Synapse builds it.** The rules are the
  owner's (`parallelism.md`); these are Synapse's picks within them, all
  provisional. A loop runs in slices when every iteration writes only
  what is its own — locals it makes, the element at its own position (the
  counter plus one constant, the same one for every read of that list),
  or a sum into one integer — and nothing leaves the loop early.
  - *Worth it*: decided as the loop starts, from its actual trips times
    the statements of one trip (an inner loop's weighed sixteen times, a
    call ten): at 64,000 or more, slices. Never from timing the machine;
    `tune.cplr` will override it once `synapse tune` exists.
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
    inline. `CYBORG_THREADS` in the environment caps the threads; the
    tests set 3, this session's share of the machine.
- **2026-09-30 — `--target <name>` names the target to build (ruled:
  owner, 2026-09-30, "yea").** A lone
  target saying `default = false` leaves the Neuron no default (owner,
  2026-09-30), and a compile (`synapse build`, `synapse run`, `synapse test`) with no
  target named is then refused; checking (`synapse check`, `synapse mir`, `synapse
  precog`) is not. A target is named `--target aarch64-macos`, the
  spelling cargo and clang use, once per command (proposed by Synapse, ruled
  by the owner). A named target must be one the Neuron's `Kinesin.nrp` lists (a
  plain `.nrv` file: one in `synapse/targets.txt`), and is built only on its
  own machine so far, as the default is.
- **2026-09-29 — `synapse prepare`: picks around the owner's rulings.** The
  rulings (the current folder's Neuron; names are Neurons' `name`s, its own or
  its dependencies', comma-separated with or without a space; a name two of
  them share is refused with their prefixes listed, and `--prefix geo2`
  picks one; a `build.nrv` runs in its own Neuron's folder; `--all` runs
  dependencies first, each once, stopping at the first failure) are built
  as ruled. Also ruled (owner, 2026-09-30, "yea" to each, as Synapse had
  built them): named Neurons run in the order written, each once, whether
  asked for by name or by prefix; the names come before `--prefix`, and
  every word after it is a prefix, so `synapse prepare --prefix geo2 money`
  and a second `--prefix` are refused. Picks beside them, still
  provisional: a named Neuron with no `build.nrv` is refused, while `--all` passes
  over those; and the failing one's exit is synapse's. Ruled (owner,
  2026-09-30): each `build.nrv` is an ordinary program with no limits
  and no channel into the build, built as a program of its own with no
  language access to its Neuron's code — it reads a Neuron's files as any
  program would — and given no command-line words: its settings come as
  any program's do, from the environment or files; a Neuron whose prefix differs from its name
  is shown as `geometry (prefix `geo2`)`. `--prefix` takes a comma list
  and mixes with names (owner, 2026-09-30): `synapse prepare money --prefix
  geo2, geo3`.
  A folder is a word with a `/`, or `.` or `..` (owner, 2026-09-30), and
  comes first, as a program does in `synapse run` (owner: "yea"); a folder
  given later is refused. `synapse prepare` with no names, or a folder alone, is
  refused, pointing to names or `--all` (owner, 2026-09-30: "`a`").
- **2026-09-29 — How `synapse test` runs tests.** One test binary holds every
  `test fn` (and no `MAIN`); `synapse test` runs it once per test, the test's
  number its argument, so a check that fires ends that test only. A `must`
  that does not hold ends its test with exit 1 and `line N: …` on stderr.
  `must.stop` runs its expression in a forked copy of the process (POSIX
  `fork`) and the test carries on in the original. A fired check is
  reported in Synapse's own words (`the position can be past the end`), not
  the oracle's. A comparison shows its sides when both have a text form
  (numbers, `bool`, characters, text); other values say only that `must`
  does not hold. Alternative: `setjmp`/`longjmp` in one process.
- **2026-10-09 — The size report's shape** (the owner asked for "size with a
  breakdown"; the breakdown is design claude's reading: by Neuron, by
  function, and the copies each generic produced). Every `synapse build`
  and `synapse run` ends with it, after the sites line: the file's size on
  disk, the program's own code and data, and each function's code by
  Neuron, largest first, a generic's copy named with its types
  (`biggest[int32]`, "a copy of the generic `biggest` (2 in all)"). It is
  measured in the object file before linking, each function running from
  its symbol to the next; one LLVM inlined everywhere has no room of its
  own and is named on one line. The entry, `main`, is counted as the
  program's, since `MAIN` is often inlined into it. A blank a bare literal
  filled is named as built, 64 bits wide. Everything else on disk is Axon,
  the runtime, and the linker's. Alternative: measure the linked file,
  which needs its symbol table kept.
- **2026-10-09 — Where the base's `optional` lives.** It is declared once,
  in a module of its own, and a bare `optional` falls back to it when the
  module has none, which no Neuron can have (`optional` is the base's). A
  type's own function (`fn Counter.next`) without `pub` is seen across its
  Neuron, as the oracle has it (not asked). A `for` over an iterator takes
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
- **2026-10-10 — How Synapse reads a C header.** An `incl.C` line is the
  `extern.C` block Synapse writes (`compiler/src/cheader.rs`): the Clang
  beside the LLVM Synapse is built with (else `clang` on `PATH`) is run
  three times at once on the line's headers (Clang is CNC's, owner
  2026-10-10: borrowed so until the self-contained build, when it ships
  inside the CNC lobe, fetched only by a build listing CNC) — its AST as JSON, its
  `#define`s, and its own `#define`s to leave out — and what comes through
  is written as Nerve and parsed at the line. What comes through follows
  the oracle's README ("What of a C header comes through"); where it says
  nothing, Synapse matches what the oracle does (found by running it): a
  `#define` number may carry C's `u`/`l` suffixes and be hex or octal, but
  not `+5`, `-(3)` or `0b101`; an empty `""` text comes through; a pointer
  that is itself `const` (`char *const`, so `const char *const *` too)
  leaves its function out; only globals C declares `extern` come
  through, `mut` unless C's own type is `const`; C's `...` is a last
  parameter `...'pN'`. Clang 22's own `__size_t` is `uint64`, and
  `__ptrdiff_t` and `__signed_size_t` are `int64`. The headers are read
  for the machine Synapse runs on, not the target. A C struct (owner,
  2026-10-10) is written as a `layout.C struct` when whole, and as
  `layout.C struct FILE;` when opaque, a form only Synapse's own text may
  use; an opaque struct is refused wherever it would be held rather than
  pointed at (a variable, a parameter, a field), not only when built or
  sized. A bit field, or a struct or union written inside a struct, makes
  it opaque. A struct with no fields is opaque.
