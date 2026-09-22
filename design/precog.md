# Precog

Opened 2026-09-22. What the compiler does between the checked AST and
LLVM: run the whole program at compile time over sets of values, prove
every check site it can, and report the rest. The rules Precog must meet
are in `core-features.md` (tiers, no development mode, final build),
`reproducible-builds.md` (no clocks, no compile-time I/O), `parallelism.md`
(what a parallel `for` needs proven), `input.md` (where values come from).

**This document starts from Kespar** (`~/Kespar/design/precog.md`,
2026-09-16..19): the owner's proof-of-concept of Precog, built from this
repo's docs, with a three-layer analyser, a separately-authored oracle, a
program generator that found real Precog bugs, and a bench. Kespar's
method is carried over; every number Kespar marked **provisional** is
still provisional here, listed under Open. What Kespar could not do
(ownership, failures, range types, the final build, LLVM) is added.

## The pipeline

```
.cpls ──parse──▶ AST ──check──▶ typed AST ──Precog──▶ annotated AST ──lower──▶ LLVM IR ──▶ binary
                                              │
                                              ▼
                                         proof report
```

`jetpack build` runs everything and inserts a runtime check at every
unproven site; `jetpack final` runs the same Precog and refuses the
program if any default-tier site is unproven (`core-features.md`). Both
run Precog in full — there is no build that skips it.

## Layer 1 — known values are executed

Everything that does not depend on the outside world is **known**: a
literal, arithmetic on literals, a call with known arguments, a loop over a
known range. Precog executes it at compile time with the language's exact
semantics (integers checked against their width, IEEE bins at their
width, `deci` exactly, text as text). The result is folded in. A program
that reads nothing is executed to the end and its binary is its output; a
check that would fail on known values is a **compile error at the line**
— the program would always fail there, so it never builds. Code that only
ever runs on known values is not in the binary.

Cost is counted in steps (one per AST node evaluated), never in time.

## Layer 2 — bounded values are tracked as sets

A value from outside the program has a **domain**: what its type allows.
`std::input.line[]` is any text; `to.int32[…] otherwise …` on it is any
`int32`; `to.int32[0, 100][…]` is `[0, 100]`; a `list[T][lo, hi]` has a
length in `[lo, hi]`; an `extern` result is anything its C-shaped type
holds. Precog runs the program abstractly over those domains: each
expression gets the set of values it can take, computed exactly through
every operation (`{1, 2, 3} x {10}` is `{10, 20, 30}`).

- A set larger than **4096 values** (provisional) becomes an interval
  `[lo, hi]` — sound, less precise; an interval is spelled out as a set
  again only at **64** values or fewer (provisional).
- Bins are intervals from the start. Text carries a length interval per
  unit. A list carries a length interval and an element domain. A struct
  carries a domain per field. An enum carries the set of possible cases,
  each with its payload's domain.
- **Control flow**: at an `if`, the condition's set decides which branches
  are possible, and each branch is analysed with the condition assumed —
  a guard `if 'n' <== 100 { … }` narrows `'n'` inside. `match` narrows the
  enum to one case per arm. `otherwise` splits: the success value's
  domain on one side, each failure's on the other.
- **Loops** iterate the analysis to a fixpoint, widening to intervals when
  a set keeps growing. A counted loop is followed **trip by trip** up to
  **100,000 trips per loop** and **2,000,000 across the program**
  (provisional; nested loops multiply), then widened.
- **Calls** are analysed per call site with the arguments' domains — the
  same per-shape specialisation generics and variadics already use —
  recursion depth-limited at **8** (provisional). So `square['i']` with
  `'i'` in `[1, 1000]` proves its own multiply, without a contract on
  `square`.
- **Ownership** is what Kespar lacked: every value has exactly one owner
  and lends are tracked by the checker, so Precog knows which memory a
  loop iteration can touch. That is the proof `parallelism.md` needs —
  two iterations that would conflict cannot both own or mutably lend the
  same value — and it is a fact about the program, not a domain.

At each check site Precog asks: is every value in the result's set legal?
Yes → **proven**, no check. No value legal → a **certain failure**, a
compile error. Otherwise → **unproven**, on to layer 3.

Check sites are: integer overflow (plain operators; `+~` etc. are never
sites), division and `mod` by zero, `[pos]` bounds, narrowing (`uint64`
into `int32`, a value into a range type), a length bound on a bounded
list after every change, `to.…` conversions that are not failures, and
the "must leave" rules the checker already enforces. Failures (`fail`) are
not check sites: they are values with domains.

## Layer 3 — the exact question

An unproven site is asked exactly, as a **bit-precise SMT query** over the
real program semantics — Z3 (provisional: through the `z3` crate). The
query encodes the path to the site — the domains, every assignment on
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
  without a counterexample. The budget is Z3's `rlimit` — a step count,
  never a timeout — **10,000,000** per query (provisional), so the answer
  is the same on every machine.

Unrolling to a bound means a site inside a loop may be proven for the
first 16 trips and unproven after. Kespar reported that honestly and
named the missing piece, decided here:

## Loop invariants — templates first, then unproven (decided 2026-09-22)

A loop Precog cannot run to the end (its trip count depends on input and
is too big or unbounded to follow) needs a fact that holds on every trip,
checked once — an invariant. Nobody writes one. Precog **guesses from a
fixed list of shapes** and keeps each guess the solver confirms
inductively (true before the first trip; true after a trip if true before
it): a counter stays within the range's ends; a sum is bounded by trips
times the largest element; an index never decreases and never passes the
length; a running max/min stays within the elements' domain; a count
never exceeds the trips so far. A confirmed guess is a proof, never an
assumption. If no shape fits or none is confirmed, the loop's sites are
**unproven** — working build checks them, final build asks for a guard or
`nocheck` — exactly as if no guessing existed. Guessing only adds proofs.

The programmer's way to supply a fact is the one the language already
has: a **guardrail** inside the loop (`if 'total' > 1_000_000 { fail
too_big; }`), which Precog reads on every trip; `check` in front makes
the compiler say whether it was enough. No invariant syntax. The
template list is provisional and grows by measurement (Kespar's bench,
the generator), not by guessing what programs do. Owner: "a and fallback
to c".

## The tiers, as emitted

- **proven** → plain instruction, and the proven range handed to LLVM as
  `nsw` / `nuw` / `!range` so it can optimise on it.
- **default, unproven** → working build: a checked instruction, trap on
  failure with the site named; final build: refused.
- **`check`, unproven** → compile error with the counterexample, or
  "solver gave up at line N; add a bound at line M".
- **`nocheck`** → plain instruction, no proof attempted, listed.

## `:=` widths

From layer 2's sets: the union of every value the name holds and every
expression it types; the narrowest width that holds it — then adjusted
for speed as `core-features.md` says (a scalar takes the machine's
natural width; list elements and struct fields take the narrow one). If
layer 2 only has an interval touching the 64-bit limit, layer 3 is asked
for the maximum (bounded binary search on "can it exceed K?"); failing
that, `:=` at that boundary is the compile error `core-features.md`
already specifies: write the type.

## Determinism

`reproducible-builds.md`, restated for Precog:

- No wall-clock anywhere; every limit is a count.
- Z3 is called with a fixed configuration and `rlimit`; results do not
  depend on thread scheduling.
- Sites are analysed and reported in source order.
- Compile-time arithmetic is performed at the exact width the target uses;
  the target is a description in `JetPack.toml`, never the host.
- Nothing is read at compile time except the source and the config: no
  files, clock, environment, network.
- The compiler's and solver's versions are in the report header.

## Reuse and parallel work

Precog's own work splits: sites are independent questions, call-site
specialisations are independent jobs, layer-3 queries run across all
cores. Results are **content-addressed**: a function's analysis is keyed
by (its bytes, its argument domains, the target), so an unchanged function
with unchanged inputs is not re-proven — reused, never skipped
(`core-features.md`, 2026-09-16). A shared cache of those keys is what a
team's build farm serves. Memory: must run a large program in 32 GB and
scale near-linearly with cores and memory on independent units (owner,
2026-09-21). GPU offload (`parallelism.md`) is a decision Precog makes
from the same independence proof.

## The report

Per check site, in source order:

```
line 7   'total' + square['i']   overflow           proven    (layer 2: max 333833500)
line 12  'xs'[pos 'k']           out of bounds      checked   (layer 3: sat, k=100 when len=100)
line 15  'a' / 'b'               division by zero   trusted   (nocheck)
line 9   square: 'x' x 'x'       overflow           proven    (layer 2, at 1 call site)
```

Three lists in the build output: `nocheck` sites, `check` sites passed,
default sites still checked (working build only; a final build has none).
`--types` adds every `:=` name with its chosen width and the value that
decided it. The Precog oracle (`compiler-verification.md`) reads this
report to know which claims to attack.

## What Kespar learned (2026-09-16..19)

- The sieve took 14.7 s of compile time until four engineering fixes
  brought it to 2.6 s: no state clones to read a condition, loop-exit
  states joined in place, exact sets reference-counted, ranges not spelled
  out past 64 values. Trip-by-trip following is where compile time goes.
- Z3 goes quadratic on nested loops past ~16 unrolls.
- Big lists are carried as a summary (length interval + element domain),
  not a value per element.
- None of the four bench programs needed Z3: layer 2 proved everything or
  the program folded to its output.
- The fuzzer found real Precog bugs the oracle caught — three in one
  commit. The generator and the oracle are not optional.

## Open

- The invariant template list: which shapes, measured against real
  programs.
- Every provisional number: set cap 4096 / re-set at 64, trips 100k /
  2M, recursion depth 8, unroll 16 / 64, `rlimit` 10M. Kespar's picks;
  to be measured, not guessed.
- What a text's domain is beyond a length interval (a set of literal
  values? a character class?).
- How `deci` and `uenr` are tracked (exact arithmetic on unbounded
  values: sets? intervals over rationals?).
- Whether the solver is Z3 (Kespar) or CVC5 / Alt-Ergo / several; what
  "fixed configuration" pins.
- The Precog oracle's shape (brute force over domains; how far).
- Interaction with `extern`: a C result's domain is its type's full range;
  is there ever a way to say more (a contract on an extern)? Today: no.
