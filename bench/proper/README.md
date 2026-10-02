# The proper benchmark

The same four programs as [`bench/`](../README.md), in eight languages — C, Rust, Go,
Java, Python, Julia, LuaJIT and CyborgPL — each written **the way that language's own practice says**,
using only what the language comes with (owner, 2026-10-01). `bench/` asks every
language to write the plain loop; this one asks each to write what an expert in it
would. Results: [`RESULTS.md`](RESULTS.md).

| Program | What it does | N |
|---|---|---|
| `sum` | Σ (i × i mod 7) for i = 1..N, then mod 1 000 000 007 | 300 000 000 |
| `fib` | naive recursive Fibonacci | 35 |
| `lists` | a list of N numbers (i mod 1000), summed 10 times | 20 000 000 |
| `sieve` | the primes up to N (sieve of Eratosthenes) | 100 000 000 |

## The rules

- **Same algorithm, same answer.** N comes from the environment, the answer's low byte
  is the exit code, and every answer is checked against C's. `fib` stays naive
  recursion: that is the program.
- **Only what the language comes with**: its standard library and its usual compiler
  settings. No NumPy, no Rayon, no OpenMP.
- **Threads are not capped.** A version uses every core when its language's usual
  tools make that the natural way to write the loop.
- Time is wall-clock from process start to exit, best of 3 after one untimed warm-up.

## What "proper" meant in each

| | `sum` | `lists` | `sieve` |
|---|---|---|---|
| **C** | pthreads, one per core | one `malloc` of known size | `calloc` bytes |
| **Rust** | `std::thread::scope`, iterator sums | `collect` into a `Vec`, `iter().sum()` | `Vec<bool>`, `step_by` |
| **Go** | a goroutine per core, `sync.WaitGroup` | `make` with full capacity, `append` | `[]bool` |
| **Java** | `LongStream…parallel()` | `long[]` from a `LongStream`, `Arrays.stream(…).sum()` | `boolean[]` |
| **Python** | `multiprocessing.Pool`, a worker per core | list comprehension, built-in `sum` | `bytearray`, slice assignment, `count` |
| **Julia** | a `Threads.@spawn` task per thread (`-t auto`) | a comprehension, `sum` | `Vector{Bool}`, range assignment |
| **LuaJIT** | one plain loop (no threads come with it) | an FFI `int32_t` array of known size | an FFI byte array |
| **CyborgPL** | `for` loops; CCA splits them across every core itself when Precog proves it safe | `for` | `for` |

CyborgPL's versions are the ones in `bench/`: its proper practice is the plain `for`,
and its compiler does the rest. Unlike `bench/`, nothing sets `CYBORG_THREADS` here.

## Running it

```
python3 run.py               # all seven
python3 run.py c cyborgpl    # only these
python3 run.py --quick       # tiny sizes: does everything build and agree?
```

It needs CCA built (`cca/target/release/cca`, or `CCA=`), clang, rustc, Go, a JDK,
Python 3 and Julia.
