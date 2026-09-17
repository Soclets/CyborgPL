# The scripting language

Owner's direction, 2026-09-18. A higher-level language on top of CyborgPL.

## What is decided

- **It exists.** Writers of it do not think about the low-level parts of
  CyborgPL — widths (`bin16`), streams (`stdout`), the entry point (`MAIN`),
  and the like.
- **The final product is compiled with Cyborg.** A script is lowered to
  CyborgPL and goes through Precog like anything else: full proofs, the
  three tiers, LLVM.
- **While iterating, it compiles itself.** It has its own fast path for the
  edit-run loop. That path may assume what the real build will later prove,
  because the real build always runs — this is how the "slow everyday
  edits" concern is answered *without* a development mode in Cyborg itself
  (see `core-features.md`, "No development mode": Cyborg never skips
  Precog; the scripting language's iteration compiler is not Cyborg).

## The iteration rule (owner, 2026-09-18)

The scripting language's own compiler, used while iterating, **assumes the
safest**: every check on, widest types, nothing proven away — the default
tier everywhere. It hands the program to Cyborg only when the writer asks
for the final product, and Cyborg then proves what it can and removes what
it proves. So an iteration build is never *less* safe than the final one,
only slower; the writer sees the same behaviour in both, and the final one
is faster.

Name: the owner wrote **N%** (2026-09-18); to be confirmed against the
existing `nsharp` (N#) repo.

## What it implies (open)

1. **What the writer doesn't see:** probably also the ownership verbs, the
   typed form, `std::` paths. `print["hi"]` rather than
   `std::printu.stdout["hi" \n]`; a script file is its own `MAIN`.
2. **The boundary problem.** Precog refuses `:=` on a value it cannot bound;
   a script writer never writes a type, so the lowering must supply wide
   defaults at every boundary (`int64`, `str.utf8`, unbounded `list`) and
   let the default tier keep its runtime checks. Precog still proves what it
   can; the writer never sees a refusal.
3. **The iteration compiler must agree with the real build** — same program,
   same output — or the writer debugs a different program than they ship.
   The oracle is already that machine for lowered scripts; differential
   testing applies.
4. **Syntax:** a strict subset of CyborgPL (lowering = inserting defaults;
   a script writer graduates to Cyborg by adding chains), or its own.
   Claude leans subset.
5. **A name.**
