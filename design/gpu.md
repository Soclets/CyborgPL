# GPU offload

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

Opened 2026-09-22. Part of v1 (owner, 2026-09-21), **revised 2026-10-19: v1 has Metal through `inline.MSL` and
std's explicit control, but not the automatic loops** (owner: "no auto GPU loops yet"). The rule is
`parallelism.md`'s, extended: the programmer writes `for`, the prover
decides CPU cores or GPU, `for seq` says no. No GPU syntax. Underneath,
the loop body is compiled for the GPU target (by LLVM for Vulkan, by
kayo's own Metal generator for Metal) and the runtime, the runtime, launches it and
moves the data. This document is what the prover needs to
decide, and what the runtime does.

## What a body may contain — plain values, and failures (owner: b)

A GPU runs thousands of copies of the body at once, each on its own
slice of the range, with no operating system underneath. the prover may
offload a loop whose body uses only:

- plain values — `int`, `uint`, `bin`, `bool`, `char.byte` / `.point`
  — and lists, arrays and structs of them, lent into the loop or built
  inside it;
- arithmetic, comparisons, `if`, nested `for` over known ranges,
  indexing into the lent lists;
- calls to functions whose bodies meet the same rule;
- **`fail`**. A body may fail. A GPU cannot stop mid-launch, so an
  iteration that fails records its index and reason (an atomic minimum,
  so the *lowest* failing index wins — the same iteration that would
  have failed first sequentially) and the launch runs to the end; the
  runtime then raises that failure on the CPU exactly as if that
  iteration had run there. Measured 2026-09-22 on the M5 (Metal, the
  same numeric body with and without a `fail` path, 16M to 1G
  iterations, 8.6 GB of buffers at the top): the cost is **0.0%** — the
  branch is predicted away and the atomic runs only on failing
  iterations. The GPU was 13–42× one CPU core across those sizes.

Never offloadable: `extern` calls, `std::input` / `print`, `inline.ASM`,
`no_check` sites (nothing proven, nothing offloadable), text
(allocation per `++` on a GPU is 10–100× slower than on the CPU; a
text loop is never worth offloading), `std::uenr` and `deci` (software
arithmetic with allocation). A loop that touches any of these stays on
the CPU, silently — that is not an error, it is the prover deciding.

Proposed: (a) plain values only; (b) a plus failures; (c) b plus text.
Owner: b.

## How data crosses — the prover keeps it there (owner: b)

On unified memory (the M5) a lent list is already where the GPU can
see it; nothing moves. On a discrete GPU a list crosses a bus in and
its changes cross back, and the copy can cost more than the loop.
the prover decides the crossings the way it decides everything: it sees
the whole program. A list used by three offloaded loops in a row, with
no CPU access between them, goes over once and comes back once; a list
the CPU reads between two loops comes back between them. This is a
liveness question ownership already mostly answers (who touches what,
when). Proposed: (a) copy per loop — simple, twice the copies; (b)
this; (c) a programmer hint — nothing else in Cyborg is a hint. Owner: b.

## Targets in v1 — Metal and Vulkan (owner)

Metal for Apple (the machine this is designed on; the benchmark's
runtime), Vulkan / SPIR-V for everyone else — AMD, Intel, NVIDIA,
phones — the portable one. CUDA / PTX is not in v1: NVIDIA runs Vulkan,
and CUDA would need NVIDIA's driver found (never bundled) and hardware
to test on. the prover's decisions are the same for any GPU; a target is a
runtime module for launching and memory plus a the manifest (`Synthia.ttsr` / `CyJet.cyr`) line, so
CUDA later is an addition, not a change. Proposed: Metal only; Metal +
CUDA; all three. Owner: Metal and Vulkan.

**Metal is built first, Vulkan after** (owner, 2026-10-01: "it's gonna be
Metal before Vulkan"): Metal is this machine's GPU, so it can be built
and tested here.
The first public GitHub release needs only Metal (owner, 2026-10-01).

**How each is compiled** (owner, 2026-09-30: direct text). Vulkan: LLVM
compiles the body to SPIR-V. Metal: kayo writes Metal Shading Language
text straight from its own MIR, which Apple's compiler then builds. This
takes the place of making SPIR-V once and converting it. Direct text can
use Metal-only features (operations across a SIMD group, threadgroup
memory) and carry the prover's facts into the code, where a converter loses
some of both. It costs a second code generator, and so a second place
the prover's proofs must be carried correctly and tested. The speed edge is
expected, not measured. No C++: the Metal runtime is reached from Rust.

## The report (owner: yes)

A line per `for` the prover considered — offloaded, parallel on CPU cores,
sequential and why (`diagnostics.md`).

## Measured once, on purpose: `synapse tune` (owner, 2026-09-28; kayo's since 2026-09-30)

By default a cost model decides CPU or GPU: the loop's work (trips times
the body's cost) against the GPU's launch cost. The owner asked whether
the prover could instead run both at compile time when the building machine
is the target. Not inside a build: the prover uses no wall-clock
(`reproducible-builds.md`), so a build never times anything. Instead, as
profile-guided optimisation, a separate command measures once:

```
synapse tune              # builds a measuring version, runs it on this machine with the inputs given,
                      # times each undecided loop on CPU and GPU, and writes tune.cyr
synapse build --final     # reads tune.cyr as an input, like the lock file
```

**The measuring is kayo's** (owner, 2026-09-30: a): kayo's package manager
can do whatever Synthia can, and Synthia does what a kayo user would, so
`synthia tune` runs `synapse tune`. **Synthia also shows the results
readably** (owner: "synthia should have a readable version"), as
`synthia show --readable` does a manifest's times; its spelling is open.
Proposed and not chosen: both implementing the measuring; Synthia only,
as this section first had it (written 2026-09-28, when Synthia was to be
CyborgPL's one package manager).

`tune.cyr` records the target, the compiler version and, per loop, the
winner and its times. It is committed with the code, so a build of the
same source and the same file is always the same program. An entry is
ignored, and the build says so, when its loop has changed since it was
measured, or when the file was made for a different target. Then the cost
model decides. Proposed alternative: the cost model only. The file name
was design claude's proposal, accepted with the rest; it was `tune.cplt`
until the owner made it **`tune.cyr`** (2026-09-30: a), CyborgPL syntax
like `CyJet.cyr`, plain `const`s only, as `.cyr` (was `.cplr`) is the ending for
CyborgPL-syntax files that never run.

## Explicit control and raw GPU code (owner, 2026-10-01)

Beside the automatic loops, a program may take the GPU in hand: **std**
moves data to GPU memory and back, launches GPU code with N threads, and
forces a loop onto the GPU over the prover's choice (as `for para` forces
limbs); Apple's GPU limb, **`metal`** (owner, 2026-10-22: "Just `metal`"; one of
three hand-written-GPU limbs, 2026-10-06), has **`inline.MSL { … }`**, Metal Shading
Language written inline — a `no_check` site like `inline.ASM`, checked by
Apple's Metal compiler at build time. A block runs where it is
written, as `inline.ASM` does, and needs no std (owner): reuse is a
function holding it.

The **thread count** — how many copies of the kernel run, which plain
Metal sets from the host with `dispatchThreads` — comes **after the name**
(owner, 2026-10-01: b), as any number:

```
no_check inline.MSL threads std::list.length(lend $xs) {
    kernel void twice(device int *xs [[buffer(0)]], uint id [[thread_position_in_grid]]) {
        xs[id] = xs[id] * 2;
    }
} with [lend.mut $xs -> buffer(0)];
```

How the block runs comes first; `with` holds what it touches. The code
inside the braces is Metal's, passed to Apple's compiler unchanged. A lent
list binds with **`->`** (`-->` until 2026-10-07), as in `inline.ASM`: `lend.mut` says the kernel
may write it, and the changes are in `'xs'` when the block ends (owner,
2026-10-01: a). The slot is named as Metal names it, `buffer(0)`,
matching the kernel's `[[buffer(0)]]`, as `inline.ASM` writes ARM's `x0`
(owner, 2026-10-01).

**Settled 2026-10-22 (owner):**
- **A single value binds with `copy`**: `copy $k -> buffer(1)`, a Metal `constant`.
- **One kernel per block**; helper functions without `kernel` are fine. Shared Metal code lives in `.metal` files,
  pulled in with Metal's own `#include "shaders/common.metal"`, the path from the kit's root, the registry refusing
  one outside the kit.
- **Numbers, lists of numbers, `layout.C struct`s and lists of them bind**, handed over as they are (Apple's GPU
  shares the CPU's memory), and **kayo writes the Metal declaration of each bound `layout.C struct`** at the top of
  the block, so it is written once:

```
layout.C struct Body { x: f32, y: f32, vx: f32, vy: f32 }

no_check inline.MSL threads $n {
    #include "shaders/common.metal"
    kernel void step(device Body *bodies [[buffer(0)]], constant float &dt [[buffer(1)]], uint id [[thread_position_in_grid]]) {
        bodies[id].x += bodies[id].vx * dt;
    }
} with [lend.mut $bodies -> buffer(0), copy $dt -> buffer(1)];
```

- The limb is **Metal** (2026-10-06), listed as `metal { … }` under `my limbs:`.
- **A lent list is read-only in the kernel** (owner, 2026-10-25: "(a)"; not built): a `lend` binding's kernel parameter is
  `const device …` or `constant …`, or the block is refused (`metal::lent_but_written`); `lend.mut` is for a kernel that writes.
- **Apple's defaults unless a block says otherwise** (owner, 2026-10-25: "yes"; not built): `inline.MSL` builds with fast
  math and signed overflow undefined; `inline.MSL.precise` asks Metal's safe math mode, precise functions, no fused
  multiply-add; `inline.MSL.wrap` makes signed overflow wrap (`-fwrapv`); `inline.MSL.precise.wrap` both.
- **No copy when page-aligned** (owner, 2026-10-25; not built): Metal shares memory without a copy only when it starts on
  a page and spans whole pages (16 KB on this Mac). A bound list that is page-aligned goes without a copy; otherwise it
  is copied in, and back after `lend.mut`. Precog gives page-aligned memory to lists of 16 KB or more that can reach a
  block, and, unable to follow one, to it too; `std::list.page_aligned[T]` and `std::list.not_page_aligned[T]` say it
  outright. The measurements: `kayo/metal/bench/RESULTS.md`.

## Open

- the prover's offload threshold: how many iterations, of what body, before
  the launch cost is worth it — a number for the bench.
- A loop-nest model (the polyhedral kind) for tiling and fusion — a
  fourth IR, **built with GPU loops** (owner, 2026-10-01: "Could we have
  LNIR when we do GPU loops? It's a hobby lang, and I think that's more
  fun. Only if it doesn't reduce perf"; it had been "only if offload gets
  serious", `precog.md`, three IRs). **It never makes a program slower**:
  it changes a loop only when the prover's estimate, or a `tune.cyr`
  measurement, says the change wins, and otherwise passes the loop
  through as MIR would have it; the bench builds with and without it to
  check. It **it is our own, in Rust, not MLIR** (owner, 2026-10-01: "our
  own"), and it is called **LNIR**, *Loop-Nest IR* (owner): built from MIR after the prover, it reads the prover's proofs directly,
  so it can transform loops MLIR's analysis could not see into; MLIR's
  ready-made passes would be faster to get, and it has no Metal output
  anyway (Metal is kayo's own text).
