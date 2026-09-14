# Parallelism

Decided 2026-09-14. Concurrency (tasks that pause, resume, and share state —
async, channels) is a separate, undesigned topic; see `DECISIONS.md` open
questions.

## Automatic

The programmer writes `loop.for`. Precog decides whether it runs in
parallel:

- **Whether it can:** the iterations must provably not touch each other's
  data. Ownership makes that a proof — two iterations that would conflict
  cannot both own the same value — rather than the guess a C or Go compiler
  has to make.
- **Whether it should:** Precog usually knows the trip count or its bound;
  a short loop is not worth waking a thread for.

No threads, pools, or locks in user code. There is nothing for a lock to
protect, because only conflict-free loops are parallelised.

## Underneath

The runtime (the `no_std` Rust library in every binary) carries a thread
pool — one thread per core, created at startup, idle when unused — and a
scheduler that hands them slices of a parallelised loop. The OS places
threads on cores; the program never allocates a core. A few hundred lines.

## The escape

A loop that is parallel-safe but must run in order for an external reason
(writing lines to a file, driving a device) opts out with a chain segment:

```
loop.for.seq 'line' in [lend 'lines'] {
    std::print.stdout['line' \n];
}
```

This is the entire parallelism syntax of v1: one word, for saying no.
