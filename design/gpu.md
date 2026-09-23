# GPU offload

Opened 2026-09-22. Part of v1 (owner, 2026-09-21). The rule is
`parallelism.md`'s, extended: the programmer writes `for`, Precog
decides CPU cores or GPU, `for seq` says no. No GPU syntax. Underneath,
the loop body is compiled by LLVM for the GPU target and the runtime
launches it and moves the data. This document is what Precog needs to
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

Never offloadable: `extern` calls, `std::input` / `print`, `inline.ASM`,
`nocheck` sites (nothing proven, nothing offloadable), text
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
runtime module for launching and memory plus a `Synthia.toml` line, so
CUDA later is an addition, not a change. Proposed: Metal only; Metal +
CUDA; all three. Owner: Metal and Vulkan.

## The report (owner: yes)

A line per `for` Precog considered — offloaded, parallel on cores,
sequential and why (`diagnostics.md`).

## Open

- Precog's offload threshold: how many iterations, of what body, before
  the launch cost is worth it — a number for the bench.
- A loop-nest model (the polyhedral kind) for tiling and fusion — a
  fourth IR only if offload gets serious (`precog.md`, three IRs).
