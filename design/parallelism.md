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
(`change 'total' = 'total' plus 'x';`) may run in parallel, each core folding
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
    std::print.stored.stdout['line', \n];
}
```

A loop that prints, reads input or calls C stays in order by itself
(owner, 2026-09-30: a); `seq` is then for loops Precog would run in
parallel but you want in order, like an exact `bin` total. The other way,
**`for para`** forces a loop parallel (owner: "`para`"). It overrides
only that automatic order (owner: "`a`, because `nocheck` exists"): the
`!check` every C call already carries is also the writer's word that the
C is safe on many cores at once, and printing under `para` may come out in
any order. A conflict Precog can see — two iterations writing the same
place, other than a reduction — stays an error, `para` or not.

This was the entire parallelism syntax of v1 — one word for saying no —
until `para` joined it.
(`seq` and `para` sit before `temp`/`perm`: `for seq perm 'i'` — see
`control-flow.md`. The old chain spelling had them the other way round.)

## No cheating: `for !cheat` (owner, 2026-10-01)

```
for !cheat 'i': int64 in range[1, 'n'] { change 'acc' = 'acc' plus 'i'; }
for !cheat seq 'i' in lend 'xs' { … }
for !cheat para 'i' in range[1, 'n'] { … }
```

**`!cheat`, "no cheat"** (`!c` until 2026-10-06): every round's work really happens at run time. The
compiler may not replace the loop with its answer (clang turns
`acc += i` over a known range into n(n+1)/2, and CCA's slices fold the
same way) nor drop rounds whose results look unused. It may still do the
work faster — SIMD, or across cores — so `!cheat` is about whether the work
happens, not how; add `seq` for one round at a time, in order (owner:
"Skipping work, because `seq` could be used with `!c`"). Written in
CyborgPL for timing code and the like, where today an empty
`inline.Assembly` passing the value through a register does the job.

**The strongest modifier comes first** (owner: "yeah, strongest first"):
`!cheat` changes what the loop must do, `seq` and `para` only how its rounds
are scheduled, so `for !cheat seq`, never `for seq !cheat` (refused). It is the
rule `.useful` already follows in `print.useful.stdout` and
`to.useful.str.utf8`. `!cheat` is a token of its own, not a reserved name.
`examples/for_no_cheat.cpls`; reject test for_no_cheat_order.

## The GPU (owner, 2026-09-21: automatic)

Same rule, one more place to run: the programmer writes `for`, Precog
decides CPU cores or GPU, `for seq` says no. No GPU syntax. Underneath,
loop bodies are compiled by LLVM for the GPU target and a runtime launches
them and moves the data. Precog offloads only when the iterations are
provably independent (the existing test), the trip count is worth the
transfer, and the body stays within what the GPU can do. Part of v1.
Undesigned beyond this.
