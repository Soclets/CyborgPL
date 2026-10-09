# check — the verification session's tools

check claude's code (`design/verification-session.md`). It attacks what
CCA and the oracle *claim* — their output, exit codes, messages and
reports — and never reads `cca/` or `oracle/src`.

```
cd check && cargo build --release        # needs oracle/ and cca/ built with --release
./target/release/check run --cases 1000 --keep-going
./target/release/check corpus
```


**The 2026-10-07 revision** (design/revision-2026-10-07.md): the generator, the printer and the
corpus are written in it — `->`, `equal_to` / `not_equal` / `at_most` / `at_least`, `not`,
`immut`, `no_check`, `no_cheat`, `range[from a, to b, by c]`, `pos` on every index,
`list.fill[value v, count n]`, `simd.fill[value v]`, `is` for a case's value, `[@x, @y]` groups
(required for a borrow from two inputs), `brain16` only. `#strict` and MTM are gone, so the
generator writes neither. Older notes below keep the spellings of their day.

**list.remove, shadow, str.at / str.slice** (owner, 2026-10-08): the generator removes an item
(`pos`, its item printed or dropped; past the end as a risk) where the list's least length stays
known (once per life of the list, outside `check`); shadows a visible plain name (never a loop's
counter) with `shadow`, in the same scope or an inner one, and in 50 programs in 1000
(`--shadow-wrong`) writes a plain `let` over it instead, a mistake both engines must refuse;
and, with `cord` (the standard library, stdkit until 2026-10-09) not avoided, reads text by `str.at.byte/point/grapheme` and
`str.slice.grapheme`, out of range sometimes. The evaluator models remove and shadow; the text
reads are compared between the engines only. Default avoid: `cord` (CCA has no standard library yet).


**Precog is Cortex** (owner, 2026-10-09): CCA's analyzer's Nerve name. The notes below say Precog,
as they did when written; `check claims` asks `cca cortex --json` (`cca precog` is refused, owner
2026-10-09: one spelling per thing).

## What is here

- **`gen/`** — the program generator. `seed → program` is a pure function
  (SplitMix64, no clock, no machine state), so any finding replays from its
  seed and size. Programs are valid by construction: every name declared
  before use, types matched, every loop bounded, and every operation kept
  inside its type by the generator's own simple interval bookkeeping, which
  folds an operand into range with `lo + (e mod w)` — a plain CyborgPL
  guardrail — where it could overflow. A few operations per thousand are
  left unguarded on purpose (`--risk`, per mille), so the engines are also
  compared on checks that fire — never inside a `check` block or on a
  `check` statement, since CCA must refuse a `check` it cannot prove. It writes: `int8`…`int64`, `uint8`…`uint64`,
  `bool`; `let` (plain / `mut` / `immut`), assignment, `if` / `else if` /
  `else`, `for` over `range` with every step sign (and `perm`), `while` and
  `loop` with counters, `break` / `continue` / early `return`, blocks,
  `check` / `nocheck` on statements and blocks, functions with typed
  parameters and returns calling one another, `plus minus times`, every `div.X` / `mod.X`,
  `pow`, unary `-` (word operators, owner 2026-10-03, 7f2dafb; `-->` arrows),
  bitwise `and.bit or.bit xor not.bit shl shr`, wrapping `plus.wrap minus.wrap times.wrap`,
  comparisons, `and or not`, `to.<int>[…]` narrowing and widening, range
  types on names, parameters and returns (`int32[0, 100]`, `int32[1, max]`,
  `int32[min, 100]`), and both parenthesised and "loose" spellings of the
  precedence rules. **Lists** (design/lists.md, 2026-09-29): typed literals
  (`[]`, `[7,]`, `[a, b]`), items read by position, `change 'xs'[p] = v`,
  `list.append`, `list.length`, and `for 'x'` / `for 'i', 'x' in lend 'xs'`.
  The generator keeps each list's shortest and longest possible length, so a
  position is guarded (a literal within the shortest, `1 + (e mod
  list.length[lend 'xs'])`, or the walk's own position) unless it is left
  unguarded on purpose. A list being walked is never changed. Positions stay
  out of `check` blocks and `check` statements: a guarded position is a fact
  about the list's length that Precog need not see. The Precog oracle's
  analyses (b) and (c) do not model lists yet: a program with one has no
  complete interval answer, and the solver skips it once it changes a list.
  **Returned borrows** (46faa74, 09044b6; `--avoid borrows` turns them
  off): about a third of programs get borrow functions `b1…`, each handing
  back a borrow of one of its `lent` list parameters — directly, through an
  earlier borrow function, or either of two by the lists' lengths — with no
  groups (Precog reads the body) or `@x` on exactly what the body can
  return. MAIN gets blocks of lists, borrows of them through these functions
  (some through earlier borrows, so the chains run several calls deep),
  uses, and a `give`, an append or a change of one list, followed only by
  uses of the borrows that do not depend on it. `--borrow-wrong` (per
  mille, default 250) gives a program one mistake on purpose: a borrow of a
  local handed back, a returned parameter left out of the group, an unsaid
  borrow (`-> list[T]`), or a list changed while a borrow of it is used
  after. Such a program must be refused by both engines
  (`*-accepted-borrow-mistake`; where each points is not compared yet);
  a right one must not be refused for a borrow (`refused-right-borrow`).
  **`#strict` files** (owner, 2026-09-27; `--strict N` per mille, default
  200; `--avoid strict`): `pos` on positions, labelled ranges with `by`,
  `mut`/`immut` on every declaration, `temp`/`perm` on every `for`,
  `giveS` in a match that gives a value, groups on returned borrows, and a
  printer whose every site is proven in its own function. Since a strict
  file refuses any site not proven from its own function, in every build,
  the whole program is generated under the `check` rules (no risk, no
  positions, nothing unseen), and promised ranges are whole types.
  **Parallel-friendly loops** (2026-09-30): `for 'x' in lend.mut 'xs' {
  change 'x' = … }` (the item written back, owner 4ef0ed8; the walked
  list is not touched otherwise while it lasts), and loops over every
  position of a list whose trips are independent: each item rewritten
  from itself, or all added into a sum printed after. CCA may run these in
  slices (34d9c18). A CCA binary gets `CYBORG_THREADS` so that jobs ×
  threads stays within the three-core share: 1 by default, `--jobs 1
  --threads 3` for a run aimed at the slicing.
  **deci and conversions** (owner "yea", 2026-09-30; `--avoid deci`):
  blocks of deci32/64/128 values from literals (some with trailing zeros)
  and from each other through `plus minus times div.near mod.floor pow` and negation; conversions
  from integers (with `otherwise` only where the integer type could have
  more digits), from text (good and bad), from bins, and between widths; a
  deci range type, sometimes pushed out of it; and text read as integers.
  All is seen through printed integers: `to.int64[d x 10^k] otherwise -1`,
  comparisons printing 1 or 0, and `str.bytes` of the text. My exact
  evaluator and the solver do not model deci: such programs are compared
  between the engines only (`Program::unevaluable`). Text and characters too: text into ASCII (`not_ascii`),
  byte lists read as UTF-8 (`not_utf8`, from a stray byte or 255),
  `char.point.from_code` (`not_a_char` past the last point), one
  character of text (`not_one_char`), a digit read through text, and text
  into utf16/utf32, seen as lengths and codes.
  **brain16** (owner, 2026-10-02; `--avoid brain16`): blocks of `brain16`
  / `bfloat16` / `bf16` values from literals, integers, text, a deci and a
  bin, worked on with `plus minus times div.near mod.floor pow`, seen as `to.int64[…] otherwise
  -1`, either deci form scaled, comparisons and the length of either text
  form; sometimes into a `bin16`, past its 65504 as a risk. Never inside a
  `check` (Precog has no brain16 domain, so every site is a run-time
  check), and, like deci, compared between the engines only.
  **Every division says how it rounds** (owner, 2026-10-02, 5a12c07): the
  old `div` / `mod` are written `div.floor` / `mod.floor` (the printer
  too); on integers also `.trunc`, `.ceil` and `.near` (a tie to even) and
  their `mod`s, always parenthesised as an operand and modelled by the
  evaluator, the solver and the intervals through one `BinOp::quotient`;
  on decis and brain16 `div.near` is ordinary division, with `.floor`,
  `.ceil`, `.trunc` and their `mod`s (never `mod.near`, refused there).
  The roundings past `.floor` are avoided by default until CCA has them
  (`rounding`).
  **`for !cheat`** (owner, 2026-10-01, `!cheat` since 2b7f336; `--avoid
  no-cheat`): about one range loop in seven is `for !cheat` or `for !cheat
  seq`; same answers as a plain `for`.
  **Enums** (design/enums.md; `--avoid enums` turns them off): about
  half of programs get one or two, with integer data on some cases; enum
  names, changes, `==` on them, `match` statements, and `match` giving a
  value in a declaration, where an arm may leave instead. A case's data
  has a range only the generator knows, so it is kept out of `check`.
  The solver (c) models enums: a value is two terms, its case and its
  data; a `match` runs each arm under its own case, with the data bound;
  `==` compares both. (Written after a layer-3 proof of a divisor from a
  match was found wrong, 2026-09-29.)
  These findings are not shrunk: a step would not keep the program's own
  expectation up to date. The mistake sits at the top of MAIN, where it
  always runs (the oracle checked borrows only on the paths it ran;
  mostly fixed in 061e1fc).
- **Printing** (owner, 2026-09-23): every program carries its own printer in
  the base language — digits into a `list[uint8]`, then C's `write` under
  `nocheck` — varied by seed, compared like everything else.
- **Input** (owner, 2026-09-23): `--inputs K` makes MAIN read K values as the
  lengths of environment variables `CHK_IN1…` through `extern.C` `getenv` +
  `strlen`. The harness picks four input sets per program from the seed
  (all zero, then edges and random lengths); a finding records the
  variables and their lengths. Program runs see only those variables.
- **`harness/`** — the `check` binary. For each program and input set: the
  oracle binary; `cca build` and the binary it writes. Compared:
  - both ran → stdout byte for byte, both exit 0;
  - both stopped at run time → the same line, the same stdout before it;
  - CCA refused at compile time at line L → the oracle must stop at line L
    (a check that fails on known values is a compile error at the line,
    `design/precog.md` layer 1);
  - CCA's "not yet" → counted as coverage, never a finding.
  Findings are grouped by kind and by both engines' messages with numbers
  removed, so one root cause shows as one line. Each group's first seed is
  shrunk (remove statements and functions, replace expressions with their
  parts or small literals), keeping a step only if the same kind of finding
  survives in the build that showed it. A candidate must first be well
  scoped and make the oracle behave as before (both cheap); its CCA build
  gets 30 s. A shrunk program carries its inputs as `# env` lines.
- **`corpus/`** — every past finding, re-run for ever by `check corpus`.
  A `# env NAME LENGTH` line gives an input set.

## The Precog oracle (design/precog.md, attacks a–d)

What CCA claims to have *proven* is attacked four ways, none sharing code
or reasoning with CCA:

- **(a) Every input.** `check run --exhaust` narrows each input at the door
  of MAIN into a small typed value — an `int8` or `uint8` (256 values), a
  `bool`, or a small range type like `int16[-5, 20]` — so the rest of the
  program, and Precog, see exactly that domain; the raw inputs are never
  used again. Every combination (at most `--domain`, default 4,096) is run
  through the oracle and both CCA builds, so a proven site reached by any
  input in the domain is checked by the `--assert-proofs` build.
- **(b) A simple interval analysis** (`gen/src/intervals.rs`), written
  separately from everything else: one interval per name, loops trip by
  trip, calls followed, joins at merges, bare-number groups folded exactly.
  A claim whose value (b) keeps inside what CCA claims is justified; if (b)
  cannot follow the whole program (a loop past 14 trips, a call past depth
  12) it justifies nothing.
- **(c) The negation, re-encoded** (`gen/src/solve.rs`): check claude's own
  translation of the program into Z3 questions, following design/ (exact
  maths checked where it lands, floored `/` and `mod`, bitwise at the
  operands' width, shift counts 0..width−1). `check claims` reads
  `cca precog --json`, matches each proven claim to the generator's tree by
  its span, lets (b) justify what it can, and asks Z3 of the rest, over
  every input Precog considers (any `uint64` length): can its value fall
  outside the set or interval CCA claimed (or, for a claim without one,
  can the site fail)? A "yes" counts only when **confirmed**: the program
  is run exactly on the solver's input by `gen/src/eval.rs` (an evaluator
  of the generator's tree, checked against the oracle by `check evaltest`)
  and the site is seen to fail or its value to leave the claim. The
  translation approximates on purpose (loops cut, deep calls unknown), and
  unconfirmed answers are counted, never reported. A confirmed refutation
  with a short enough input is also run through both engines. `check claims-run --cases N` does it
  over many programs. `check aim --seed N` shows every site of one program
  and a failing input where Z3 finds one; `check run --target` adds those
  inputs to each program's input sets.
- **(d) Proofs as assertions**: every program is also built with
  `cca build --assert-proofs` (see below).

A claim whose span and type fit more than one of my sites — two arguments
of one call with the same type, which CCA gives the call's span — is
counted as ambiguous and not asked (2026-09-29: four "refutations" in a
100-program batch were such pairings, not CCA's). Instances of one site in
several inlined calls share its node and are still asked together.

## Translation validation (Alive2) — retired

**Not run any more** (owner, 2026-10-02: "We have RRx2 to replace Alive2"):
the LLVM-IR half of translation validation is RRx2-TV's, run when the
owner says (`design/verification-session.md` item 4, c42ac8f). Mine is the
Cyborg half, that the IR CCA hands to LLVM means what the MIR meant. The
command below stays for the record.

`check tv --cases N` (owner: Alive2 installed 2026-09-30; CCA's
`--emit-llvm`, ae478cd): for each program, Alive2 checks that LLVM's
rewrites of CCA's IR keep its meaning, with the flags and facts CCA added
from Precog's proofs. Two checks, in `harness/src/tv.rs`: **per function**
(every function made external, LLVM's function-level O3 passes without
inlining or attribute inference, each function against its own rewrite:
the meaningful one), and **CCA's own pipeline** (`before.ll` against
`after.ll`), judged only for functions that call none of the module's own
functions, since an inlined callee always reads as "source more defined"
there. An incorrect transformation's files are kept in `work/tv/<seed>/`.
Alive2's own optimiser crashes on this Homebrew LLVM, so `opt` makes the
per-function target. First run (30 programs): 211 correct, 0 incorrect,
70 not proved (functions that never return, solver timeouts).

## Which build a run uses

Every program is built twice: `cca build` (the working build: unproven
sites keep a runtime check) and `cca build --assert-proofs`
(`design/precog.md` attack (d): every proven site keeps a run-time check
that exits 104 with `proof failed:`). Both are compared with the oracle;
an exit 104 is a finding of its own kind, `proof-failed`, the most
serious. Not the final build. A green run says nothing about sites that
no input reached — that is what attacks (a)–(c) are for.

## Known bugs steered around

`--avoid` keeps the generator away from bugs already reported, so they do
not drown new ones; their cases are in `corpus/`. `--avoid none` aims
straight at them. Default: `none`.

- `new-words`: no `!mut`, `!check` or spaced `! 'x'` (the main spellings of
  9a9e573). With them on, three programs in
  four use them and the fourth MTM's `immut`, `nocheck` and `not`, which a
  lone file also takes. In CCA since c9d7b2a8.
- `no-cheat`: no `for !cheat` (in the oracle since 2b7f336, CCA c9d7b2a8).

- `rounding`: only `div.floor` / `mod.floor` on integers and `div.near` /
  `mod.floor` on decis and brain16 (all four roundings in CCA 81445c7).

- `brain-mod`: no `mod` on brain16 (the divisor's sign, owner 426371c;
  in CCA 3657a11). (Negative
  brain16 values, corpus `2026-10-02-brain16-*`, fixed in a155938.)
- `brain16`: no brain16 blocks at all.

- `deci-useful`: no `to.useful.…` forms (CCA took the 2026-09-30 spelling
  in 32b7039).

- `exact-bits`: every operand of a shift, bitwise or wrapping operator gets
  a size with `to.T[…]`. The width rule is settled (owner, 2026-09-23 a and
  363f19f) and the evaluator follows it; CCA had refused an exact answer
  under a unary minus and a bare number shifted by an exact count (corpus
  `2026-10-05-*`, fixed in 94dbca3), so this is off by default now.

- `data-width`: read a `match` arm's data only as `to.T['d']` (the oracle
  gave it no width for `not.bit` or the wrapping operators; fixed 2eb7b2f).
- `item-width`: read a `for … in lend` item only as `to.T['x']` (the
  oracle gave the item no width for `+~` or `not.bit`; fixed in 5750ea6).

- `mixed-width`: print through `to.int64[…]` / `to.uint64[…]`. Needed
  only while mixed widths were open; settled by the owner (dc83c05: plain
  math is exact, checked where it lands). The generator never mixes widths
  in one operation, so the provisional wrapping-on-mixed-sizes pick never
  arises.
- `ranges`: no range types (CCA took them in f23f299).

- `shl-width`: `shl`, `not.bit` and the wrapping operators only on names
  and `to.T[…]` — no longer needed (CCA takes the 88da73f rule since
  f40f659), kept as an option.
- `u64-high` (uint64 values ≥ 2^63 handled as int64) and `range-edge` (a
  `for` whose last step passes the counter type's edge) — both fixed in
  CCA c36cfce; still available as `--avoid` items.
- Always (no switch): a shift, bitwise or wrapping operator never has bare
  numbers on both sides — with no width anywhere its meaning has no ruling
  yet (design claude, 2026-09-24).
- `pow`: no `pow` (formerly `xx`; CCA generates it since d549b34; on by default).

## Harness hygiene

**Three cores** (owner, 2026-09-23): the machine's ten cores are split
between the three heavy sessions, three each, one left for the desktop.
`check` runs at most three jobs at a time (`--jobs` can only lower it),
counting every `cca build` and oracle run; `check/.cargo/config.toml` builds
with three jobs, and `oracle/` and `cca/` are built here with `-j 3`.

Each engine run gets its own process group and its own output files, and
the group is killed when the run ends. (Before this, a `cca build` stopped
at its limit left a solver running that wrote into the next run's stdout —
NUL bytes and false "output" findings, 2026-09-23.) The shrinker confirms
every step it keeps by running it alone; a step that disagrees only under
load is set aside in `work/flaky/` with both runs' details.

## Provisional

- **A shrink stops after 15 minutes** (2026-09-29), keeping the smallest
  program so far: shrinking a program that made CCA overflow its stack ran
  79 minutes on one core. Alternative: no limit.
- **An oracle stop before CCA's refusal agrees** (2026-09-29, seed 4091):
  CCA refuses at compile time a site that fails whenever it is reached,
  while an earlier failure it keeps as a run-time check (it may never run)
  stops the oracle first. Counted as agreement when the oracle's line comes
  before CCA's; line order can be wrong across loops and calls.
  Alternative: compare only where both name a line, and list the rest.

Picks this session had to make to keep moving. Each is a question, not a
decision; dated, with the alternative.

- **Engines run as processes, not linked in** (2026-09-23). Kespar linked
  its engines as libraries; here that would mean compiling against
  `cca/` and `oracle/src`, which this session must not read. Cost: a CCA
  build per case (≈ 0.1–0.5 s).
- **Wall-clock limits: 20 s per program run, 120 s per `cca build`**
  (2026-09-23), a safety net against a hang. A run past its limit is a
  finding (`…-timeout`); a build past it is counted as slow, not a finding,
  since compile time is not a concern (`design/core-features.md`). Neither
  engine offers a step budget to the outside yet; with one, the run limit
  would be a count, as `design/precog.md` asks of Precog. Alternative: no
  limit.
- **The oracle's refusals and its run-time stops are not told apart**
  (2026-09-23): it prints both as `file: line N: …` and exits 1. The
  harness only compares lines. Alternative: ask the oracle for a marker.
- **Input sets**: four per program, lengths from 0, 1, 2, 3, 7, 10, 127,
  128, 255, 256, 1000, 32767, 32768, 65536 or random below 70,000
  (2026-09-23). Alternative: exhaustive over small domains (attack (a)).
- **The Precog oracle's numbers** (2026-09-24): at most 4,096 input
  combinations for (a) (Kespar's number); inputs of at most 200,000
  characters when an input must be run (the environment's practical limit);
  Z3 at `rlimit` 20,000,000 per question (2026-09-29: 86% of claim
  questions answered on 20 programs in 275 s, against 79% at 6,000,000 and
  65% at 1,500,000 with the "nothing failed before" condition, now dropped;
  `CHECK_RLIMIT` overrides) and 600 s per program for claims; loops
  unrolled to 14 trips, calls inlined to depth 12, translations cut at
  60,000 definitions (calls stop being inlined at 20,000); powers unrolled
  to exponent 16. Alternatives: bigger bounds, at more time per program.
- **Claims with nothing to ask** (2026-09-24): a negative literal's minus,
  a `for` counter over literal bounds, and any claim inside the program's
  own printer are counted, not checked. A call's result landing in the
  callee's return type is not matched (the translation checks it at the
  `return` instead).
- **Generator sizes and weights** (statement counts, depth, how often each
  construct appears, the work budget of 20,000 statement-trips for MAIN and
  400 per function) are guesses (2026-09-23), to be tuned by what finds bugs.
