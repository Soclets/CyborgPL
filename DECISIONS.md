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

## Open questions

- What "best type" means for `:=` — smallest fitting width? most precise? most constrained by later use? (raised by Claude, 2026-09-14)
- How far "everything at compile time" goes — constant folding only, or full evaluation of any call whose inputs are known (CTFE), or whole-program abstract interpretation? (raised by Claude, 2026-09-14)
- Ownership: Rust-style with borrows/lifetimes, or ownership without borrowing (moves + copies only)? (raised by Claude, 2026-09-14)
- Dot chaining: does it cover declarations and control flow too, or only expressions? (raised by Claude, 2026-09-14)
