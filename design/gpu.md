# GPU offload

Opened 2026-09-22. Part of v1 (owner, 2026-09-21). The rule is
`parallelism.md`'s, extended: the programmer writes `for`, Precog
decides CPU cores or GPU, `for seq` says no. No GPU syntax. Underneath,
the loop body is compiled for the GPU target (by LLVM for Vulkan, by
CCA's own Metal generator for Metal) and the runtime launches it and
moves the data. This document is what Precog needs to
decide, and what the runtime does.

## What a body may contain — plain values, and failures (owner: b)

A GPU runs thousands of copies of the body at once, each on its own
slice of the range, with no operating system underneath. Precog may
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

Never offloadable: `extern` calls, `std::input` / `print`, `inline.Assembly`,
`!check` sites (nothing proven, nothing offloadable), text
(allocation per `++` on a GPU is 10–100× slower than on the CPU; a
text loop is never worth offloading), `std::uenr` and `deci` (software
arithmetic with allocation). A loop that touches any of these stays on
the CPU, silently — that is not an error, it is Precog deciding.

Proposed: (a) plain values only; (b) a plus failures; (c) b plus text.
Owner: b.

## How data crosses — Precog keeps it there (owner: b)

On unified memory (the M5) a lent list is already where the GPU can
see it; nothing moves. On a discrete GPU a list crosses a bus in and
its changes cross back, and the copy can cost more than the loop.
Precog decides the crossings the way it decides everything: it sees
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
to test on. Precog's decisions are the same for any GPU; a target is a
runtime module for launching and memory plus a the manifest (`Synthia.ttsr` / `CCA.cplr`) line, so
CUDA later is an addition, not a change. Proposed: Metal only; Metal +
CUDA; all three. Owner: Metal and Vulkan.

**Metal is built first, Vulkan after** (owner, 2026-10-01: "it's gonna be
Metal before Vulkan"): Metal is this machine's GPU, so it can be built
and tested here.
The first public GitHub release needs only Metal (owner, 2026-10-01).

**How each is compiled** (owner, 2026-09-30: direct text). Vulkan: LLVM
compiles the body to SPIR-V. Metal: CCA writes Metal Shading Language
text straight from its own MIR, which Apple's compiler then builds. This
takes the place of making SPIR-V once and converting it. Direct text can
use Metal-only features (operations across a SIMD group, threadgroup
memory) and carry Precog's facts into the code, where a converter loses
some of both. It costs a second code generator, and so a second place
Precog's proofs must be carried correctly and tested. The speed edge is
expected, not measured. No C++: the Metal runtime is reached from Rust.

## The report (owner: yes)

A line per `for` Precog considered — offloaded, parallel on CPU cores,
sequential and why (`diagnostics.md`).

## Measured once, on purpose: `cca tune` (owner, 2026-09-28; CCA's since 2026-09-30)

By default a cost model decides CPU or GPU: the loop's work (trips times
the body's cost) against the GPU's launch cost. The owner asked whether
Precog could instead run both at compile time when the building machine
is the target. Not inside a build: Precog uses no wall-clock
(`reproducible-builds.md`), so a build never times anything. Instead, as
profile-guided optimisation, a separate command measures once:

```
cca tune              # builds a measuring version, runs it on this machine with the inputs given,
                      # times each undecided loop on CPU and GPU, and writes tune.cplr
cca build --final     # reads tune.cplr as an input, like the lock file
```

**The measuring is CCA's** (owner, 2026-09-30: a): CCA's package manager
can do whatever Synthia can, and Synthia does what a CCA user would, so
`synthia tune` runs `cca tune`. **Synthia also shows the results
readably** (owner: "synthia should have a readable version"), as
`synthia show --readable` does a manifest's times; its spelling is open.
Proposed and not chosen: both implementing the measuring; Synthia only,
as this section first had it (written 2026-09-28, when Synthia was to be
CyborgPL's one package manager).

`tune.cplr` records the target, the compiler version and, per loop, the
winner and its times. It is committed with the code, so a build of the
same source and the same file is always the same program. An entry is
ignored, and the build says so, when its loop has changed since it was
measured, or when the file was made for a different target. Then the cost
model decides. Proposed alternative: the cost model only. The file name
was design claude's proposal, accepted with the rest; it was `tune.cplt`
until the owner made it **`tune.cplr`** (2026-09-30: a), CyborgPL syntax
like `CCA.cplr`, plain `const`s only, as `.cplr` is the extension for
CyborgPL-syntax tooling files.

## Explicit control and raw GPU code (owner, 2026-10-01)

Beside the automatic loops, a program may take the GPU in hand: **std**
moves data to GPU memory and back, launches GPU code with N threads, and
forces a loop onto the GPU over Precog's choice (as `for para` forces
cores); the **base language** has **`inline.MSL { … }`**, Metal Shading
Language written inline — a `!check` site like `inline.Assembly`, checked by
Apple's Metal compiler at build time. A block runs where it is
written, as `inline.Assembly` does, and needs no std (owner): reuse is a
function holding it.

The **thread count** — how many copies of the kernel run, which plain
Metal sets from the host with `dispatchThreads` — comes **after the name**
(owner, 2026-10-01: b), as any number:

```
!check inline.MSL threads list.length[lend 'xs'] {
    kernel void twice(device int *xs [[buffer(0)]], uint id [[thread_position_in_grid]]) {
        xs[id] = xs[id] * 2;
    }
} with [lend.mut 'xs' --> buffer(0)];
```

How the block runs comes first; `with` holds what it touches. The code
inside the braces is Metal's, passed to Apple's compiler unchanged. A lent
list binds with **`-->`**, as in `inline.Assembly`: `lend.mut` says the kernel
may write it, and the changes are in `'xs'` when the block ends (owner,
2026-10-01: a). The slot is named as Metal names it, `buffer(0)`,
matching the kernel's `[[buffer(0)]]`, as `inline.Assembly` writes ARM's `x0`
(owner, 2026-10-01).

## Open

- Precog's offload threshold: how many iterations, of what body, before
  the launch cost is worth it — a number for the bench.
- A loop-nest model (the polyhedral kind) for tiling and fusion — a
  fourth IR, **built with GPU loops** (owner, 2026-10-01: "Could we have
  LNIR when we do GPU loops? It's a hobby lang, and I think that's more
  fun. Only if it doesn't reduce perf"; it had been "only if offload gets
  serious", `precog.md`, three IRs). **It never makes a program slower**:
  it changes a loop only when Precog's estimate, or a `tune.cplr`
  measurement, says the change wins, and otherwise passes the loop
  through as MIR would have it; the bench builds with and without it to
  check. It **it is our own, in Rust, not MLIR** (owner, 2026-10-01: "our
  own"), and it is called **LNIR**, *Loop-Nest IR* (owner): built from MIR after Precog, it reads Precog's proofs directly,
  so it can transform loops MLIR's analysis could not see into; MLIR's
  ready-made passes would be faster to get, and it has no Metal output
  anyway (Metal is CCA's own text).
