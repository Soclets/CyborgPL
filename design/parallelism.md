# Parallelism

> Revised 2026-10-07: code here is in the new syntax; an old spelling in the prose is history. The old → new table is `revision-2026-10-07.md`.

Decided 2026-09-14. Concurrency (tasks that wait on the outside world —
`concur`, channels, `pick`) is `concurrency.md`, decided 2026-09-22.

## Automatic

The programmer writes `for`. the prover decides whether it runs in
parallel:

- **Whether it can:** the iterations must provably not touch each other's
  data. Ownership makes that a proof — two iterations that would conflict
  cannot both own the same value — rather than the guess a C or Go compiler
  has to make.
- **Whether it should:** the prover usually knows the trip count or its bound;
  a short loop is not worth waking a thread for. **the prover estimates it
  itself** (owner, 2026-09-30: "Bro, Precog exists"), from the trips and
  the work of each, never from a number someone picks or from timing the
  machine — and **a measurement in `tune.cyr` wins where there is one**
  (owner: a), as for the GPU: `synapse tune` times the undecided loops once,
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

the runtime, the runtime (the `no_std` Rust library in every binary), carries a thread
pool — one thread per core, created at startup, idle when unused — and a
scheduler that hands them slices of a parallelised loop. The OS places
threads on cores; the program never allocates a core. A few hundred lines.

## The escape

A loop that is parallel-safe but must run in order for an external reason
(writing lines to a file, driving a device) opts out with one word before the name (revised 2026-09-19; it was the chain segment `loop.temp.seq.for`):

```
for seq $line in lend $lines:
    std::print.stored($line, newln);
end of for
```

A loop that prints, reads input or calls C stays in order by itself
(owner, 2026-09-30: a); `seq` is then for loops the prover would run in
parallel but you want in order, like an exact `bin` total. The other way,
**`for para`** forces a loop parallel (owner: "`para`"). It overrides
only that automatic order (owner: "`a`, because `nocheck` exists"): the
`no_check` every C call already carries is also the writer's word that the
C is safe on many cores at once, and printing under `para` may come out in
any order. A conflict the prover can see — two iterations writing the same
place, other than a reduction — stays an error, `para` or not.

This was the entire parallelism syntax of v1 — one word for saying no —
until `para` joined it.
(`seq` and `para` sit before `temp`/`perm`: `for seq perm 'i'` — see
`control-flow.md`. The old chain spelling had them the other way round.)

## No cheating: `no_cheat` (owner, 2026-10-01; any statement since 2026-10-18)

```
no_skip for $i: int.sign.64 [from: 1, to: $n, by: 1]:
    change $acc = $acc plus $i;
end of for
no_skip for seq $i in lend $xs:
    …
end of for
no_skip for %para $i [from: 1, to: $n, by: 1]:
    …
end of for
```

**`no_cheat`, "no cheat"** (`!c` until 2026-10-06, `!cheat` until 2026-10-07): every round's work really happens at run time. The
compiler may not replace the loop with its answer (clang turns
`acc += i` over a known range into n(n+1)/2, and kayo's slices fold the
same way) nor drop rounds whose results look unused. It may still do the
work faster — SIMD, or across cores — so `no_cheat` is about whether the work
happens, not how; add `seq` for one round at a time, in order (owner:
"Skipping work, because `seq` could be used with `!c`"). Written in
CyborgPL for timing code and the like, where today an empty
`inline.ASM` passing the value through a register does the job.

**The strongest modifier comes first** (owner: "yeah, strongest first"):
`no_cheat` changes what the loop must do, `seq` and `para` only how its rounds
are scheduled, so it read `for no_cheat seq`, never `for seq no_cheat`. **Since 2026-10-18 it goes in front of the `for`
(owner: "(a)"): `no_cheat for seq 'i' …`, and `for no_cheat` is an old spelling** (built in the oracle 2026-10-19). It is the
rule `.useful` already follows in `print.useful.stdout` and
`to.useful.str.utf8`.

**Any statement or block, calls included** (owner, 2026-10-18: "(b)"; built in the oracle 2026-10-19, which reads the word and runs everything anyway; the check word always comes first: `no_check no_cheat for …`, `NO_FUCKING_CHECKS no_cheat …`, `check no_cheat …` (owner, 2026-10-19; `no_check` is the stronger word and the unchecked part should be seen first, and `check` follows for one rule)): `no_cheat let 'f' := fib['n'];`,
`no_cheat { … }`. It goes where `no_check` goes and covers everything the statement runs, calls and recursion
all the way down, a call through a variable and a task started inside included (owner, 2026-10-19: "(a)"): no folding to the answer, no dropped work, and no computing calls that look alike once (what
kayo's fib did after four levels of inlining, `bench/RESULTS.md` 2026-10-18). Faster ways to do the same work stay
allowed.
`examples/for_no_cheat.cy`; reject test for_no_cheat_order.

## The GPU (owner, 2026-09-21: automatic)

Same rule, one more place to run: the programmer writes `for`, the prover
decides CPU cores or GPU, `for seq` says no. No GPU syntax. Underneath,
loop bodies are compiled by LLVM for the GPU target and a runtime launches
them and moves the data. the prover offloads only when the iterations are
provably independent (the existing test), the trip count is worth the
transfer, and the body stays within what the GPU can do. Part of v1.
Undesigned beyond this.
