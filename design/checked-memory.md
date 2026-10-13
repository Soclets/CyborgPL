# Checked manual memory

Direction decided 2026-10-13 (owner: "I think we'll do that instead of
refc"); **not designed yet**. The conversation is in `DECISIONS.md` under
2026-10-13.

## What it is for

Ownership covers almost every program, proven and free at run time. It
cannot cover data whose longest holder is known only while running: a
node with two parents, data several threads share and drop in any order,
an entity that dies when the game says. Rust answers that with reference
counting (`Rc`, `Arc`); Nerve answers it with **manual memory the language
checks**, instead (owner).

The layers, then:

1. **Ownership** — almost everything; proven, no run-time cost.
2. **Checked manual memory** — data that really is shared; you allocate
   and free, the language makes a wrong use a caught failure.
3. **`malloc` / `free` through C** (`ffi.md`, CNC lobe) — every pointer use
   `no_check`; for C, custom allocators, the last bit of speed.

## How it is checked: generations

- Each allocation carries a **generation** number; a handle to it holds the
  generation it was made with. Copies of a handle are fine.
- **Freeing bumps the generation.** A use compares the two: different means
  freed, and the use fails instead of reading reused memory. A second free
  is caught the same way.
- **Cortex removes the comparison** wherever it proves the handle alive.
- Leaks stay possible (a handle never freed is safe but wasteful).

## Working and final builds (owner, 2026-10-13)

The checks follow the rule every check follows (`core-features.md`,
2026-09-20):

- **Working build**: an unproven use of a handle is checked at run time and
  fails loudly at its line.
- **Final build**: an unproven use is refused, with a counterexample. It is
  proven, or guarded by a written "is it alive" test that Cortex reads, or
  `no_check`. The final product holds no inserted check; the generation
  stays in memory for the written guards.

## Open

- The spelling: allocating, freeing, the handle's type, the alive test.
- What a freed slot is reused for (memory under a generation cannot simply
  go back to the OS while old handles may be compared against it).
- Handles across threads and tasks (`concurrency.md`).
- What Cortex proves about handles.
- Whether a garbage collector is ruled out (asked 2026-10-13, not ruled;
  design claude advised against one).
