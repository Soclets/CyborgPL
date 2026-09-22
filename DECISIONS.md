# CyborgPL v2 — Design decisions

Running log. Each entry: date, the decision, who proposed it, and who chose it.
Nothing here is final until the owner says so; open questions live at the bottom.

## Decided

### 2026-09-22 — The Precog oracle: exhaustive execution + independent interval analysis + re-encoded negation (owner: b + c)

Proposed (a) exhaustive/random execution over small domains (Kespar's
generator); (b) a plus an independent simple interval analysis
(attacks layer 2); (c) a plus the negation of each solver-proven site
through a second encoding (attacks layer 3). Claude first recommended
c; owner asked c vs b + c; given Kespar's bugs were all layer 2, Claude
shifted to b + c. Owner: b + c. Different author than Precog, no shared
code.

### 2026-09-22 — Layer 3 solver: a Z3 + CVC5 portfolio (owner: b)

Proposed (a) Z3 only, pinned; (b) a portfolio of Z3 and CVC5, both to
full budget, results combined; (c) Z3 behind an interface. Owner: b.
Determinism kept: never first-to-answer; any `unsat` proves; both `sat`
→ Z3's counterexample by fixed order; both `unknown` → unproven. A
`sat`/`unsat` disagreement is a solver bug and open as to what the
build does beyond reporting it.

### 2026-09-22 — Precog tracks `std::uenr` as exact rationals plus a digit-count bound (owner: b)

Proposed (a) intervals over exact rationals, no overflow site, memory
not Precog's business; (b) a plus a digit-count bound carried through
arithmetic, with a warning (never an error or a check site) when it
grows past a threshold; (c) opaque. Owner asked the a/b difference
(silence vs an early "this will eat memory" note on `'x' = 'x' x 'x'`
in a loop) and chose b. (Claude wrote bare `uenr` in the question; the
name has been `std::uenr` since 2026-09-19 — owner reminded.) `deci`
is an integer-like domain at its fixed digit count.

### 2026-09-22 — Precog's text domain: length + exact values + character classes (owner: c)

Proposed (a) a length interval only (Kespar), (b) plus an exact set of
values while small so `'cmd' == "quit"` narrows, (c) plus character-class
facts so a `to.int32` after a digits check is proven not to fail. Owner
asked the b/c difference (conversions after validation come out proven
vs keep a failure path) and chose c. Each text operation carries a rule
for its result's class; no rule → "any text". The class list is open.

### 2026-09-22 — Loop invariants: Precog guesses from templates, else unproven (owner)

For a loop Precog cannot run to the end: (a) guess invariants from a
fixed list of shapes (counter, sum, monotone index, max/min, count) and
keep those the solver confirms inductively; (b) a plus a programmer
hint — which Claude then withdrew, since a guardrail `if` inside the
loop is already the language's way to state a fact Precog reads; (c)
none — trip-by-trip and widening only (Kespar). Owner asked what
guessing changes: how many ordinary loops the final build refuses.
Owner: "a and fallback to c" — templates first; no fit or no
confirmation → unproven, as if guessing did not exist. The template
list is provisional, grown by measurement. `design/precog.md`.

### 2026-09-22 — `design/precog.md` opened, ported from Kespar (owner: "Let's go")

Kespar's three-layer method carried over as the starting point; every
number Kespar marked provisional stays provisional and is listed under
Open; what Kespar lacked (ownership as the parallel proof, failures as
domains, range types and bounded lists as domains, the final build,
LLVM range metadata, content-addressed reuse, the 32 GB / scaling
requirement) added from this repo's decisions. Questions follow one at
a time, starting with loop invariants.

### 2026-09-21 — Precog has a first design: Kespar (`~/Kespar`, 2026-09-16..19)

Not a decision; a correction of the record. Claude had said all day that
Precog was undesigned. The owner: "Kespar was built by your Precog docs."
`~/Kespar/design/precog.md` (149 lines) + ~14.6k lines of Rust: a
three-layer Precog — known values executed at compile time; bounded
values tracked as exact sets (≤4096) then intervals, loops to a fixpoint
with widening and trip-by-trip following (100k/loop, 2M/program),
functions inlined per call site (depth 8); what is left asked of Z3 as a
bit-precise query (unroll 16/64, `rlimit` not timeout) — plus a
separately-authored oracle, a program generator that found Precog bugs,
and a bench (sieve 14.7 s → 2.6 s of compile time after four engineering
fixes). No clock anywhere: same answer on every machine. Missing by its
own account: loop invariants. The POC is finished ("abandoned"); the
Cyborg Precog document starts from it. Its provisional numbers are open
questions here.

### 2026-09-21 — v1 public release includes GPU support (owner, direction)

Stated while discussing hardware and cross-compilation. Not designed.
Three readings, cost ascending: (a) GPU libraries through `extern`
(already possible in principle; a bindings Limb); (b) kernels written in
Cyborg, compiled via LLVM to PTX / SPIR-V / Metal, Precog proving the
index math; (c) automatic offload of a parallel `for`, the parallelism
pillar extended to the GPU, dependent on (b). Owner: **c**. So: the
programmer writes `for`, Precog decides CPU cores or GPU, `for seq` says
no — no GPU syntax in the language. (b) exists underneath as mechanism
(loop bodies compiled via LLVM to PTX / SPIR-V / Metal, a runtime that
launches and copies). Precog's offload test: iterations provably
independent (the existing parallel test), trip count worth the bus
transfer, body within what the GPU can do (no text, no `extern`, no
failure escaping the loop). Undesigned beyond this.

### 2026-09-21 — Three oracles: interpreter, Precog, translation validation (owner)

Owner asked how many oracles there should be. Claude: one interpreter
(the triangle docs / oracle / compiler closes), and if a third existed
it should be a different *kind* of check — a brute-force Precog oracle
(the one worth having), translation validation, or the program
generator (already decided). Owner: "Interpreter + Precog +
translation-validation seems good." Recorded as the intended set in
`design/compiler-verification.md`. Added the same day (owner): **twin
programs** — other languages as oracles for the parts they share. Benchmarks the same day: the oracle
(release) runs the 1000-digit pi spigot in 49 s vs CPython 0.49 s vs
C++/Rust 0.023 s — as intended; it is the spec, not the product.

### 2026-09-21 — Cold read 5 (`examples/inventory.cpls`): nothing changed; a wrong recommendation caught

A fresh chat traced the output correctly, praised the range types
("build the language around it"), failures in the signature, `bin64`,
and quoted names for keyword-proofing. It asked about `otherwise fail`
(passes the name on — decided), `std::` vs `list.`/`char.` (language ops
vs the Limb — decided, docs), unhandled `list.append`/`remove` (check
sites, the tiers — decided), `std::exit` in an arm (arms must leave —
decided), `>==`, brackets, 1-based, quotes (reader five). It called
`let mut 'best'` beside an unmarked owned parameter inconsistent; Claude
accepted that Rust-shaped assumption, recommended `mut` on parameters
and argued against "auto mut" — then checked: plain `let` and plain
parameters are Precog-inferred (2026-09-14), the oracle runs
`let 'x' := 1; 'x' = 2;`. Nothing inconsistent, nothing changed; a docs
line added. Lesson recorded in memory: verify in DECISIONS/design and
the oracle before recommending. Also noted: the example's
`heaviest[...copy 'shelf']` teaches an expensive pattern — the example's
fault, not the language's.

### 2026-09-21 — Extern data: `extern let ['mut'] 'x': T;` (owner)

Open since the FFI decisions. Proposed: `extern let` mirroring `extern
fn` (reads and writes nocheck sites, C-shaped type, no value); no extern
data, only C helper functions; `extern` on a `ptr` read with `ptr.read`.
Owner: a. Noted: `errno` is a macro on macOS/Linux, so std will offer
`std::c.errno[]`; `extern let` is for plain globals.
`examples/extern_data.cpls`, five reject tests.

### 2026-09-21 — `char` conversions: `char.code` / `from_code`, text → char fails, units down fail (owner)

Three questions. (1) Number ↔ char: `to.int32[`7`]` gives the code (55,
the C trap), the digit (7, a parse in disguise), or is refused with two
named operations — `char.code[…]` and `char.<unit>.from_code[…]
otherwise …` (fails `not_a_char`), digit value via text — owner: c. (2)
Text → char: a failing conversion `to.char.point["A"] otherwise …`
(`not_one_char`), or only `std::str.at` — owner: a. (3) Between units:
down fails (`not_one_point`, `not_one_byte`), up always works, or all
refused — owner: a. Oracle fix on the way: a char literal now takes its
declared unit (`let 'g': char.grapheme = `é`` was staying a point).
`examples/chars.cpls`, four reject tests.

### 2026-09-21 — A module and a type cannot share a name (owner)

Proposed: refuse (one name, one thing, per Limb) or allow and tell apart
by what follows the dot. Owner: a, refuse. Covers module blocks and file
modules, against structs, enums and interfaces. Reject test
`module_type_clash`.

### 2026-09-21 — LLVM through `inkwell`, then `llvm-sys`, then the C++ wrapper; version follows inkwell (owner)

Owner asked how the language is built and in what (recap of the
2026-09-14 pipeline: Rust front end, checker + Precog, lowering via the
LLVM C API, LLVM, system linker; `no_std` Rust runtime; std in
CyborgPL; not self-hosted), then inkwell vs llvm-sys: safety difference
is in the compiler's own code only; inkwell 0.10 is on LLVM 22.1 while
LLVM 23.1.1 and llvm-sys 231 are out; the drop-down to llvm-sys reaches
unwrapped C-API calls on the *same* LLVM, not a newer one. The 22 → 23
changes that touch Cyborg: branch-opcode C-API change (inkwell's), IR
byte type (unused), AArch64 clobber bug for `x29`/`x30` written as `xN`
— dodged by renaming to `fp`/`lr` in the asm lowering, as Clang does.
Owner: record it. `design/implementation-languages.md`, `design/ffi.md`.

### 2026-09-21 — bin → int fails unless whole: `not_whole` (owner)

Open since 2026-09-19. Proposed: fail unless whole (rounding said first,
`to.int32[std::round['x']]`), truncate (C, quiet), floor (matches `/`,
quiet). Owner: a. The rounding toolbox — `std::round`, `floor`, `ceil`,
`trunc`, a bin to a whole bin — is std's; names provisional.
`examples/whole.cpls`.

### 2026-09-21 — `list.remove` hands the element back; no `give` of an element (owner)

Surfaced while explaining `...give`: the oracle had refused
`give 'xs'[pos i]` since functions-as-values ("it would leave a hole")
without the owner ever being asked — Claude's default, now put properly.
Owner asked what giving an element is for (moving a big owned element
into a function without a copy), why no hole (every list access in the
program would carry an "is it still there?" check; a marker that keeps
positions is an `enum` element's job), whether shrinking does what a
hole can (all but keeping positions), and whether a `give` that shrinks
differs from a `list.remove` that hands back (no — two spellings).
Owner: `list.remove` hands. Bare `list.remove[…];` still just drops it.
`list.append` as a value is refused. `examples/queue.cpls`.

### 2026-09-21 — Owned elements through `...`: say the verb — `...copy 'xs'` / `...give 'xs'` (owner)

Proposed: always copy (safe, a copy per element), always give (moves
out, empties the list), or say it after `...`, as every owned hand-over
in Cyborg says its verb. Owner: c. A `give` empties the whole list —
never holes. No verb into a `lent ...` lends each element; plain
elements take no verb. `examples/spread.cpls`.

### 2026-09-21 — `...'xs'` unpacks a list into a variadic call; only where its length is known (owner)

Open since 2026-09-19. Owner first asked what the line even does
("unpacks the list: `sum[...'xs']` becomes `sum[1, 2, 3]`"), confirmed
`sum` is an ordinary variadic function, then said yes. Follow-up: a list
whose length Precog cannot see (from input, no bound) — (b) a compile
error at that call, "give it a bound or pass it as a list", or (c) the
compiler quietly builds a list-taking version. Owner: b. Forwarding a
bundle to another variadic is the same mechanism (a bundle's count is
always known) and closes that older open item. Unpacking owned elements
into an owning variadic (copy or give?) is left open; the oracle refuses
it. `examples/spread.cpls`.

### 2026-09-21 — A length bound on a list: `list[T][lo, hi]` (owner)

Opened from the `uint64` decision: how to tell Precog a length it can't
see. Three placements proposed — `list[int32][0, 1000]` (the thing, then
its bounds, as `int32[0, 100]`), `list[int32, 0, 1000]` (mistakable for
three things), `list[int32] length[0, 1000]` (a word). Owner: `a`. Claude
applied the existing rule for what follows — every change to a bounded
list is a check site, its length is `uint64[lo, hi]`, low bound ≥ 0, no
bound on `array` — and asked the owner to confirm. `examples/bounded.cpls`.

### 2026-09-21 — `list.length` / `array.length` are `uint64` (owner)

Open since 2026-09-17: the number's type. Proposed `int32` (matches
counters; lies past 2 billion), `uint64` (a count: never negative, never
too big; narrowing into `int32` is a check site Precog proves wherever it
can see the list) and `int64`. Owner asked whether Precog makes the
choice moot — it makes the check free in the visible case, and the final
build may store a proven-small length however it likes; the written type
is for the reader and the unseen case. Owner: "`uint64` it is". For the
unseen case (a list from input) the owner asked for a way to state the
bound: a guard `if` already works; a length bound on the list's type
(`list[int32][0, 1000]`) is opened as the next question.

### 2026-09-21 — `&` / `&mut` as alternatives to `lend` / `lend.mut` (and `lent` / `lent.mut`); `printu` kept, long alias added (owner)

Four cold readers in a row tripped on `lent` vs `lend` and called `printu`
cryptic. Claude laid out the cost of each (one letter apart; a made-up
word) and asked what, if anything, to change. Owner: keep `printu`, add
`std::this_will_print_the_useful_numbers.this_will_go_to_the_standard_output`
as its alternative (the `…standard_error` twin follows); add `&` / `&mut`
as alternatives to the lending verbs. Both spellings are equal — `&'s'`
is `lend 's'`, `&mut 's'` is `lend.mut 's'`, `&'x': T` in a parameter is
`lent 'x': T`, `fn[&T]` is `fn[lent T]`. Binary `&` between two values is
still an undefined custom operator. Every `&` / `&mut` makes the build
print `What are you? Rust programmer or Claude?` on stdout — one line
per `&` — then carry on (owner: "You're not supposed to use that"). `examples/alternatives.cpls`.

### 2026-09-21 — Cold read 4 (`examples/shop.cpls`): lends may escape (kept), three notes (owner)

A fresh Claude chat read the shop program: the call-site verbs are "the
thing to build around", the failure system and `pos` earn their place;
it asked whether a borrow can escape a call, what arithmetic does to a
range type, how failures propagate, where `<T>` is introduced, whether
interfaces are structural, and why mutation is unmarked — all decided,
mostly not visible in one program. It misread `int32[0, 100_000]` as a
generic instantiation (a cost, logged), claimed there is no char literal
(there is: backticks), and called the quotes "two shifted keystrokes" —
they are unshifted (owner). Recurring for the fourth reader: `lent`/`lend`
one letter apart; `printu` cryptic.

Claude proposed adopting "a lend never outlives the scope that made it"
(no lifetime annotations ever; a returned view becomes a copy Precog
elides where it can) and, asked why it was trying to change it, admitted
the reason was that the oracle already behaves that way. Owner: "Keep it
as it is." So: views may escape, Precog infers lifetimes, the optional
annotation's spelling stays open, and **the oracle's refusal to return a
lend is a known gap, not the design.**

### 2026-09-21 — Callbacks: capture-free, C-shaped functions cross to C; `fn[…]` in extern signatures (owner)

Owner asked what callbacks are for (sorting, events, signals, servers).
(1) what may cross: only capture-free functions (a), over smuggling
captures via userdata (b) — owner asked what (b) was for, chose a.
(2) `fn[…]` in an extern signature, C-shaped, by value, no fail; a C
function pointer received is a `fn[…]` value called under `nocheck` —
yes. Consequence: `ptr[T]` allowed in any fn signature. Oracle emulates
`qsort` with the callback (`examples/callbacks.cpls`), six reject tests.

### 2026-09-21 — Functions as values: bare name, `fn[…] -> T` types, `with [lend 'v']` captures (owner)

Owner asked what it is for (sorting by a rule, callbacks, map/filter, C
callbacks). (1) a bare name is the function as a value, `'f'[…]` calls it
— yes. (2) the type is the signature without the names, verbs included —
owner asked how it compares to a normal fn (names and body removed) —
yes. (3) named-only / inline-without-capture / inline with explicit
`with [lend 'limit']` captures — owner: c. `design/functions-as-values.md`;
oracle: `examples/functions_as_values.cpls`, ten reject tests; `lend
'xs'[pos i]` added on the way. Closes open item 11.

### 2026-09-21 — Bitwise: `and.bit`, `or.bit`, `not.bit`, `xor`, `shl`, `shr` (owner)

Owner asked why the spelling extends from `and`/`or`/`not`, then what
"keep a 1 only if both have a 1" meant, then how that relates to `and`
— walked through column by column; "`and.bit` it is." No precedence
(parenthesise), integers only, shifts are check sites.
`examples/bitwise.cpls`; the FNV-1a hash example now uses `xor` and
matches the real algorithm (1335831723 for "hello"); six reject tests.

### 2026-09-20 — Input: `std::input.line` / `.all` failing with `closed`; range types `int32[0, 100]` (owner)

(1) reading: two std functions returning `str.utf8`, `closed` on nothing
left — yes (owner caught Claude's nested `otherwise` inside a bracket,
which the whole-right-hand-side rule forbids; two lines instead).
(2) how Precog learns an outside value's range: guardrail `if` / bound on
the conversion / range types; owner noted `int32` is already a bound;
chose **c, range types**. (3) spelling `int32[0, 100]` — yes. (4) two
ways in: assignment (a check site) and `to.int32[0, 100][…]` (fails
`out_of_range`) — yes. `design/input.md`; oracle: `examples/input.cpls`,
`examples/ranges.cpls`, ten reject tests. Closes "input" and "boundary
range annotations".

### 2026-09-20 — Wrapping arithmetic: `+~ -~ x~ xx~` (owner)

Homes: std functions / operators / a wrapping type; Claude recommended
operators (the floor-vs-`div.trunc` logic: the default operator is the
safe meaning, the other has its own spelling) with Zig's `+%`; owner:
"`%` is percent, not modulo" — Claude offered Swift's `&+` or a tilde;
owner: "`&` reads like borrow" — "`~` it is." `examples/wrapping.cpls`
(FNV-1a with no `nocheck`), four reject tests. Surfaced: no bitwise
operators (xor, and, or, shifts) exist yet — logged open.
Owner then floated `and or not` → `& / !` and `/` → `div`; Claude: `div`
and `!` fine, `&` has the borrow reading the owner had just rejected, `/`
for or would be misread by everyone; offered two coherent sets. Owner:
"Nvm, back to normal." Unchanged.

### 2026-09-20 — Interfaces: `interface`, operators definable for user types, automatic satisfaction + `satisfies` claim, `<T: Ordered>` (owner)

"How do you say what your generic supports?" — you don't yet; owner:
"now". (1) may a type define the language's operators for itself — yes
(revises the 2026-09-19 no-redefinition rule: never for the language's
types). (2) `interface Name { fn …;  op …; }` about some `<T>`, and
`<T: Ordered>` on one mention — yes. (3) automatic / declared / both:
owner asked twice what the options meant, chose (c) automatic plus an
optional checked claim; asked what `is`, what `Ordered`, what the claim
line are; worried `is` reads wrong with other names — Claude offered
`satisfies`; "`satisfies` it is." Consequence set by Claude: verbs on a
`<T>` value are allowed and do nothing for a plain type, so generic code
can `return give 'a'`. `design/interfaces.md`; oracle:
`examples/interfaces.cpls`, nine reject tests. Closes "generic
constraints".

### 2026-09-20 — Generics: `<T>` on every mention; struct/enum blanks by first appearance (owner)

Claude offered a declared list (`fn max [type T] […]`) vs implicit; owner
proposed `fn max ['a': <T>, 'b': <T>] -> T`, then "both use `<>`" — every
mention. For structs, Claude proposed a bracket after the name for order
(`struct Pair [<A>, <B>]`); owner asked why, then "how's it without
generics?", then "why does it need `[<T>]`?" — it doesn't; owner: "no
bracket, first appearance." `design/generics.md`; oracle:
`examples/generics.cpls`, eight reject tests. Closes the generics
spelling item; constraints are open.

### 2026-09-20 — Custom operators: `op`, words and glyphs, any run, no precedence, lent-or-`give` operands (owner)

Owner: "How do you make a custom operator?" — not yet; then "one by one".
(1) words only vs glyphs too — owner: glyphs too. (2) the characters
`! % & * + - / < > = ? @ ^ | ~`; Claude proposed a three-character cap;
owner: `<+>-` is just an operator, a negative goes on the operand — a
maximal run is one operator, no cap. (3) precedence — owner: "Require"
(parentheses whenever a custom operator meets another). (4) operands —
Claude proposed always-lent; owner asked the difference, then how Rust
behaves (by value; four impls), then which has better performance
(by-value can reuse a dead operand's buffer); owner: "middle" — lent by
default, `give` on an operand takes it by value and the caller writes
`give`. `design/operators.md`; oracle: `examples/operators.cpls`, thirteen
reject tests. Closes the "custom operators" open item.

### 2026-09-20 — The borrowed prints are removed (owner)

"Remove `std::<lang>.<prints>`." `std::c.printf`, `std::c.fprintf.stderr`,
`std::rs.print` / `println` / `eprint` / `eprintln` — decided 2026-09-19 as
a joke feature, gone a day later. `print` and `printu` are the prints.
The oracle refuses the old names by name; examples and tests removed.

### 2026-09-20 — `/` floors, `mod` is the mathematician's; truncation is `std::div.trunc` / `std::mod.trunc` (owner)

Owner asked the difference (negatives only: `-7 mod 2` is `1` floored,
`-1` truncated — the clock/hash bug), chose floor, then "should we leave
hardware that to std?" — yes. Also confirmed ("yea") the final-build
understanding: working build keeps runtime checks at unproven sites, the
final build makes them compile errors, nothing unproven ships. Oracle
switched from its day-one provisional truncation; `examples/division.cpls`.

### 2026-09-20 — The final build has no runtime checks: an unproven site is a compile error there (owner's clarification of 2026-09-14)

Owner, on being told the unproven runtime checks ship: "Oh shit, that's
not what I meant" — "'runtime checks stay, no matter the cost' — that was
when building around, dev testing, not the final product." So: two
builds, the same full Precog, differing only in what an unproven site
becomes. Working build (`jetpack build`, proposed): a runtime check, so
the program runs and crashes loudly where a guardrail is missing. Final
build (`jetpack final`, proposed): a compile error with the
counterexample — the default tier behaves as `check`. Nothing unproven
and nothing runtime-checked ships; `nocheck` remains the only way to ship
an unproven site, listed in the build output. Not the rejected
development mode: nothing is skipped, the final build is stricter. The
2026-09-14 tiers entry is corrected below rather than appended to.
Claude's understanding was restated as a table and awaits the owner's
"yes"; the build-command names are Claude's proposal, JetPack's, so no
file-wide switch enters the language.

Owner confirmed the next day ("yea"); `/` and `mod` decided then too.

### 2026-09-20 — Enums: `enum Status { empty  claimed: str.utf8  locked }`, `Status.claimed["x"]`, `match` (owner)

Owner asked "didn't we already do enums?" (no — failures chose inline
lists, leaving item 4 open) and "what are enums?" (one of a few options,
like `bool` with your own names). Declaration shape and `enum` keyword —
yes. Branching: the `otherwise` arms with a word in front; owner asked
for a small program, then "why `match`?" — the value is matched against
shapes; `switch` has fall-through baggage, `when` reads as time. "`match`
it is." Consequences set by Claude: `match` is a statement; an enum is
always owned like a struct; `==` works. `design/enums.md`; oracle:
`examples/enums.cpls`, thirteen reject tests. Closes open item 4.

### 2026-09-20 — Text: three counts, no length; walk by unit; no `[pos]`; toolbox in std; exact `==`; no `<` (owner)

Four questions. (1) length: no `str.length` — `str.bytes` / `str.points`
/ `str.graphemes` (owner asked "what?", got the emoji example, yes).
(2) walking: `str.each.<unit>[…]`, no bare `for` over a text — yes.
(3) the toolbox (`split`, `trim`, `upper`, …) is std's, `==` compares
bytes exactly, `<`/`>` on text is an error — yes. (4) no `'s'[pos 3]`;
`std::str.at.<unit>` with a failure — yes. `design/strings.md`; oracle
enforces the language half and emulates the std half
(`examples/text.cpls`, seven reject tests). A `char` now compares by
content across units (Claude, needed for the example).

### 2026-09-20 — Inline assembly: a statement, raw braces, arrows for direction (owner)

Three questions: statement vs value — owner: statement (option 1); the
block — Claude first wrote a list of strings (Rust's shape), owner: "I
don't think Assembly looks like that" — raw verbatim braces; bindings —
Claude proposed words and arrows both, owner asked what the arrows were
for, then: "drop the words, keep the arrows." Then the head: owner
proposed `nocheck inline_code.ASM`; Claude offered house style
`inline.asm` (no underscore, lowercase) and the `inline.<language>` family
(`inline.C` later via clang); owner took `inline.` but kept the capitals —
"the inline code name should be easy to spot, just like `MAIN`" — so
**`nocheck inline.ASM { … } with ['v' -> x0, 'n' <- x0, clobber x16];`**.
In `design/ffi.md`; oracle parses/checks and refuses to run.

### 2026-09-20 — FFI in the oracle (Claude, per yesterday's decisions)

`extern fn`, `ptr[T]`, `nocheck` on extern calls and pointer reads/writes,
`ptr.is_null`, `layout.c struct` with C-shaped fields, variadic externs,
`size.of`/`align.of` — all enforced; libc emulated for six functions so
`examples/ffi.cpls` runs. Nineteen reject tests. `asm` refused as
unspelled. Surfaced: a `str` has no length operation yet (string
operations, open).

### 2026-09-19 — Low level is the base language's: `extern`, `ptr[T]`, `layout.c`, `size.of`, inline asm (owner)

Owner asked how to print without std, then how std's print gets made
(intrinsics → runtime → syscall), then — "I don't wanna rush (like Xag)"
— asked what `extern` is; Claude explained from "a program is a pile of
named functions" up. Then one by one: the OS door is open to every Limb,
not std-only ("Syscall and lower-level stuff should be able to do, using
the base Cyborg language"); asm is in too. (1) `extern fn name […] -> T;`
— yes. (2) pointers: `ptr[T]` only in extern signatures, a lend satisfies
it — owner: a. (3) an extern call is a `nocheck` site — "yes, reuse
`nocheck`". (4) a returned pointer: hand back only, or read through —
owner: **b** (read/write through), against Claude's recommendation of a.
(5) `ptr.read/write/at`, `nocheck` sites — yes. Missing features listed;
`ptr.is_null` — yes; `layout.c struct` — yes; variadic externs — yes;
`size.of`/`align.of` — yes (owner asked what size and alignment are).
Raw allocation: none in the language (`list[uint8]`, or `malloc` as an
extern). `design/ffi.md`. Not yet in the oracle. Open: asm spelling,
callbacks, extern data.

### 2026-09-19 — `std::uenr` renamed `std::uenr`, Unbounded Exact Numerical Representation (owner)

Rename only; semantics unchanged. Owner also set the focus: the base
language before std.

### 2026-09-19 — Variadic parameters: `...'name'`, compiled per call-site shape (owner)

Model: (a) a list (needs a list-of-anything) vs (b) a fixed set per call
site, specialised by Precog; owner asked "wouldn't we eventually need
generics?" — (b) is the same machinery; owner: b. Spelling: Claude offered
`many`; owner: could be confused with `array`; Claude offered `...` /
`vararg` / `rest`, recommended `...` (the ellipsis, universal); owner
asked why; "`...` it is." Owner asked about two types (`: int32, bin16`)
— no: untyped keeps each argument's own type, the body constrains; a
union type is not a thing yet. Oracle: `examples/variadic.cpls`, four
reject tests. Closes the "variadic parameters" open item.

### 2026-09-19 — Borrowed prints live under their language: `std::c.printf`, `std::rs.println` (owner) — removed 2026-09-20

"`std::<lang, only if it isn't Cyborg>.<that lang's print(s)>`" — so
`printc` → `std::c.printf` / `std::c.fprintf.stderr`; `printrs` →
`std::rs.print` / `println` / `eprint` / `eprintln` (Rust's own names, and
`println` adds the newline as Rust's does). Owner also asked whether the
values inside could go unquoted; Claude: it would need variadic
parameters (undecided, and needed for std's own `print`) and a
per-parameter `bare` flag, which makes call parsing depend on the callee
and adds a second dialect; recommended variadics soon, `bare` shelved.
Owner: "yup." Open: variadic parameters; `bare` (advised against).

### 2026-09-19 — `std::printrs`: Rust's `print!`, exactly, with checked holes (owner) — removed 2026-09-20

"Now, `printrs`, exactly like Rust's." `{}`, `{0}`, `{name}` from scope,
`{:?}`, fill/align/sign/#/0/width/precision/type, `{{` `}}`, `\n` inside
the format. Oracle: `examples/printrs.cpls`, five reject tests. Owner then
asked whether `printc`/`printrs` could be called with `( )` (a Limb-granted
ability); Claude advised against — two call spellings, and `( )` is
arithmetic grouping only by design; owner: "stick with `[]`, at least the
insides look alike." Closed.

### 2026-09-19 — `std::printc`: C's `printf`, exactly, with checked holes (owner) — removed 2026-09-20

Owner asked what C's print looks like and what it would look like here,
then "I kinda feel like adding `printc`"; Claude proposed a checked subset
with the newline as a trailing value; owner: "`printc` is exactly like
C's" — so `\n` inside the format is `printc`'s to interpret, the base
string rule untouched. Precog checks holes against values (a mismatch is
a compile error). Oracle implements d i u x X o f e g s c b %%, flags,
width, precision; `examples/printc.cpls`, five reject tests. The base
language has no print at all (owner asked; it is std's, a boundary).

### 2026-09-19 — Strings have no escapes but `\"` and `\\`; the newline is the bare `\n` outside (owner)

Owner asked whether `"\n"` prints literally — the oracle's lexer had
C-style escapes by default. Owner: "That's why I made it outside" — the
bare `\n` piece is the newline; then: "escape in double quotes uses `\`,
but only for what needs that when inside" — `\"` and `\\`. Anything else
after a backslash is literal. `examples/strings.cpls`.

### 2026-09-19 — A keyword may name a function, reached by path only (owner)

Owner: "`std::to` wouldn't be ambiguous, even with `to` being the lang's,
right? it has `std::`." Claude: correct from outside; inside its own Limb a
bare `to[…]` would collide, so the rule is "reached through a path, never
bare" — `std::to[…]` outside, `limb.to[…]` inside. Owner proposed `std:to`
then `std:::to` for the inside; Claude: `:` is taken and `limb.` already
means this. Owner: "Yup." Functions only. Oracle enforces
(`examples/keyword_fn.cpls`, `keyword_fn_bare` reject).

### 2026-09-19 — Explicit conversion: `to.int32[…]`, `to.str.utf8[…]`; `++` takes text on both sides (owner)

Cold read #3 flagged `++` converting a number implicitly — the second
reader to do so. Claude offered (a) keep it or (b) make it explicit with a
types-as-calls spelling (`int32["42"]`); owner asked what
`int32["42"] otherwise 0` was, then why not `std::to.int32`; Claude: the
compiler owns conversion, and text→number failing uses the failure model.
Owner: "b, `to.int32[…]`?" — a bare keyword head, like `list.length`.
Consequences set by Claude: `++` demands text on both sides; text→number
fails with `not_a_number`; narrowing is a check site; encoding always
spelled; stored form. `design/conversion.md`; oracle enforces (nine reject
tests); every example converted. Closes open item "explicit conversion
spelling"; `std::exit[…]` is also now a leaving statement after
`otherwise` (cold-read consequence).

### 2026-09-19 — Failures: a value the caller must handle; `-> T or fail[…]`, `fail name;`, `otherwise` (owner)

Owner asked what the base language needs; Claude ranked error handling
first. Five questions: (1) model — exceptions / failure-as-value /
sentinel; owner: **failure as a value** (b). (2) what it carries — nothing
/ a name / a name and a payload; owner: **name and payload** (c). (3) where
the reasons are declared — inline per function / a named `enum`; owner
asked what an enum is, then: **inline** (option 1), so no enum yet. (4)
the function's side: `-> bool or fail[full, bad_item: str.utf8]`,
`fail full;`, `fail bad_item["x"];` — owner: yes. (5) the caller's side:
a fallback value, a leaving statement (owner asked "isn't it `return
false`?" — both exist), `fail` to pass up, or arms with every reason
covered; owner: yes. Claude then caught that `or` collides with the
boolean operator; candidates `else`/`catch`/`otherwise`/`onfail`; owner:
**`otherwise`**. Full text in `design/failures.md`; oracle enforces
(`examples/failures.cpls`, 18 reject tests).

### 2026-09-19 — Text joins with `++`; the whitespace join is gone; `print` takes many values (owner)

Owner: "Feeling like adding `++` for strings connecting." Claude asked
whether it coexists with the whitespace join or replaces it; owner asked
"is our current method implicit?" — yes, the space was the operator, and
the number→text conversion was implicit too; owner: "we'll make it
explicit." Glyph shortlist (`++`, `..`, `...`, `<>`, `&`, `~`, a word such
as legacy's `stch`); Claude recommended `++`; owner: "`++` it is."
Consequences Claude set and flagged: precedence below `+ -`, above
comparisons, left-assoc; at least one side text, the other rendered in
stored form (the 2026-09-14 "number piece → stored form" rule survives,
the "bare number is never a piece" rule is retired since nothing is
ambiguous now); two values side by side are an error; `print`/`printu`
take any number of comma-separated values and render each — the only way
`printu` can still show the useful form of a number, since a `++` join has
already fixed the text. Oracle, examples (`examples/join.cpls`), tests,
docs converted. The "whitespace: separator or implicit space?" question
closes with it.

### 2026-09-19 — Field names are bare: `struct Locker { number: int32 }`, `Locker[number = 7]`, `'l'.number` (owner)

A cold read (a fresh Claude chat shown `examples/lockers.cpls`) caught the
inconsistency: fields were quoted in the declaration and the construction
but bare at access. Claude offered (a) quote everywhere (`'l'.'number'`)
or (b) bare everywhere, and recommended (b); owner asked "is `number` a
var?" — no, a part of one: the quotes mark the thing you hold (`'l'`), the
route into it (`.number`, `[pos 2]`) is bare. Owner: "b". The one-line
rule is now: *quotes mark a value you can hold — a variable or a const;
names reached through a dot or a Limb — functions, modules, fields — are
bare.* Oracle, examples, tests converted; the quoted spelling is refused
with a message.

### 2026-09-19 — The ladder is default → `check`; `nocheck` is the exit, not the last rung (owner)

Owner's original picture: build guardrails, then write `nocheck` to remove
the check. Clarified against the 2026-09-14 tiers entry ("proves what it
can and inserts a runtime check wherever it can't"): once a guardrail is
good, Precog removes the check by itself, with or without `check`;
`check` only adds "refuse to compile and show the counterexample if it is
not enough"; `nocheck` is for the site no guardrail can make provable.
Owner: "if that's better, then yes. Not the original one." A proven check
can never fire, so keeping it is pure runtime cost. Build output lists
`nocheck` sites **and passed `check`s** (owner, 2026-09-19). Claude also
proposed listing default-tier sites still carrying a runtime check ("where
to add a guardrail") — not yet answered. **No alias for `nocheck`** (owner,
2026-09-19): a `*` shorthand was raised for codebases dense with it; Claude
advised that `nocheck` should stay long and rare (Rust's `unsafe` is ugly
on purpose) and that `nocheck { … }` covers the honest dense case; owner
closed it.

### 2026-09-19 — `check` / `nocheck` spelled: a word in front of a statement (owner)

Owner asked for the tiers again, confirmed they are never global, asked
for a syntax; Claude proposed one rule — the word before a statement,
covering its check sites; a `{ }` block is a statement so `check { … }`
needs no second rule; innermost wins; never on an expression, a `fn`, or
the top level — and walked through a guardrail as a plain `if` before the
site. Owner: "So, `check` & `nocheck` at the front?" — yes. Oracle enforces
(`examples/check_tiers.cpls`, seven reject tests); the oracle cannot prove,
so a firing `check` reports the counterexample value and a firing
`nocheck` reports the lie. Only `nocheck` sites are listed in build output
(Claude's pick, flagged).

### 2026-09-19 — Modules (owner)

Owner asked what Rust's `pub()` is, then what modules are, then "Yeah, we
should have modules." Five questions, one at a time:
1. File-as-module (a) vs `module` block (b) vs both (c): owner asked why not
   (b), what (a) misses, what (c) is — chose **(c)**: every `.cpls` is a
   module named after it (directories nest) and `module name { … }` nests
   a block. Keyword **`module`** — owner caught that `mod` is the remainder
   operator.
2. Root: **`limb.cpls`** (owner chose (a) over "no root"; asked about the
   `.limb` archive and kept the name).
3. Names: **"whatever has most compat"** — lowercase ASCII identifier, no
   Windows device names; a bad file name is a compile error.
4. Middle audience: **`pub.limb`** (owner chose the chain over `pub(limb)`,
   which would break "`( )` is arithmetic grouping only", and over no
   middle level).
5. Reaching the root from a module: **`limb.helper[…]`** (owner chose (c)
   over lexical fallback and over the Limb's own name).
Consequences written into `design/limbs.md`: bare = this module, dotted =
relative, `limb.` = absolute; visibility checked along the path; file
modules provisionally public. Oracle runs Limb directories
(`examples/geometry/`), blocks (`examples/modules.cpls`), 20 reject tests.

### 2026-09-19 — No public variable (owner)

Asked twice how to make a public variable; Claude: none exists, and laid
out what one would be for (configuration knobs, shared counters/caches,
singletons) and the owned shape of each (parameter or struct field; a
value lent mutably; what `MAIN` owns). Owner: "No public variable." A Limb
exposes `pub const`, `pub fn`, `pub struct` only. Closed. In
`design/limbs.md`.

### 2026-09-19 — `pub`; a Limb's top-level value is `pub const 'name'` (owner)

Owner asked how to declare a public variable. Claude: a Limb has no
top-level variables yet; proposed `pub` before `fn`/`struct`/`let` with a
top-level `let` immutable only. Owner asked what top-level variables are
for; Claude: named constants (legitimate), configuration (a boundary
value), mutable program state (conflicts with ownership). Owner: "So, it's
`pub const`?" — yes; and the name is **single quoted** (`'pi'`,
`mathstuff::'pi'`), owner's choice over a bare function-like name. Rules
in `design/limbs.md`; oracle enforces (`examples/consts.cpls`, twelve
reject tests). The author's "reachable bare" flag stays open.

### 2026-09-19 — No `import`/`use`; the Limb maker decides bare reachability (owner)

Asked the difference between `import` and Rust's `use`; Claude: `import`
loads and binds, `use` only shortens a path that already works — and Cyborg
already sits on Rust's side (JetPack.toml lists, `::` reaches). Owner:
whether a Limb's items may be used bare is **the Limb maker's decision**,
stated in the Limb's source; a user cannot alias or override — "if you want
to override that, you edit the source code yourself." Spelling of the
author's choice is open, alongside `pub`. In `design/limbs.md`.

### 2026-09-19 — Syntax revision: `let`/`fn`, type after the name, `[ ]` only where it builds, bare control flow (owner)

The owner opened the day with "I'm thinking of changing the syntax to
Rust-style: `let 'x' := [1000]; let 'x':bin16 = [1000];`" and took it
through ten questions, one at a time. Everything not listed here stands
as decided before (quoted names, `:=`/`=`, the verbs, `::`/`.`, `[ ]`
semantics, operators, `MAIN`). Full spelling in `design/syntax-sketch.md`,
`functions.md`, `control-flow.md`, `structs.md`, `ownership-syntax.md`.

1. **`let`**, not `var` (owner). Claude had recommended keeping `var` —
   "says what it is"; the owner's own 2026-09-17 objection to `let`
   (`let.mut.bin16`) dissolves once the type leaves the keyword chain, and
   the quotes already mark variables.
2. **The type follows the name after `:`** — `let 'x': bin16 = 1000;` (owner's
   proposal, Claude recommended it). Touches only the five name-to-type
   sites: variables, parameters, return type, struct fields, loop counters.
   Types themselves keep their dot chains (`str.utf8`, `list.int32` — bracketed later that day,
   `std::uenr`).
3. **`mut` / `immut` before the name**: `let mut 'x': bin16 = 1000;`,
   struct fields `immut number: int32` (owner chose (a) of three; Claude
   recommended it).
4. **Parameters: verb before the name, owning parameter bare** —
   `fn count [lent 'text': str.utf8]`, `['a': int32]`, `[mut 'a': int32]`
   (owner chose (a) over Rust's type-position `'text': lent.str.utf8`, after
   asking how Rust does it; Claude recommended (a) for the def/call mirror).
5. **`fn`**, not `func` (owner, unprompted, with 4).
6. **Return type `-> T`** after the parameters: `fn add ['a': int32, 'b': int32] -> int32`
   (owner chose Rust's arrow over `: int32`; Claude recommended the arrow —
   `:` attaches a type to a name, and a result is not the function's type).
7. **`[ ]` only where it builds** (owner: "calm that `[]` spam down";
   Claude proposed the cut, owner accepted). Brackets stay for text joins,
   lists, call arguments, struct construction, index, `range[…]`; a single
   expression stands bare on the right of `:=`/`=`/`return`, in conditions,
   as a struct field's value. Pieces outside `[ ]` are an error. Supersedes
   the 2026-09-14 "mandatory in every value position" and "a stored call
   wears the outer bracket".
8. **`else if`**, **`while`**, **`for`** — bare (owner, three questions;
   Claude recommended each). `loop { }` stays.
9. **One `for`; `range[start, stop, step]` is what it runs over**;
   the counter is typed on its name (`'i': int32`); the per-bound `:=`/`=`
   markers are gone (owner asked why, accepted "it would say the same thing
   twice"). `range[…]` as a standalone value is undecided.
10. **Modifiers bare, before the name**: `for seq perm 'i' in range[1, 10]`
    — `seq` first, then `temp`/`perm` (owner chose (b) over `for.perm.seq`;
    Claude recommended (b)).
11. **`MAIN { }` stays** (owner; Claude recommended keeping it — its
    distinctness is the feature).
12. **`temp` is the default** (owner, later the same day): `for 'x' in …`
    is a temp loop; `perm` is stated when the variable outlives the loop;
    `temp` may still be written and changes nothing. Supersedes the
    2026-09-17 "temp/perm mandatory".

Also confirmed: a Limb-provided type is `'r': std::uenr`; style is a space
after `:`, none around `::`. `pub` was mentioned ("if we do `pub` too") and
is logged as open, not decided.

Oracle updated the same day (lexer `:` / `->`; parser rewritten for the
new grammar; every example and reject test converted; old spellings —
`var`, `func`, `loop.while`, `else.if`, `loop.temp.for`, `range[:= …]`,
`int32 'x'` — are refused with a message naming the new one).

### 2026-09-14 — The five core features (owner)

Proposed and chosen by the owner. Full text in [design/core-features.md](design/core-features.md).

1. **Everything at compile time** — every possible check runs at compile time; compile duration is explicitly not a concern.
2. **Ownership model** — one owner per value; compiler-determined freeing; no runtime cost.
3. **`:=` computes the *best* type** — not just any inferred type.
4. **Dot-chained syntax** — most syntax reads as a chain; "flow with the code".
5. **LLVM backend** — native code.

### 2026-09-14 — `:=` picks the type that is best for safety and performance (owner)

"Best" is defined as: the type that gives the safest *and* fastest final product, worked out by whole-program analysis of every value the binding ever holds. The compiler has effectively unlimited time, so it does the analysis rather than the programmer. Proposed options by Claude; the criterion chosen by the owner.

### 2026-09-14 — Unprovable `:=` bounds are a compile error (owner)

When the analysis cannot bound a value (it comes from outside the program —
input, files, the network), `:=` is refused and the programmer must write an
explicit type at that boundary. Downstream of the annotation everything is
provable again. No silent fallback to a wide/checked type. Options (a) wide
fallback, (b) refuse, (c) both proposed by Claude, who recommended (b); owner
chose (b).

### 2026-09-14 — Unproven safety properties get a compiler runtime check, removable only by a programmer guardrail (owner) — clarified 2026-09-19: a *proven* check is removed by Precog itself; corrected 2026-09-20: the runtime check is the *working build's* behaviour; in the final build an unproven site is a compile error

Where whole-program analysis cannot prove a safety property (overflow, bounds,
division by zero, …), the compiler **inserts a runtime check, regardless of
performance cost**. The check stays until the programmer (1) writes a real
guardrail — e.g. validates the input — and (2) explicitly disables that one
specific check via syntax. Example: a GUI field typed Bin16 receiving a larger
value must be caught by programmer-written validation, not by a compiler check
left in the final product.

Claude had recommended "compile error + programmer-supplied fact"; the owner
chose runtime checks as the default instead, with explicit per-site removal.
Note this does not alter the `:=` rule above: an unprovable `:=` is still a
compile error, since that is about choosing a type, not guarding a value.

### 2026-09-14 — Three check tiers at every safety-property site (owner)

Keyword spellings are placeholders; the tiers are decided.

- **default** (nothing written) — *"I have no guardrails yet, and I want you
  to help me."* The compiler proves what it can and inserts a runtime check
  wherever it can't. Never refuses to compile.
- **`check`** — *"I've built a guardrail, could you help me verify it?"* The
  compiler proves, exactly (bit-precise solver over the real program
  semantics, not an approximate domain), that the programmer's guardrail
  covers the property. Proven → the check is removed. Not proven → compile
  error naming the counterexample or the missing fact (e.g. an unbounded loop
  that needs a bound or invariant at the boundary).
- **`nocheck`** — *"Okay, I'm sure this is correct."* The check is removed
  with no verification. This is the only construct in the language that can
  make a final product unsafe, and that is the programmer's responsibility.

Claude proposed verified-only removal; the owner chose to also provide the
trusted tier, with the discipline left to the developer.

**Why the compiler needs the programmer at all (for the record):** a loop
whose iteration count depends on outside data has an unbounded effect; no
amount of compile time can bound it (Rice's theorem). The programmer supplies
the missing facts at the boundary — value ranges, length limits, loop bounds
or invariants — and the loop becomes provable.

### 2026-09-14 — Ownership with borrowing (owner)

Every value has exactly one owner; the owner's scope end frees it. Moves
transfer ownership; copies make an independent value; **borrows** give
temporary access without ownership, and the compiler proves the borrow never
outlives the owner and never aliases a mutation (Rust-style, with lifetimes).
Claude presented A (borrowing), B (moves/copies only with compiler-elided
copies), C (B plus opt-in verified borrows) and recommended B; owner chose A.

### 2026-09-14 — Lifetimes: inferred by default, optional verified annotation (owner)

The whole-program analyser sees every call site, so lifetimes are always
inferred and nothing has to be written. Lifetime syntax exists as an
*optional* annotation: writing one pins a function's borrow contract, and the
compiler verifies the body against it (same spirit as `check`). There is no
unverified form — a lifetime that isn't proven is a memory bug with no
performance upside. Claude presented infer-only vs both, recommended both;
owner chose both. Syntax deferred until dot-chaining is decided.

### 2026-09-14 — User names are quoted: `'name'` (owner; reaffirmed)

Every user-defined *variable* name is written in single quotes — `var 'x'`, `'x' + 'y'` — and a function's name is quoted at its definition (`func 'add'`)
— and every keyword is a bare dotted path. **This is settled; do not
relitigate.** It has been argued against (noise, unfamiliarity) and the owner
has reaffirmed it, because it is load-bearing:

- **No reserved words, ever.** `var 'var' := [1];` is legal. New keywords can
  never break existing code.
- **Word operators depend on it.** `x`, `xx`, `mod`, `and`, `or`, `not` are
  unambiguous only because a bare word is never a name.
- **Names can be anything** — `'+'`, spaces, emoji, `'is valid?'`.
- **Three token classes, visually distinct:** values wear `[ ]`, names wear
  `' '`, keywords wear nothing. A reader never needs the keyword list.
- Cost acknowledged: two extra (unshifted) characters per name, and a
  first-contact "why?" for readers from other languages. Accepted; the docs
  should state the reason up front.
- The "`'x'` looks like a character literal" objection is a prior from other
  languages; CyborgPL has no character type, so `'` has one meaning. Rule
  that follows: **if a character type is ever added, it must not use
  `'…'`.**

### 2026-09-14 — `[ ]` is the value bracket, and it is mandatory (owner) — superseded 2026-09-19: only where it builds

`[ ]` means "one value, built from what's inside". Text pieces join into a
text (`["line 1" \n "line 2"]`), a formula evaluates (`[10 + 20 + 30]`), a
single literal is still bracketed (`[10]`), and `std::print.stdout[...]` takes
the same construct. Every value position uses it — `var.bin16 'x' = [1000];`
— with no omission rule. Owner proposed bare-infix vs bracketed and chose
bracketed; Claude recommended the same.

### 2026-09-14 — Inside `[ ]`, a bare number is an operand, not a text piece (owner) — retired 2026-09-19 with the whitespace join; `"count: " ++ 10` is fine

Quoted literals, variable references, and specials like `\n` are *pieces*
and join as text. A bare numeric literal is an *operand* and must sit in an
operator expression. Two adjacent bare numbers (`[10 20]`) are a compile
error regardless of what precedes the bracket. Claude had proposed
"juxtaposition always joins as text"; the owner rejected it for bare numbers.

This is strict with no exceptions: `["count: " 10]` is also an error. A
literal belongs inside the string (`"count: 10"`); a computed number goes
into a variable first and is referenced as a piece (`["count: " 'n']`).
Owner, 2026-09-14.

### 2026-09-14 — Initial type family (owner)

`bin` (IEEE 754 binary float), `deci` (IEEE 754 decimal float), `int`
(signed), `uint` (unsigned), `str` (text), and — added once comparisons
existed — `bool`. Widths as suffixes in the chain
(`var.bin16`). Full table and notes in [design/types.md](design/types.md).

### 2026-09-14 — `str` is narrowed by encoding in the chain: `var.str.utf8` (owner) — now `'s': str.utf8`

Encoding is a chain segment on `str`, the same shape as a width on a numeric
type. Owner proposed the chain shape. Encodings `utf8`, `utf16`, `utf32`, `ascii`
(Claude's list, owner accepted); `:=` picks the best encoding from the
value's use (owner). Decided 2026-09-14.

### 2026-09-14 — `char` exists, narrowed by unit: `var.char.point` / `.grapheme` / `.byte` (owner) — now `'c': char.point`

A character type, with the chain saying which Unicode unit "one character"
means: `byte` (1 byte), `point` (one code point, 4 bytes flat), `grapheme`
(one grapheme cluster, variable size, owned). Claude laid out code point vs
grapheme cluster and proposed the two-unit chain; owner chose it and added
`byte`.

Character literals use **backticks**: `` var 'c' := [`1`]; `` — `"…"` is text,
`'…'` is a name, `` `…` `` is one character. Must be exactly one unit; with
`:=` Precog picks the unit. Owner, 2026-09-14.

### 2026-09-14 — `uenr`, Unbounded Exact Numerical Representation, from the standard Limb (owner)

An arbitrary-size number that never rounds, supplied by the std Limb rather
than the core language. Replaces the role of legacy's `bignum`/`bigint`.
Exact is exact: fractions are true rationals (owner, 2026-09-14).

### 2026-09-14 — A stored call wears the outer bracket: `var 'n' := [count[lend 'words']];` (owner) — superseded 2026-09-19: `let 'n' := count[lend 'words'];`

The `[ ]`-mandatory rule applies literally; a call's own bracket is its
argument list, not the value slot. Surfaced by the oracle (functions.md had
`:= add[10, 20]`); Claude offered both and leaned to the shorter; owner chose
the literal rule — one rule, no exception, and `[add[1] + 2]` never raises a
question.

### 2026-09-14 — Boolean literals are `true` / `false` (owner)

`var.bool 'correct?' = [true];`. Bare, reserved. Surfaced by the oracle —
no literal had ever been written down.

### 2026-09-14 — A number piece becomes text in its stored form, everywhere (owner)

`["x is " 'x']` holds `x is 0.1000000000000000055511151231257827021181583404541015625`
wherever it is built; `print` shows what a variable stores; `printu` alone
produces the useful form, for its own argument and its own pieces. Surfaced
by the oracle. Claude offered stored-by-default vs useful-when-building;
owner chose stored.

### 2026-09-14 — Shadowing only with `$:=` / `$=`; never for `immut` (owner)

First decided as "no shadowing", then refined the same day: a visible name
may be redeclared only with the `$`-marked operator, so hiding is always
deliberate and visible. An `immut` variable cannot be shadowed. Surfaced by
the oracle; owner's design.

### 2026-09-14 — `xx` chains are right-associative, as in mathematics; `( )` overrides (owner)

`[2 xx 3 xx 2]` = 512; `[(2 xx 3) xx 2]` = 64. Surfaced by the oracle.

### 2026-09-14 — A fresh value may be passed straight to a `lent` parameter (owner)

`count[["a", "b"]]` is allowed: the temporary lives for the call. No safety
issue — nothing else can reach it. Surfaced by the oracle; owner chose allow.

### 2026-09-14 — A lend may have its own name: `var 'r' := [lend 's'];` (owner) — now `let 'r' := lend 's';`

Surfaced by the oracle; owner chose allowed. Precog's lifetime inference
makes it safe in the compiler; the oracle refuses to return or store one.

### 2026-09-14 — A one-element list is `[1,]` (owner)

`[1]` is the number; `[1,]` is a list holding it. Surfaced by the oracle.
Also recorded: a `[ ]` inside a `[ ]` is a value (`[[1, 2], [3, 4]]`).

### 2026-09-14 — Mutability is statable (`var.mut`, `var.immut`) but not standard practice (owner) — now `let mut` / `let immut`

Plain `var 'x' := [1000];` is the norm; Precog infers. `immut` is a verified
contract (assignment is an error); `mut` states intent. Placed as a chain
segment before the type (`var.immut.int32`), following the existing chain
rule. No `const`: compile-time constants are Precog's to find. Owner's
decision on Claude's explanation of immutable-vs-constant.

### 2026-09-14 — Precedence: mathematics where settled, parentheses where not, parentheses override (owner)

`xx` > unary `-` > `x`/`/`/`mod` > `+`/`-` > comparisons > `not` > `and` >
`or`. Parentheses required for: `/` or `mod` beside another multiplicative
operator, `mod` beside anything, chained comparisons, `not` beside a
comparison. Claude offered a standard table vs none (recommending none);
owner chose the mathematical rule. Table in [design/operators.md](design/operators.md).

### 2026-09-14 — Builds are reproducible; requirements on Precog (owner)

Same source + compiler + named target → identical binary. Precog: no
wall-clock budgets (step counts only), pinned solver determinism (solver
version is part of the compiler version), compile-time I/O always a boundary,
target explicit and recorded, deterministic ordering, compile-time arithmetic
matching the target. JetPack lock file records a proof summary per site so
drift is visible. Raised via another session's question; Claude laid out the
requirements; owner adopted them. See
[design/reproducible-builds.md](design/reproducible-builds.md).

### 2026-09-18 — Structs (owner) — spelling revised 2026-09-19: `name: str.utf8`, `Locker[name = "A"]` (fields bare, later that day)

Blueprint `struct Name { type 'field' … }` — owner's shape: type first, no
`var` (a struct always owns its fields; Claude's reasoning, owner agreed),
no `;` (owner), `immut.` allowed on a field (owner: "allow it"). Values by
field name, every field required, each value in its own `[ ]` — owner chose
the no-exception form. Field access `'mine'.number` / `'mine'.number = [8];`
(owner confirmed the dot). Ownership: `give` on a field allowed and it
**consumes the whole variable** — owner's design, replacing Claude's
"forbid partial moves". See [design/structs.md](design/structs.md).

### 2026-09-18 — A struct is always owned (owner)

Even a struct of only plain fields needs the ownership verbs. Surfaced by
the oracle; Claude laid out plain-when-fields-are vs always-owned and
recommended always-owned (one rule; adding a field never changes call
sites); owner chose always-owned.

### 2026-09-18 — Compiler names: Cyborg Chassis Assembler (CCA), Nowork (owner)

The CyborgPL compiler is the **Cyborg Chassis Assembler**; N%'s compiler is
**Nowork**. Owner acknowledges "Assembler" may be confused with a real
assembler. Spelling "Chassis" vs the owner's "Chasis" to confirm.

### 2026-09-18 — A scripting language on top of CyborgPL (owner)

Exists; its writers never touch widths, streams, or `MAIN`; the final
product is compiled with Cyborg (full Precog); while iterating it compiles
itself, and that iteration compiler may assume what the real build will
prove. Resolves the fast-edit-loop concern without a Cyborg dev mode. Its own
iteration compiler **assumes the safest** (all checks on, nothing proven)
and hands off to Cyborg only for the final-product build (owner,
2026-09-18). Named **N%**: a separate language from both Cyborg and N#, with its own
designer-friendly syntax, fast compile, safest-by-default, performance
deferred to the Cyborg hand-off (owner, 2026-09-18). The hand-off is a
real translator. Open:
what exactly is hidden, boundary defaults, iteration/real-build agreement,
subset-or-own syntax, name. See [design/scripting.md](design/scripting.md).

### 2026-09-17 — Lists: two types, 1-based indexing, type-headed operations (owner) — element type bracketed since 2026-09-19: `list[int32]`

`list` (growable) and `array` (fixed), narrowing to the element type
(`var.list.int32`); with `:=` Precog picks (never-grown → `array`). Owner
chose two names over one after Claude laid out both and the everyday
meanings of the words. Indexing `'xs'[i]` is **1-based** — owner: "`'xs'[0]`
would be the value before the first, which doesn't exist"; Claude had
recommended 0 and withdrew it, since inclusive ranges make 1-based the
consistent choice. Operations are `list.length` / `list.append` /
`list.remove` / `array.length` — the type name as the chain's head (owner's
idea, "`list.` might just fit our lang"), `length` spelled out (owner: "`.length`
is better" than `len`). Language operations, not std. Element assignment,
empty list, boundary lengths remain open. See [design/lists.md](design/lists.md).

### 2026-09-17 — `1,000` is an error; leading zeros are illegal (owner)

A number followed by a comma and digits is an error pointing at `_`, rather
than the list `[1, 0]` the rules would otherwise make it; `000`/`007` are
not numbers. Surfaced by the OCaml oracle experiment.

### 2026-09-17 — Ownership verbs on a plain value are an error (owner)

`give 'n'` on an integer is a compile error, not a no-op: a plain value owns
nothing (no allocation behind it), so the verb would be false. Surfaced by
the OCaml oracle experiment, which allowed it. Owner chose error.

### 2026-09-17 — Comments: `#`, `#wl`, `#Nl`, `#Nld` (owner)

Owner's design, extending legacy's `#`/`#N`: `#` to end of line (trailing
allowed, unlike legacy), `#wl` whole line including preceding code, `#Nl` N
lines counting this one, `#Nld` this line plus N down. The oracle enforces
them.

### 2026-09-17 — Empty list: `= []` typed, `:= [list.empty]` untyped (owner)

Claude offered `= []` vs `= [list.empty]` for the typed form and recommended
the first; owner chose it and pointed out that under `:=` it must be
`[list.empty]`, since nothing on the left says "list". An empty `array` is an
error. See [design/lists.md](design/lists.md).

### 2026-09-17 — `pos` on every index; element assignment `'xs'[pos i] = […];` (owner)

Owner proposed `'xs'[pos 2]` and `'xs'[value-is 2]`. `pos` adopted everywhere
(reads and writes) — the bare `'xs'[2]` form is gone. `value-is` dropped:
Claude noted it hides a search with unanswered multiple-match questions;
owner: "can't you just use `if` for that one?" — a `perm` range loop finds
the position, then `pos` writes. No `list.find`. See [design/lists.md](design/lists.md).

### 2026-09-17 — Range loops; `temp`/`perm` mandatory on `for` and `range` (owner) — revised 2026-09-19: `for 'i' in range[a, b]`, `temp` is the default

`loop.temp.range 'i' [:= 0, := 1000] { }` — owner's shape. Decided one by
one: stop is **inclusive** ("what the writer sees": start from, stop at);
`temp`/`perm` **mandatory**; after `perm` the variable holds whatever the
loop last gave it (Claude's rule, owner confirmed); typed form
`loop.temp.range.int32 'i' [= 0, = 1000]` with the type last in the chain
(owner chose over `range`-last); `temp`/`perm` apply to `for` too; chain
order `loop.temp.seq.range.int32`; the body cannot assign to the counter
(Claude explained why; owner accepted); **step** exists as a third slot;
descending is a **negative step**; step 0 is an error. Parked: index
alongside element (needs indexing). Skipped: labelled `break`, character
ranges. `std::range[…]` as a value is superseded. The oracle enforces all of
it; see [design/control-flow.md](design/control-flow.md).

### 2026-09-16 — No development mode; Precog is optimised, never bypassed (owner)

Another session suggested a quicker path for everyday edits. Owner: "making
a shortcut for it is not Cyborg." Every build proves everything. Reusing
proofs for unchanged code is optimisation (same guarantees), not a
shortcut, and is allowed.

### 2026-09-14 — The oracle interpreter exists (owner)

`oracle/` — a small Rust tree-walking interpreter implementing exactly the
decided design, with `examples/*.cpls` (must run and match `.out`) and
`oracle/tests/reject/*.cpls` (must fail with the named message) as its test
suite. Owner asked for it so stale notes cannot drift from the language's
reality. It is the executable spec, not the compiler. Every pick it had to
make where the design is silent is listed in `oracle/README.md` and logged
below as open.

### 2026-09-14 — Function names are bare at the definition (owner)

`func.int32 add [...]`, matching the bare call `add[...]`. Quotes mean
variable; a function is not one. Function names are plain identifiers;
variables keep free naming. Owner asked whether functions should be quoted;
Claude laid out bare-both-ends vs quoted-both-ends (the latter needing a
separate index syntax); owner chose bare.

### 2026-09-14 — Automatic parallelism with an escape (owner)

Precog parallelises `loop.for` itself where ownership proves iterations
independent and the trip count makes it worthwhile; the runtime carries a
thread pool. The programmer writes nothing — except an opt-out chain segment
(`loop.for.seq`) for loops that must run in
order. Concurrency proper (async, channels, shared state) remains open.
Claude explained the concurrency/parallelism split and proposed this scope;
owner chose it. See [design/parallelism.md](design/parallelism.md).

### 2026-09-14 — Entry point simplified to `MAIN { }`; exit 0 implied; `std::exit[uint8]` otherwise (owner)

Supersedes `func.int32 MAIN` + mandatory `return` from earlier today. `MAIN`
is the program's block, not a function; ending it exits 0; other codes via
`std::exit[…]`, typed `uint8` (exit status is 8 bits on Unix — Claude's
correction of `int32`). Owner asked why a manual `return 0` exists at all;
answer: only because C made `main` a function. Uncallable by construction.

### 2026-09-14 — `lang:` removed; keywords are bare and reserved (owner)

The `lang:` prefix is dropped. Keywords — `func`, `var`, `return`, `if`,
`else`, `loop`, `in`, `break`, `continue`, `give`, `copy`, `lend`, `lent`,
`op`, `check`, `nocheck` — are bare and reserved, as in every mainstream
language. Claude laid out what `lang:` bought (zero reserved words, every
token self-describing) against its cost (five characters per statement, the
largest single source of verbosity, and the "esolang" first impression),
and offered keep / drop / shrink to `@`; owner chose drop: "I don't think
anyone would wanna name a function confusingly." Variables stay quoted;
`::`, `.`, `[ ]`, `{ }` unchanged. All samples in `design/` now read
without the prefix; earlier entries in this log that show `lang:` are
historical.

### 2026-09-14 — Control flow: `if` / `else.if` / `else`; `loop` / `loop.while` / `loop.for` (owner) — spelling revised 2026-09-19: `else if`, `while`, `for`

Branching as Claude proposed. Loops: Claude proposed three keywords
(`loop`, `while`, `for`); owner chose one, narrowed by the chain —
`loop`, `loop.while [cond]`, `loop.for 'x' in [list]`.
Conditions must be `bool`; braces mandatory; `break` / `continue`.
See [design/control-flow.md](design/control-flow.md).

### 2026-09-14 — Ownership transfer is explicit: `give` / `copy` / `lend` / `lend.mut` (owner) — parameter side revised 2026-09-19: `lent 'text': str.utf8`, owning is bare

A bare `'s' = ['t'];` on an owned (heap) value is a compile error; the
programmer writes `give` (move), `copy`, or `lend` /
`lend.mut` (borrow). Plain values copy with no keyword. Owner asked
whether a move needs a keyword; Claude recommended explicit and offered
Rust's words (`move`/`copy`/`borrow`) or owner's-perspective words
(`give`/`copy`/`lend`), recommending Rust's; owner chose owner's-perspective
as easier to understand. See [design/ownership-syntax.md](design/ownership-syntax.md).

Parameters declare what they accept (owner, 2026-09-14): `lent.str.utf8
'text'` receives a shared borrow, `lent.mut.…` a mutable one,
`var.…` ownership. Caller verb and parameter kind must agree. The
signature is the ownership contract.

### 2026-09-14 — No `++` / `+=` shorthand (owner)

Increment is `'i' = ['i' + 1];`. Claude laid out `++` (statement-only) and
compound assignment; owner considered `'i'++` and decided against any
shorthand.

### 2026-09-14 — Entry point is `func MAIN`, unquoted and uncallable (owner)

`func.int32 MAIN { … return [0]; }` — the return value is the exit
code, and **`MAIN` must declare `.int32` and return explicitly**; no implied
exit 0 (owner, after Claude offered both). `MAIN` is unquoted because it is the language's, not the
user's — a quoted `'main'` would be a reserved user name (owner's
observation). Owner proposed `START { }` or `func MAIN`; chose the
latter. `MAIN[]` is a compile error (Claude's recommendation; owner agreed):
one analysis root, one owner of top-level values, no legitimate re-entry.
A Limb with `MAIN` is a program; without, a library.

### 2026-09-14 — Local functions are called bare; other Limbs' via `::` (owner)

A function is a Limb member. Within its own Limb: `add[10, 20]`. From
elsewhere: `mathstuff::pow[2, 64]`. Quoted only at definition. Claude had
proposed `self::add`; owner chose bare, since `::` exists only for a real
separate `.limb`. Call vs index is told apart by shape, and so are calls and
operator words (a call is a word followed by `[`; an operator sits between
operands) — Claude first proposed reserving the six operator words as
function names; the owner pointed out the positions never collide, and the
exception was dropped. Zero reserved words, no exception.

### 2026-09-14 — `lang:` prefixes language built-ins (owner)

`lang:var.std::uenr 'n' = [1000];`, `lang:char.grapheme`, alongside
`std::print.stdout[...]`. Every path names its origin: `lang:` the language,
`Limb::` a Limb, `'…'` the user. Earlier `var …` samples read as `lang:var …`.
Scope: **every keyword** — `lang:func`, `lang:return`, `lang:if`,
`lang:check` (owner, 2026-09-14). Zero reserved words; a Limb may be named
`func`. All earlier samples read with `lang:` in front.

**Superseded the same day — see "`lang:` removed" below.**

### 2026-09-14 — `::` is the Limb boundary; `.` narrows within (owner)

Anything from a Limb is reached with `Limb::path`: `std::print.stdout[...]`,
`var.std::uenr 'n' = [1000];`. The chain continues across the boundary, so a
Limb type sits in a `var` chain like a built-in. Earlier samples written
`std.print.stdout` are superseded.

### 2026-09-14 — The typed form is always fully explicit (owner)

No bare `var.int`, `var.str`, or `var.char`: a typed declaration states the
width / encoding / unit (`var.int32`, `var.str.utf8`, `var.char.point`).
`:=` means Precog chooses; `=` means the programmer said everything. Claude
recommended; owner chose. Closes the width, encoding, and unit questions at
once.

### 2026-09-14 — `print` shows the real stored value; `printu` shows the useful one (owner)

`std::print.stdout[...]` prints exactly what is in memory — for a `bin` value
that is the full decimal expansion of the stored binary (`0.1` as `bin32`
prints `0.100000001490116119384765625`). `std::printu.stdout[...]` — `u` for
"useful" — prints the readable value (`0.1`). For `int`, `uint`, `str`, and
`deci` the two are identical; the difference only shows on `bin`. The honest
one gets the short name.

`print` on `bin128` dumps the full expansion — that is the point. `printu`'s
exact algorithm is Claude's call as long as the output is what a reader would
call useful; Claude will use shortest round-trip (Ryu-style). Owner, 2026-09-14.

### 2026-09-14 — Functions (owner) — spelling revised 2026-09-19: `fn add ['a': int32] -> int32`

`func.int32 'add' [var.int32 'a', var.int32 'b'] { return ['a' + 'b']; }`;
called as `add[10, 20]` (originally written `'add'[10, 20]`; superseded
the same day — see below). The chain narrows `func` by return type; omitted →
Precog infers. Parameters are `var` declarations; an untyped parameter's type
is inferred from all call sites. Inside `[ ]`, whitespace joins pieces into
one value and a comma separates values; several comma-separated values in a
bracket is a list. Digit grouping is `_`, not `,`. Claude proposed the shape
and `[a][b]` per-slot calls; owner chose comma-separated `[a, b]`. Full text
in [design/functions.md](design/functions.md).

### 2026-09-14 — Arithmetic operators (owner)

`+`, `-`, `x` (multiply), `xx` (power), `/`, `mod`, `==`, `and`, `or`,
`not`, `!==`, `<`, `>`, `<==`, `>==`. No `*`, no tetration. Word operators are
unambiguous because names are quoted. `<==`/`>==` are built from `==` because
a lone `=` is assignment and must never appear inside a comparison (owner
raised the inconsistency of `>=`; Claude proposed `>==`, words, unicode, or
none; owner chose `>==`). See
[design/operators.md](design/operators.md).

### 2026-09-14 — The compiler is verified by oracle + differential testing (owner)

For CyborgPL's own developers, not users: a reference interpreter (the
oracle, also the executable spec) and the optimised compiler are run on
generated programs and compared; any difference is a bug. Translation
validation of the LLVM half in CI; formal verification ruled out for now.
Claude proposed the layering; owner's framing was "oracle + optimised
generator". See [design/compiler-verification.md](design/compiler-verification.md).

### 2026-09-14 — Implementation languages (owner)

Rust for the compiler, `no_std` runtime, JetPack, test harness, and the
oracle interpreter (the oracle by a different AI model, no shared code); C++
only in a minimal LLVM wrapper; the standard Limb in CyborgPL. No C, no
OCaml. Claude first recommended Rust, then C++ throughout when the owner
raised "latest LLVM", then Rust + wrapper once the team shape was known;
owner chose Rust + wrapper. See
[design/implementation-languages.md](design/implementation-languages.md).

### 2026-09-14 — Review policy: human review follows risk (owner)

Tier 1, every line reviewed: Precog, borrow checker, proof-feeding into LLVM,
intrinsic contracts. Tier 2, tested and sampled: front end, runtime
arithmetic, JetPack, harness, oracle. Tier 3: the standard Limb is checked by
Precog itself. Claude proposed; owner chose. See
[design/review-policy.md](design/review-policy.md).

### 2026-09-14 — Licensing intent (owner)

Using the language: free, no credit anywhere. Modifying the compiler: free,
credit everywhere — source and every user-visible place. Claude mapped this
to Apache-2.0 WITH LLVM-exception (runtime, std Limb), Apache-2.0 plus an
attribution term (compiler), CC BY 4.0 (spec), trademarks (names), for
counsel to draft. See [design/licensing.md](design/licensing.md).

### 2026-09-14 — The compile-time technology is called Precog (owner)

The whole-program analysis/execution engine behind "everything at compile
time" is **Precog**. Claude proposed Precog, Foresight, Rehearsal, Ghost run,
Cortex and recommended Precog; owner chose Precog, accepting that the word is
shared with unrelated products.

### 2026-09-14 — Libraries are Limbs; the package manager is JetPack (owner)

A **Limb** is the unit of distribution, like Rust's crate — any library or program; `std` is one Limb. The package manager is **JetPack** (fast package),
manifest `JetPack.toml`. Source files are **`.cpls`** (CyborgPL Source; `.cpl`
avoided). A published Limb is a **`.limb`** file — the Limb directory packaged
for JetPack, always containing analysable source, never an opaque binary
(whole-program analysis must see inside it). See [design/tooling.md](design/tooling.md).

## Open questions

- GPU offload (owner chose automatic, 2026-09-21): the offload test, the runtime, target descriptions for GPUs, what the body may contain. Not designed.

**Language — surfaced by the second (OCaml) oracle, 2026-09-17**

**Language — surfaced by the oracle**

**Language**
- N%: its syntax (undesigned), boundary defaults in the translation, translator verification. (owner's direction, 2026-09-18)
- Range types on a text (a length bound on `str`, now that a list has one); one-ended ranges. (2026-09-20, narrowed 2026-09-21)
- Lifetime annotation spelling (optional, verified). (Claude, 2026-09-14)
- Generic `op`/`const`; std's standard interfaces; associated types. (2026-09-20)
- Zero values: a value is required at declaration; decide explicitly — Claude recommends no implicit zero (the oracle enforces it). (Claude, 2026-09-14)
- Concurrency (async, channels, shared mutable state) — not designed; parallelism is decided separately. (Claude, 2026-09-14)
- Conversion, still open parts: a useful-form (short) number → text. (2026-09-19)
- A per-parameter `bare` flag (arguments written without quotes) — sketched for `std::c.printf`, advised against, shelved. (2026-09-19)
- Type widths to confirm: `bin16/32/64/128`, `deci32/64/128`, `int`/`uint` 8–128. (Claude, 2026-09-14)
- Limb name declared in source (`limb [...]`)? Revisit with `pub` and the author's bare-reachability flag (`design/limbs.md`). (Claude, 2026-09-14)
- The Limb author's "reachable bare" flag — spelling, beside or on `pub`. (`design/limbs.md`, 2026-09-19)
- `range[…]` as a standalone value (not only after `for … in`). (Claude, 2026-09-19)

**Precog**
- Non-terminating compile-time code: with step budgets now required, the budget's size and the error it produces. (Claude, 2026-09-14)
- Build output lists every `nocheck` site? Claude recommends yes. (Claude, 2026-09-14)

**Process**
- Build the oracle interpreter first, before the real compiler? Claude recommends yes. (Claude, 2026-09-14)
