# CyborgPL v2 — Design decisions

Running log. Each entry: date, the decision, who proposed it, and who chose it.
Nothing here is final until the owner says so; open questions live at the bottom.

## Decided

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

### 2026-09-14 — Unproven safety properties get a compiler runtime check, removable only by a programmer guardrail (owner)

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

### 2026-09-14 — `[ ]` is the value bracket, and it is mandatory (owner)

`[ ]` means "one value, built from what's inside". Text pieces join into a
text (`["line 1" \n "line 2"]`), a formula evaluates (`[10 + 20 + 30]`), a
single literal is still bracketed (`[10]`), and `std::print.stdout[...]` takes
the same construct. Every value position uses it — `var.bin16 'x' = [1000];`
— with no omission rule. Owner proposed bare-infix vs bracketed and chose
bracketed; Claude recommended the same.

### 2026-09-14 — Inside `[ ]`, a bare number is an operand, not a text piece (owner)

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

### 2026-09-14 — `str` is narrowed by encoding in the chain: `var.str.utf8` (owner)

Encoding is a chain segment on `str`, the same shape as a width on a numeric
type. Owner proposed the chain shape. Encodings `utf8`, `utf16`, `utf32`, `ascii`
(Claude's list, owner accepted); `:=` picks the best encoding from the
value's use (owner). Decided 2026-09-14.

### 2026-09-14 — `char` exists, narrowed by unit: `var.char.point` / `.grapheme` / `.byte` (owner)

A character type, with the chain saying which Unicode unit "one character"
means: `byte` (1 byte), `point` (one code point, 4 bytes flat), `grapheme`
(one grapheme cluster, variable size, owned). Claude laid out code point vs
grapheme cluster and proposed the two-unit chain; owner chose it and added
`byte`.

Character literals use **backticks**: `` var 'c' := [`1`]; `` — `"…"` is text,
`'…'` is a name, `` `…` `` is one character. Must be exactly one unit; with
`:=` Precog picks the unit. Owner, 2026-09-14.

### 2026-09-14 — `uer`, Unbounded Exact Representation, from the standard Limb (owner)

An arbitrary-size number that never rounds, supplied by the std Limb rather
than the core language. Replaces the role of legacy's `bignum`/`bigint`.
Exact is exact: fractions are true rationals (owner, 2026-09-14).

### 2026-09-14 — Automatic parallelism with an escape (owner)

Precog parallelises `loop.for` itself where ownership proves iterations
independent and the trip count makes it worthwhile; the runtime carries a
thread pool. The programmer writes nothing — except an opt-out chain segment
(`loop.for.sequential`, spelling to confirm) for loops that must run in
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

### 2026-09-14 — Control flow: `if` / `else.if` / `else`; `loop` / `loop.while` / `loop.for` (owner)

Branching as Claude proposed. Loops: Claude proposed three keywords
(`loop`, `while`, `for`); owner chose one, narrowed by the chain —
`loop`, `loop.while [cond]`, `loop.for 'x' in [list]`.
Conditions must be `bool`; braces mandatory; `break` / `continue`.
See [design/control-flow.md](design/control-flow.md).

### 2026-09-14 — Ownership transfer is explicit: `give` / `copy` / `lend` / `lend.mut` (owner)

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

`lang:var.std::uer 'n' = [1000];`, `lang:char.grapheme`, alongside
`std::print.stdout[...]`. Every path names its origin: `lang:` the language,
`Limb::` a Limb, `'…'` the user. Earlier `var …` samples read as `lang:var …`.
Scope: **every keyword** — `lang:func`, `lang:return`, `lang:if`,
`lang:check` (owner, 2026-09-14). Zero reserved words; a Limb may be named
`func`. All earlier samples read with `lang:` in front.

**Superseded the same day — see "`lang:` removed" below.**

### 2026-09-14 — `::` is the Limb boundary; `.` narrows within (owner)

Anything from a Limb is reached with `Limb::path`: `std::print.stdout[...]`,
`var.std::uer 'n' = [1000];`. The chain continues across the boundary, so a
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

### 2026-09-14 — Functions (owner)

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

**Language**
- Precedence: standard, or parentheses required when mixing levels (Pony-style)? (Claude, 2026-09-14)
- Range values (`std::range[0, 10]` vs an operator). (Claude, 2026-09-14)
- Structs / records — not designed. (Claude, 2026-09-14)
- Lifetime annotation spelling (optional, verified). (Claude, 2026-09-14)
- Check-tier syntax: where `check` / `nocheck` sit on a statement. (Claude, 2026-09-14)
- Custom operators: `op` definition shape, bare infix use, and the precedence rule (Claude recommends: no precedence anywhere, parentheses required when mixing different operators). (Claude, 2026-09-14)
- Boundary annotations: stating value ranges / length limits on external input, needed for `check` to succeed on loops. (Claude, 2026-09-14)
- Wrapping arithmetic: hashes and ring counters *want* wrap; without an explicit wrapping form every such site becomes `nocheck`. (Claude, 2026-09-14)
- Zero values: the `var … := [ ]` shape implies a value is required at declaration; decide explicitly — Claude recommends no implicit zero. (Claude, 2026-09-14)
- Error handling — not designed. (Claude, 2026-09-14)
- Concurrency (async, channels, shared mutable state) — not designed; parallelism is decided separately. (Claude, 2026-09-14)
- `loop.for.sequential` as the parallelism opt-out spelling — to confirm. (Claude, 2026-09-14)
- `/` on integers (truncate? error on inexact?); `mod` sign rule. (Claude, 2026-09-14)
- Explicit numeric conversion spelling. (Claude, 2026-09-14)
- Type widths to confirm: `bin16/32/64/128`, `deci32/64/128`, `int`/`uint` 8–128. (Claude, 2026-09-14)
- Whitespace between pieces in `[ ]`: separator only, or an implicit space? (Claude, 2026-09-14)
- Long form (`variable`, `standard`) valid alongside the short form, or short only? (Claude, 2026-09-14)
- Limb name declared in source (`limb [...]`)? Superseded in spirit by bare local calls; revisit with the module system. (Claude, 2026-09-14)
- Parameter lists: drop the redundant `var` inside `[ ]`? (Claude's suggestion, 2026-09-14)

**Precog**
- Compile-time side effects: I/O always a boundary? Claude recommends yes. (Claude, 2026-09-14)
- Analysis precision budget and non-terminating compile-time code. (Claude, 2026-09-14)
- Build output lists every `nocheck` site? Claude recommends yes. (Claude, 2026-09-14)

**Process**
- Build the oracle interpreter first, before the real compiler? Claude recommends yes. (Claude, 2026-09-14)
