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
  and LLVM, out to an object file and a linked binary. The heap is in:
  lists, text built at run time, maps, enums and `match`, and failures as
  values, all of them freeing what they take. Every
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

- `cca build --assert-proofs file.cpls -o out` (2026-09-23): the build that
  checks Precog (`precog.md`, attack (d)). Every site Precog proved keeps
  a run-time check, nothing proven is promised to LLVM, and layer 1's
  folded answers are computed at run time instead. A wrong proof prints
  `proof failed: <the site's sentence>` and `where: file, line N, column
  M`, and exits **104** (an ordinary check that fires exits 101). `nocheck`
  sites proved nothing and are not asserted. The verification session's
  harness runs every generated program this way.

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
  in `Synthia.cplm`, and Synthia does not exist yet, so
  `types::TARGET_SIMD_BITS` holds 128 — the oracle's fixed target and this
  machine's. It moves into the target description the day there is one.
  Alternative: refuse every `simd` until Synthia exists, which would leave
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
  The compiler's own dependency has to be found somewhere, and Synthia does
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
- **2026-09-22 — A function that can fail hands back a variant.** Case 0
  is the answer, each later case is one of the reasons, and the caller
  reads the tag: the same shape as an enum, because it is the same idea.
  `failures.md` decided the model — "the failure is a value the caller
  must handle" — and said nothing about the machine. Alternative: an
  out-parameter for the answer and the reason in the return register,
  which is smaller for a wide answer and worse for every other case.
- **2026-09-22 — Every text handle carries the mark, for now.** The owner
  decided (c): Precog knows which memory a text is in and a free is
  emitted only where it can be the heap, with the sentinel — the high bit
  of the length — as the fallback for a name that may hold either. Precog
  does not carry that fact yet, so CCA pays the fallback everywhere: a
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
- **2026-09-22 — A key type with its own `hash` is refused.** `maps.md`
  allows one, and makes "equal keys hash the same" a check site Precog
  must prove or the final build refuses the map. Precog does not prove
  that yet, so CCA says so rather than trusting it. Structural hashing —
  plain values, text, chars, structs, enums and lists of them — works.
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
  folding). Alternative: loop invariants found by induction, which would
  also prove what gradebook needs (`examples_unproven.txt`).
- **2026-09-23 — Channels: the counting, none of Precog's three parts yet.**
  The layout is the owner's (one block, slots inline, never growing; the
  program holds a pointer to it) and so is `closed` (count the tasks that
  can reach it; a `take` on an empty channel fails `closed` when every
  holder is itself waiting to take from it). What is built is the plain
  form: every channel on the heap, every slot the type allows, and the
  counting everywhere. Of Precog's half of `closed`, one part is built
  (2026-09-23, `mir/channels.rs`): the task that made a channel lets go
  of it where it provably never names it again — directly or through a
  borrow held in another name — rather than at the end of its block, so
  MAIN lending one channel to a producer and a consumer no longer keeps
  the consumer's last `take` waiting for ever (the oracle says `closed`,
  and so does CCA now). A channel ever given away, or a name assigned
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
  exists it is the language's own way out, CCA knows it ends the program
  without a list, and these names stop carrying any weight — not a
  permanent rule about C.
- **2026-09-23 — The exact answer is a 65-bit value.** Plain math is
  exact and checked where it lands (owner, `operators.md`); CCA types
  every `+ - x xx` on integers as one internal type whose range is
  `int64`'s lowest to `uint64`'s highest (spelled `integer`), holds it
  in 65 bits, and computes `+ - x` in 128 so the only question in
  between is that window. Alternative: the narrowest type Precog proves
  (`operators.md` allows it, "especially for SIMD"); later, as an
  optimisation, since the answer is the same.
- **2026-09-23 — `:=` of an exact answer keeps the exact type, for now.**
  The width is decided (`precog.md`, "`:=` widths"): Precog picks the
  narrowest type holding every value the name can have, then adjusts for
  speed. CCA does not choose it yet; it holds the exact type, which holds
  the same values, so only the representation differs.
- **2026-09-23 — Bitwise operators and shifts on an exact answer.**
  Wrapping on mixed sizes is decided (owner, b): an error until written
  `to.int16['a' + 'b'] +~ 1`, and CCA refuses it. The bitwise operators
  take the one size an exact answer's typed operands share; mixed sizes,
  a `:=` name holding an exact answer, and any shift of one say not yet,
  since none of those was asked.
