# the prover

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

Opened 2026-09-22. What the compiler does between the checked AST and
LLVM: run the whole program at compile time over sets of values, prove
every check site it can, and report the rest. The rules the prover must meet
are in `core-features.md` (tiers, no development mode, final build),
`reproducible-builds.md` (no clocks, no compile-time I/O), `parallelism.md`
(what a parallel `for` needs proven), `input.md` (where values come from).

**This document starts from Kespar** (`~/Kespar/design/precog.md`,
2026-09-16..19): the owner's proof-of-concept of the prover, built from this
repo's docs, with a three-layer analyser, a separately-authored oracle, a
program generator that found real the prover bugs, and a bench. Kespar's
method is carried over; every number Kespar marked **provisional** is
still provisional here, listed under Open. What Kespar could not do
(ownership, failures, range types, the final build, LLVM) is added.

## The pipeline

```
.cy ──parse──▶ AST ──check──▶ typed AST ──build──▶ MIR ──Precog──▶ MIR + proofs ──lower──▶ LLVM IR ──▶ binary
                                                          │
                                                          ▼
                                                     proof report
```

**Three IRs** (decided 2026-09-22, owner: b): our AST (the parser's, and
what messages quote), our **MIR** (a control-flow graph: basic blocks,
explicit jumps, one operation per instruction on named temporaries,
every check site, drop, lend and failure edge an explicit instruction,
a source span on each), and LLVM's. The typed AST is what rustc calls
HIR — a second high-level IR would duplicate it (proposed as c,
declined). Everything that reasons — the ownership checker, the prover's
sets and its SMT encoding, the parallel-loop proof, the lowering — reads
MIR, so the flow graph is derived once. **MIR carries proofs**: a
temporary's type includes its proven domain, and a proof made in the prover
rides the instruction to LLVM as `nsw` / `nuw` / `!range`. Proof reuse
keys on canonical MIR (see Reuse), never on source bytes. **the prover is
the last MIR pass** (decided 2026-09-22): after it, MIR is frozen and
lowered as is. Any MIR-level optimisation runs before the prover, so the prover
proves exactly what is emitted, and no later pass can change an
instruction out from under its proof.

`synthia build` runs everything and inserts a runtime check at every
unproven site; `synthia final` runs the same the prover and refuses the
program if any default-tier site is unproven (`core-features.md`). Both
run the prover in full — there is no build that skips it.

## Layer 1 — known values are executed

Everything that does not depend on the outside world is **known**: a
literal, arithmetic on literals, a call with known arguments, a loop over a
known range. the prover executes it at compile time with the language's exact
semantics (integers checked against their width, IEEE bins at their
width, `deci` exactly, text as text). The result is folded in. A program
that reads nothing is executed to the end and its binary is its output; a
check that would fail on known values is a **compile error at the line**
— the program would always fail there, so it never builds. Code that only
ever runs on known values is not in the binary.

Cost is counted in steps (one per AST node evaluated), never in time.

## Layer 2 — bounded values are tracked as sets

A value from outside the program has a **domain**: what its type allows.
`std::input.line[]` is any text; `to.i32[…] otherwise …` on it is any
`i32`; `to.i32[min: 0, max: 100][…]` is `[0, 100]`; a `list[T][lo, hi]` has a
length in `[lo, hi]`; an `extern` result is anything its C-shaped type
holds — or, with a ranged return (`ffi.md`, 2026-09-22), the range,
enforced at the door as a failure. the prover runs the program abstractly
over those domains: each expression gets the set of values it can take, computed exactly through
every operation (`{1, 2, 3} times {10}` is `{10, 20, 30}`).

- A set larger than **4096 values** (provisional) becomes an interval
  `[lo, hi]` — sound, less precise; an interval is spelled out as a set
  again only at **64** values or fewer (provisional).
- Bins are intervals from the start. A list carries a length interval and
  an element domain.
- **`deci`** is tracked like an integer with its own bounds: sets, then
  intervals, at its fixed digit count. **`std::uenr`** (unbounded exact;
  decided 2026-09-22, owner: b) is tracked as an **interval over exact
  rationals** — no width, so no overflow site; its check sites are
  division by zero and narrowing out of it (`to.i64[…]` fails
  `out_of_range`) — **plus a digit-count bound** carried through
  arithmetic (`+` keeps the larger, `x` adds, `xx` multiplies). A
  computation whose digit count grows past a bound (provisional: 10⁶
  digits) gets a **warning** naming the line — not an error, not a check
  site: no value is wrong, and memory is the program's business. Proposed
  (a) rationals only, (b) plus the digit bound, (c) opaque; owner chose b.
- **Text** (decided 2026-09-22, owner: c) carries three things: a length
  interval per unit; an **exact set of values** while small (a literal,
  a literal list, `'cmd' == "quit"` narrowing to `{"quit"}` inside the
  branch — the number rule applied to text, falling back past the set
  cap); and **character-class facts** — "all ASCII digits", "no newline",
  "ASCII only", "non-empty" — so that after a validation (`if
  std::str.all_digits[lend 's']`) a `to.int32['s']` is proven not to fail
  `not_a_number`, and with a length ≤ 9 not `out_of_range` either. Every
  text operation (`++`, `slice`, `trim`, `split`, `replace`, `at`, …)
  carries a rule for the class of its result; an operation with no rule
  yields "any text", never a wrong class. Proposed as (a) length only,
  (b) length + exact values, (c) b + classes; owner chose c.
- A struct carries a domain per field. An enum carries the set of
  possible cases, each with its payload's domain.
- **Control flow**: at an `if`, the condition's set decides which branches
  are possible, and each branch is analysed with the condition assumed —
  a guard `if 'n' </== 100 { … }` narrows `'n'` inside. `match` narrows the
  enum to one case per arm. `otherwise` splits: the success value's
  domain on one side, each failure's on the other.
- **Loops** iterate the analysis to a fixpoint, widening to intervals when
  a set keeps growing. A counted loop is followed **trip by trip** up to
  **100,000 trips per loop** (owner, 2026-09-23: "keep it at 100k", with
  builds of 7 s to 4.5 min measured on 30-line programs) and
  **2,000,000 across the program** (provisional; nested loops multiply),
  then widened.
- **Calls** are analysed per call site with the arguments' domains — the
  same per-shape specialisation generics and variadics already use —
  recursion depth-limited at **8** (provisional). Past that, a recursive
  function's result is bounded **by induction** (owner, 2026-09-23: c):
  assume the bound for smaller arguments and prove it for this one, so
  `fib` on `[0, 92]` is proven without `no_check`. So `square['i']` with
  `'i'` in `[1, 1000]` proves its own multiply, without a contract on
  `square`.
- **Ownership** is what Kespar lacked: every value has exactly one owner
  and lends are tracked by the checker, so the prover knows which memory a
  loop iteration can touch. That is the proof `parallelism.md` needs —
  two iterations that would conflict cannot both own or mutably lend the
  same value — and it is a fact about the program, not a domain.

At each check site the prover asks: is every value in the result's set legal?
Yes → **proven**, no check. No value legal → see below. Otherwise →
**unproven**, on to layer 3.

**A site where no value is legal** (owner, 2026-09-23) is a compile
error only when it is **reached on every run** — layer 1's case, where
the program would always fail there and so never builds. A site that
fails whenever it runs but *may not run* is **unproven**, like any
other: a runtime check in the working build with a `note:` saying the
line fails whenever it runs, and a compile error with the
counterexample in the final build. The working build never refuses to
compile (2026-09-14), and this is why it matters: the prover's sets widen,
so it sometimes believes values reach a line that never do. The
generator found exactly that — a program with no input at all where kayo
refused a line inside an `else` the oracle shows is never taken. Under
the stricter reading, the analyser's imprecision becomes the
programmer's error. Proposed alternatives: refuse any such site,
reached or not (what this document said before); the same without the
note.

Check sites are: integer overflow (plain operators, which are exact
and checked where the value lands, or past 64 bits in between — owner,
2026-09-23, `operators.md`; `+~` etc. are never sites), division and `mod` by zero, `[i]` bounds, narrowing (`u64`
into `i32`, a value into a range type), a length bound on a bounded
list after every change, `to.…` conversions that are not failures, and
the "must leave" rules the checker already enforces. Failures (`fail`) are
not check sites: they are values with domains.

## Layer 3 — the exact question

An unproven site is asked exactly, as a **bit-precise SMT query** over the
real program semantics, put to a **portfolio of two solvers, Z3 and CVC5**
(decided 2026-09-22, owner: b; over Z3 alone, or Z3 behind an interface).
Both are pinned to exact versions and fixed configurations. For
determinism the portfolio is not "first to answer": **both run to their
full budget**, and the results combine — any `unsat` is a proof (a wrong
`unsat` would be a solver bug, which is what the the prover oracle exists to
catch); if both say `sat`, Z3's counterexample is the one reported (fixed
order, so the same input on every machine); `unknown` from both is
unproven. The doubling is paid in parallel across cores.

The query encodes the path to the site — the domains, every assignment on
the way, every branch taken, loops unrolled to **16 iterations per loop
and 64 across the program** (provisional; deeper made Z3 quadratic on
nested loops in Kespar) — as bit-vectors of the exact widths, and asks
whether any input reaches the site with an illegal value.

- `unsat` → **proven**. No check.
- `sat` → a **counterexample**: concrete inputs that make the site fail.
  Default tier, working build: the site keeps its runtime check. `check`
  tier, or any tier in the final build: a compile error that prints the
  counterexample.
- unknown, or the solver's budget spent → **unproven**, treated as `sat`
  without a counterexample. The budget is each solver's `rlimit` — a step count,
  never a timeout — **10,000,000** per query (provisional), so the answer
  is the same on every machine.

**Disagreement** (`unsat` from one, `sat` from the other; decided
2026-09-22, owner: c): something is wrong — a solver, or the prover's own
translation of the program into the query, which both solvers received
(owner's point: a bogus counterexample may be right about a wrong
query). the prover runs the `sat` solver's counterexample at compile time —
the program's reads replaced by the counterexample's values, no I/O,
deterministic. If the site fails on them, the site is `sat` for real.
If it does not, the site is **unproven** — neither answer is trusted on
that query. The build continues with the safe answer either way. The
report says, in plain words, that this is a bug in a solver or in
the prover and not in the program, names both answers, points at Cyborg's
issue tracker (never a solver's — the writer need not know what SMT is,
and the diagnosis of which is at fault needs the the prover oracle's second
encoding), and writes the query with both answers to a file to attach.
Proposed alternatives: refuse the build; treat as unproven without
running the counterexample.

Unrolling to a bound means a site inside a loop may be proven for the
first 16 trips and unproven after. Kespar reported that honestly and
named the missing piece, decided here:

## Loop invariants — templates first, then unproven (decided 2026-09-22)

A loop the prover cannot run to the end (its trip count depends on input and
is too big or unbounded to follow) needs a fact that holds on every trip,
checked once — an invariant. Nobody writes one. the prover **guesses from a
fixed list of shapes** and keeps each guess the solver confirms
inductively (true before the first trip; true after a trip if true before
it): a counter stays within the range's ends; a sum is bounded by trips
times the largest element — which is what makes an average fit the
element type, since a sum bounded by *n* × max divided by *n* is at
most max (`examples/gradebook.cy`, proven by kayo since dd04991, 2026-09-24); an index never decreases and never passes the
length; a running max/min stays within the elements' domain; a count
never exceeds the trips so far. A confirmed guess is a proof, never an
assumption. If no shape fits or none is confirmed, the loop's sites are
**unproven** — working build checks them, final build asks for a guard or
`no_check` — exactly as if no guessing existed. Guessing only adds proofs.

The programmer's way to supply a fact is the one the language already
has: a **guardrail** inside the loop (`if 'total' > 1_000_000 { fail
too_big; }`), which Precog reads on every trip; `check` in front makes
the compiler say whether it was enough. No invariant syntax. The
template list is provisional and grows by measurement (Kespar's bench,
the generator), not by guessing what programs do. Owner: "a and fallback
to c".

## The tiers, as emitted

- **proven** → plain instruction, and the proven fact handed to LLVM
  wherever LLVM benefits (owner, 2026-09-22): always where it has a
  native home (`nsw` / `nuw` on add, sub, mul, shifts; `!range` on loads
  and calls — free); through `llvm.assume` per kind of fact where
  measurement shows it helps (it can also hinder — rustc dropped most of
  its assumes for that reason); and through a custom pass in the C++
  wrapper for a kind of fact worth more than LLVM offers a home for.
  Removing the check is the win that needs nothing from LLVM; a fact
  LLVM ignores costs speed, never correctness. **Measured by us, when
  we build** (owner): the compiler's suite carries a bench that builds
  the same programs with each kind of fact handed over and withheld,
  results kept in the repo and re-run on every LLVM upgrade — our
  numbers on our LLVM, not another compiler's findings on theirs.
- **default, unproven** → working build: a checked instruction, trap on
  failure with the site named; final build: refused.
- **`check`, unproven** → compile error with the counterexample, or
  "solver gave up at line N; add a bound at line M".
- **`no_check`** → plain instruction, no proof attempted, listed.

## `:=` widths

From layer 2's sets: the union of every value the name holds and every
expression it types; the narrowest width that holds it — then adjusted
for speed as `core-features.md` says (a scalar takes the machine's
natural width; list elements and struct fields take the narrow one). If
layer 2 only has an interval touching the 64-bit limit, layer 3 is asked
for the maximum (bounded binary search on "can it exceed K?"); failing
that, `:=` at that boundary is the compile error `core-features.md`
already specifies: write the type.

A decimal literal under `:=` means `f64`'s answer; a narrower bin is
picked only where it gives identical results across the whole program,
since a bin's width changes values (owner, 2026-10-02). `*0.1` is a
`deci` instead.

## Determinism

`reproducible-builds.md`, restated for the prover:

- No wall-clock anywhere; every limit is a count.
- Z3 and CVC5 are called with fixed configurations and step budgets
  (`rlimit`); results do not depend on thread scheduling.
- Sites are analysed and reported in source order.
- Compile-time arithmetic is performed at the exact width the target uses;
  the target is a description in the manifest (`Synthia.ttsr` / `CyJet.cyr`), never the host.
- Nothing is read at compile time except the source and the config: no
  files, clock, environment, network.
- The compiler's and both solvers' versions are in the report header.

## Reuse and parallel work

the prover's own work splits: sites are independent questions, call-site
specialisations are independent jobs, layer-3 queries run across all
cores. Results are **content-addressed**. The key for a function's analysis is
the hash of: its MIR in **canonical form** (spans live in a side table
and are not hashed — a comment shifts every span and must invalidate
nothing); the canonical MIR of **everything it reaches** through calls
(a Merkle hash over the call graph — with per-call-site analysis the
callee is proven *inside* the caller's proof, so a callee change must
invalidate the caller); its argument domains; the target description;
the the prover version; the pinned solver versions and configurations.
Tiers are MIR instructions, so they are in the hash already. An entry
whose key matches is reused, never re-proven; nothing is ever skipped
(`core-features.md`, 2026-09-16). A shared cache of those keys is what a
team's build farm serves. (Cold read of this document, 2026-09-22: the
first draft hashed raw MIR bytes and the function alone — both wrong.) Memory: must run a large program in 32 GB and
scale near-linearly with cores and memory on independent units (owner,
2026-09-21). GPU offload (`parallelism.md`) is a decision the prover makes
from the same independence proof.

## The report

Per check site, in source order:

```
line 7   $total + square($i)   overflow           proven    (layer 2: max 333833500)
line 12  $xs[$k]           out of bounds      checked   (layer 3: sat, k=100 when len=100)
line 15  $a div.floor $b       division by zero   trusted   (no_check)
line 9   square: $x x $x       overflow           proven    (layer 2, at: 1 call site)
```

Three lists in the build output: `no_check` sites, `check` sites passed,
default sites still checked (working build only; a final build has none).
`--types` adds every `:=` name with its chosen width and the value that
decided it. The the prover oracle (`compiler-verification.md`) reads this
report to know which claims to attack.

## Tables: a small pure function computed at compile time (owner, 2026-10-17; limits 2026-10-19)

When Precog proves a function pure and its input range small (fib's is 0 to 92), it computes every answer at
compile time and the function becomes a lookup in a table ("compile time is free"). The limit is the programmer's,
who knows the target:

```
targets = [
    cyjet::target[name = "aarch64-macos", default = true, table_bytes = 262_144],
    cyjet::target[name = "aarch64-none", table_bytes = 4_096],      # a small board: small tables
]

table_bytes 1_024 fn square [let $x: u8] -> u16:    # this function's own limit
    …
;fn square
table_bytes 0 fn fib [let $n: i64[min: 0, max: 92]] -> i64:   # never a table
    …
;fn fib
```

- Per target in `CyJet.cyr`, in bytes; **65,536** when not written (design claude's number: about the fast cache
  of this Mac's cores, past which a lookup can be slower than computing).
- Per function, `table_bytes N` in front of `fn`, where the `bare` marks sit; it overrides the target's.
- A table bigger than its limit is not made: the function compiles as written. Under `no_cheat` no table is used.
- The build report lists each table (entries, bytes, which limit applied) under Optimizations and Sizes.
- The oracle reads both and computes every call: the answers are the same with or without a table.
  `examples/table_limits/`, reject tests `table_bytes_*`.

## The the prover oracle (decided 2026-09-22, owner: a + b + c)

The second of the three oracles in `compiler-verification.md`, attacking
every claim in the report, by a different author than the prover, no shared
code. Three attacks, one per way the prover can be wrong:

- **a. Exhaustive and random execution.** For every site the prover calls
  proven, run the program through the interpreter oracle over every input
  when the domains are small (all of `i8`, every list to length 4),
  and over random plus edge values (0, −1, min, max, empty, one past the
  bound) when they are not. A check that fires is a the prover bug. Kespar's
  generator did this and found three layer-2 bugs in one commit.
- **b. An independent interval analysis** — deliberately simple,
  separately written — recomputing each site's range. A site where
  the prover claims a range the simple analysis cannot justify is flagged.
  Attacks layer 2, which is where Kespar's bugs were.
- **c. The negation, re-encoded.** For every solver-proven site, a fresh
  solver instance is asked "find an input that fails here" through a
  second, independently written program-to-query translation. Attacks
  layer 3's encoding with the part of the prover least likely to be wrong,
  the solvers themselves.
- **d. Proofs as assertions** (decided 2026-09-22, owner: yes; from the
  cold read of this document). A proven fact handed to LLVM as `nsw` /
  `nuw` / `!range` is a *promise*: if the prover was wrong, the result is
  silent undefined behaviour, not a missed check. So the compiler has an
  authors-only mode that emits every proven fact as a **runtime
  assertion** instead of a promise — same the prover, same proofs, nothing
  trusted — and the differential harness runs the examples, the twin
  programs and the generated programs through it. An assertion that
  trips is a wrong proof, caught before any writer's binary is built.
  Not a user build: `synthia build` and `synthia final` are unchanged,
  and the no-development-mode rule is untouched — this mode proves
  exactly as much, it only refuses to believe itself.

Owner chose b + c on top of a after asking the difference: c alone
leaves layer-2 proofs covered only by testing; b covers them by analysis.

## What Kespar learned (2026-09-16..19)

- The sieve took 14.7 s of compile time until four engineering fixes
  brought it to 2.6 s: no state clones to read a condition, loop-exit
  states joined in place, exact sets reference-counted, ranges not spelled
  out past 64 values. Trip-by-trip following is where compile time goes.
- **Trip-by-trip is where the proofs come from, not only where the
  compile time goes** (kayo, 2026-09-22). Joining at a loop head cannot
  see that a counter and a sum move together, so it gives the counter
  every value from 0 upwards and the sum overflows in the analysis
  though never in the program. `examples/scores.cy` went from 4 of 9
  sites proven to 9 of 9 — and from 1675 steps to 170.
- Narrowing must reach through operations and fields, not only names:
  `if list.length[lend 's'.scores] > 0` has to teach the *list* its
  length, so that a second `list.length` of it proves the divisor
  non-zero (`examples/gradebook.cy`'s comment, now true).
- Z3 goes quadratic on nested loops past ~16 unrolls.
- Big lists are carried as a summary (length interval + element domain),
  not a value per element.
- None of the four bench programs needed Z3: layer 2 proved everything or
  the program folded to its output.
- The fuzzer found real the prover bugs the oracle caught — three in one
  commit. The generator and the oracle are not optional.

## Open

- The invariant template list: which shapes, measured against real
  programs.
- Every provisional number: set cap 4096 / re-set at 64, trips 2M
  across the program (100k per loop is ruled), induction over at most
  4,096 argument tuples per recursive function (kayo's pick, 59c3f56), recursion depth 8, unroll 16 / 64, `rlimit` 10M. Kespar's picks;
  to be measured, not guessed.
- The character-class vocabulary for text (which classes; how each std
  text operation maps them) — decided in principle (c), list open.
- The `std::uenr` digit-count warning threshold (provisional 10⁶).
- The pinned solver versions and configurations.
