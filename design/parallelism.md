# Parallelism

Decided 2026-09-14. Concurrency (tasks that wait on the outside world —
`concur`, channels, `pick`) is `concurrency.md`, decided 2026-09-22.

## Automatic

The programmer writes `for`. Precog decides whether it runs in
parallel:

- **Whether it can:** the iterations must provably not touch each other's
  data. Ownership makes that a proof — two iterations that would conflict
  cannot both own the same value — rather than the guess a C or Go compiler
  has to make.
- **Whether it should:** Precog usually knows the trip count or its bound;
  a short loop is not worth waking a thread for. **Precog estimates it
  itself** (owner, 2026-09-30: "Bro, Precog exists"), from the trips and
  the work of each, never from a number someone picks or from timing the
  machine — and **a measurement in `tune.cplr` wins where there is one**
  (owner: a), as for the GPU: `cca tune` times the undecided loops once,
  outside the build (`gpu.md`). A missing or stale entry only costs speed,
  never correctness. Proposed: a fixed trip threshold; the estimate alone
  for cores, tuning for the GPU only.

**Reductions** (owner, 2026-09-30: c): a loop that folds into one variable
(`change 'total' = 'total' + 'x';`) may run in parallel, each core folding
its own slice and the slices combined at the end — for every type,
`bin` and `deci` included. For those the order changes the rounding, so
the same binary may give a total that differs in its last digits from run
to run; integers, exact, give the same answer every time. Proposed: (a) no
reductions; (b) integers only, the answer never changing; design claude
recommended b. Reproducible builds (the same binary) are untouched: this is
about a program's output, which no rule had fixed.

No threads, pools, or locks in user code. There is nothing for a lock to
protect, because only conflict-free loops are parallelised.

## Underneath

The runtime (the `no_std` Rust library in every binary) carries a thread
pool — one thread per core, created at startup, idle when unused — and a
scheduler that hands them slices of a parallelised loop. The OS places
threads on cores; the program never allocates a core. A few hundred lines.

## The escape

A loop that is parallel-safe but must run in order for an external reason
(writing lines to a file, driving a device) opts out with one word before the name (revised 2026-09-19; it was the chain segment `loop.temp.seq.for`):

```
for seq 'line' in lend 'lines' {
    std::print.stdout['line', \n];
}
```

A loop that prints, reads input or calls C stays in order by itself
(owner, 2026-09-30: a); `seq` is then for loops Precog would run in
parallel but you want in order, like an exact `bin` total. The other way,
**`for para`** forces a loop parallel (owner: "`para`"); what it may
override is open.

This was the entire parallelism syntax of v1 — one word for saying no —
until `para` joined it.
(`seq` sits after `temp`/`perm` in the chain — see `control-flow.md`.)

## The GPU (owner, 2026-09-21: automatic)

Same rule, one more place to run: the programmer writes `for`, Precog
decides CPU cores or GPU, `for seq` says no. No GPU syntax. Underneath,
loop bodies are compiled by LLVM for the GPU target and a runtime launches
them and moves the data. Precog offloads only when the iterations are
provably independent (the existing test), the trip count is worth the
transfer, and the body stays within what the GPU can do. Part of v1.
Undesigned beyond this.
