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

## Open questions

- Check-disable syntax: is the disable *verified* (compiler must prove the guardrail suffices, else compile error) or *trusted* (programmer's word, an unsafe escape hatch)? Claude recommends verified. (raised by Claude, 2026-09-14)
- Analysis precision budget: precise by default with a very large loop-unroll bound, widen beyond it and report where? (raised by Claude, 2026-09-14)
- Compile-time side effects: does a call like `read_file("config.txt")` with known arguments run at compile time, or is I/O always a boundary? Claude recommends always a boundary. (raised by Claude, 2026-09-14)
- Non-terminating compile-time execution: accept (it is a program bug) or a step budget that becomes an error? (raised by Claude, 2026-09-14)
- Ownership: Rust-style with borrows/lifetimes, or ownership without borrowing (moves + copies only)? (raised by Claude, 2026-09-14)
- Dot chaining: does it cover declarations and control flow too, or only expressions? (raised by Claude, 2026-09-14)
