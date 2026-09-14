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

### 2026-09-14 — `[ ]` is the value bracket, and it is mandatory (owner)

`[ ]` means "one value, built from what's inside". Text pieces join into a
text (`["line 1" \n "line 2"]`), a formula evaluates (`[10 + 20 + 30]`), a
single literal is still bracketed (`[10]`), and `std.print.stdout[...]` takes
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
(signed), `uint` (unsigned), `str` (text). Widths as suffixes in the chain
(`var.bin16`). Full table and notes in [design/types.md](design/types.md).

### 2026-09-14 — `print` shows the real stored value; `printu` shows the useful one (owner)

`std.print.stdout[...]` prints exactly what is in memory — for a `bin` value
that is the full decimal expansion of the stored binary (`0.1` as `bin32`
prints `0.100000001490116119384765625`). `std.printu.stdout[...]` — `u` for
"useful" — prints the readable value (`0.1`). For `int`, `uint`, `str`, and
`deci` the two are identical; the difference only shows on `bin`. The honest
one gets the short name.

`print` on `bin128` dumps the full expansion — that is the point. `printu`'s
exact algorithm is Claude's call as long as the output is what a reader would
call useful; Claude will use shortest round-trip (Ryu-style). Owner, 2026-09-14.

### 2026-09-14 — Libraries are Limbs; the package manager is JetPack (owner)

A library is a **Limb**. The package manager is **JetPack** (fast package),
manifest `JetPack.toml`. See [design/tooling.md](design/tooling.md).

## Open questions

- Should the build output list every `nocheck` site so they are auditable in one place? Claude recommends yes. (raised by Claude, 2026-09-14)
- Boundary annotations: how does the programmer state value ranges / length limits on an external-input type (e.g. `list<i64 in 0..1000, len <= 100_000>` as a placeholder)? Needed for `check` to succeed on loops. (raised by Claude, 2026-09-14)
- Analysis precision budget: precise by default with a very large loop-unroll bound, widen beyond it and report where? (raised by Claude, 2026-09-14)
- Compile-time side effects: does a call like `read_file("config.txt")` with known arguments run at compile time, or is I/O always a boundary? Claude recommends always a boundary. (raised by Claude, 2026-09-14)
- Non-terminating compile-time execution: accept (it is a program bug) or a step budget that becomes an error? (raised by Claude, 2026-09-14)
- Borrow and lifetime syntax: how `&`/`&mut`-equivalents and the optional lifetime annotation look in a dot-chained language. Depends on the dot-chaining decision. (raised by Claude, 2026-09-14)
- Whether mixed-precedence arithmetic must be parenthesised (`[a + (b * 2)]`, Pony-style) or standard precedence applies. (raised by Claude, 2026-09-14)
- Operator glyphs: `*` vs `x` for multiply, `^` vs `**` for power, whether tetration survives. (raised by Claude, 2026-09-14)
- Explicit numeric conversion spelling (no implicit cross-family conversion). (raised by Claude, 2026-09-14)
- Type widths to confirm: `bin16/32/64/128`, `deci32/64/128`, `int`/`uint` 8–128. Is the width mandatory in the explicit form (no bare `int`)? Claude recommends yes. (raised by Claude, 2026-09-14)
- "Limb": the standard library's name, or the word for any library/package? (raised by Claude, 2026-09-14)
- Missing types: boolean, character, byte alias, arbitrary precision (legacy `bignum`/`bigint`). (raised by Claude, 2026-09-14)
- Dot chaining: the owner's sketch (`design/syntax-sketch.md`) reads chains as general→specific *paths* (`var.bin16`, `std.print.stdout`), not method calls. Still to confirm as the direction, plus: short form only or long form too; `[ ]` whitespace semantics; list syntax; integer type names. (owner sketch, 2026-09-14)
