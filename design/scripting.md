# The scripting language

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

Owner's direction, 2026-09-18. A higher-level language on top of CyborgPL.

## What is decided

- **It exists.** Writers of it do not think about the low-level parts of
  CyborgPL — widths (`bin16`), streams (`stdout`), the entry point (`MAIN`),
  and the like.
- **The final product is compiled with Cyborg.** A script is lowered to
  CyborgPL and goes through the prover like anything else: full proofs, the
  three tiers, LLVM.
- **While iterating, it compiles itself.** It has its own fast path for the
  edit-run loop. That path may assume what the real build will later prove,
  because the real build always runs — this is how the "slow everyday
  edits" concern is answered *without* a development mode in Cyborg itself
  (see `core-features.md`, "No development mode": Cyborg never skips
  the prover; the scripting language's iteration compiler is not Cyborg).

## The iteration rule (owner, 2026-09-18)

The scripting language's own compiler, used while iterating, **assumes the
safest**: every check on, widest types, nothing proven away — the default
tier everywhere. It hands the program to Cyborg only when the writer asks
for the final product, and Cyborg then proves what it can and removes what
it proves. So an iteration build is never *less* safe than the final one,
only slower; the writer sees the same behaviour in both, and the final one
is faster.

## Lunae, formerly N% (owner, 2026-09-18; renamed 2026-09-27)

The scripting language is **Lunae** (named N% until 2026-09-27; the owner
liked Lua's name) — its own language, distinct from both
Cyborg and the owner's earlier N# (`nsharp`). **Its own syntax, not
Cyborg's.** Designer-friendly; fast to compile; safest by default; does not
care about performance until it is handed to Cyborg. The hand-off is
therefore a real translation — Lunae source → CyborgPL source → the prover — not
an insertion of defaults into Cyborg syntax; the translator is the
component that has to be right, and the differential harness (Lunae's own
compiler vs. the lowered CyborgPL through the oracle) is how that is known.

## What it implies (open)

1. **What the writer doesn't see:** probably also the ownership verbs, the
   typed form, `std::` paths. `print["hi"]` rather than
   `std::print.useful.stdout["hi", \n]`; a script file is its own `MAIN`.
2. **The boundary problem.** the prover refuses `:=` on a value it cannot bound;
   a script writer never writes a type, so the lowering must supply wide
   defaults at every boundary (`int64`, `str.utf8`, unbounded `list`) and
   let the default tier keep its runtime checks. the prover still proves what it
   can; the writer never sees a refusal.
3. **The iteration compiler must agree with the real build** — same program,
   same output — or the writer debugs a different program than they ship.
   The oracle is already that machine for lowered scripts; differential
   testing applies.
4. ~~Syntax~~ — decided: its own (above).
5. ~~A name~~ — decided: N%, renamed **Lunae** (2026-09-27).
6. The translator Lunae → CyborgPL, and its verification.
