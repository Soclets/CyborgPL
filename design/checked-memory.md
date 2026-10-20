# Checked manual memory

Direction decided 2026-10-13 (owner: "I think we'll do that instead of
refc"); **not designed yet**. The conversation is in `DECISIONS.md` under
2026-10-13.

## What it is for

Ownership covers almost every program, proven and free at run time. It
cannot cover data whose longest holder is known only while running: a
node with two parents, data several threads share and drop in any order,
an entity that dies when the game says. Rust answers that with reference
counting (`Rc`, `Arc`); CyborgPL answers it with **manual memory the language
checks**, instead (owner).

The layers, then:

1. **Ownership** — almost everything; proven, no run-time cost.
2. **Checked manual memory** — data that really is shared; you allocate
   and free, the language makes a wrong use a caught failure.
3. **`malloc` / `free` through C** (`ffi.md`, CNC limb) — every pointer use
   `no_check`; for C, custom allocators, the last bit of speed.

## The handle: `manual[T]` (owner, 2026-10-13: "Manual it is")

```
struct Node {
    'value': i32;
    'left': manual[Node];
    'right': manual[Node];
}
```

- `T` is any type: `manual[Node]`, `manual[i32]`, `manual[array[u8, 64]]`.
- A **pointer type**, as `ptr[T]` is, not a number: inside, an address and
  the generation the handle was made with. No conversion to or from an
  integer.
- **Plain**: copying a handle copies those two, never the `T`; two copies
  reach the same value (two parents, one child).
- **Not owned**: a variable holding one ends and nothing is freed.
- **`manual[T]` reads; `manual.mut[T]` reads and writes** (owner,
  2026-10-13: "(b)"), as `ptr` / `ptr.mut` and `lend` / `lend.mut`. A
  `manual.mut[T]` can be handed out as a `manual[T]`, never the reverse.
- Kept apart from `ptr[T]`, C's unchecked pointer: the name says the duty a
  reader must not miss, that someone frees it.

## How it is checked: generations

- Each allocation carries a **generation** number; a handle to it holds the
  generation it was made with. Copies of a handle are fine.
- **Freeing bumps the generation.** A use compares the two: different means
  freed, and the use fails instead of reading reused memory. A second free
  is caught the same way.
- **the prover removes the comparison** wherever it proves the handle alive.
- Leaks stay possible (a handle never freed is safe but wasteful).

## Where the memory comes from (owner, 2026-10-13)

- **The checking is the base's** (owner: "(a)"): handles, generations,
  caught use after free and double free, the prover's proofs, the same on bare
  metal as on an OS. Only the OS allocator is outside it (design claude's
  reading: in `std`, with everything else that needs an OS).

- **A handle works on its own** (owner: "Like, manual"): allocated
  anywhere, freed by the program, no pool to carry around.
- **CyborgPL ships an allocator only for targets with an OS**, taking its
  memory from the OS (owner: "We're no shipping our own freestanding one,
  only OS one").
- **Freestanding, the program brings its own allocator** (pools, bump,
  anything, over RAM it names), as freestanding C and Rust's `no_std` do;
  the checking is the same on top. A program that wants none uses only
  fixed memory and the stack, which ownership already covers.
- **Each allocation names its allocator** (owner, 2026-10-13: "(b)", over
  one for the whole program), as Zig does: different memory can serve
  different things, fast RAM for one buffer, DMA-reachable RAM for another.
  **The OS one is named too** (owner: "(a)"), so a kit that allocates
  takes its allocator from its caller and builds for bare metal unchanged.
- **Who keeps freed memory: both ways** (owner, 2026-10-13: "Both"):
  - **By default, CyborgPL.** The program's allocator only supplies fresh
    memory (as the OS is to mimalloc) and never sees a free; CyborgPL reuses
    a freed block only for an allocation of its size, from per-thread
    lists, so no allocator can break the checks. **Checked arenas** are
    built in: everything from one source freed at once, by bumping one
    generation for the source that every handle also checks.
  - **As trusted code, the allocator.** An allocator may handle its own
    frees, promising that a freed block is reused only at its size and
    never given back. The promise is not checked: such an allocator is
    listed with the build's `no_check` sites.
- The comparison never depends on the allocator; its soundness depends on
  freed memory keeping its size, which is why the default keeps it.

## Working and final builds (owner, 2026-10-13)

The checks follow the rule every check follows (`core-features.md`,
2026-09-20):

- **Working build**: an unproven use of a handle is checked at run time and
  fails loudly at its line.
- **Final build**: an unproven use is refused, with a counterexample. It is
  proven, or guarded by a written "is it alive" test that the prover reads, or
  `no_check`. The final product holds no inserted check; the generation
  stays in memory for the written guards.

## Open

- The spelling: allocating, freeing, the alive test.
- The generation's width (64 bits never wraps; 32 makes handles smaller):
  kayo's, unless the owner wants it.
- Handles across threads and tasks (`concurrency.md`).
- What the prover proves about handles.
- Whether a garbage collector is ruled out (asked 2026-10-13, not ruled;
  design claude advised against one).
