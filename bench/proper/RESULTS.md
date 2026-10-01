# The proper benchmark: results

arm64, Darwin 27.0.0; run 2026-10-01 by `bench/proper/run.py`. Best of 3 after a warm-up, wall-clock from start to exit. Every answer is checked against C's.

## sum (N = 300,000,000)

| Language | Time | vs CyborgPL |
|---|---|---|
| CyborgPL (CCA, final build) | 0.020 s | 1.00× |
| C (clang -O2) | 0.021 s | 1.04× |
| Rust (rustc, opt-level 3) | 0.024 s | 1.20× |
| Go | 0.041 s | 2.11× |
| Java | 0.066 s | 3.34× |
| Python (CPython) | 1.808 s | 91.78× |

## fib (N = 35)

| Language | Time | vs CyborgPL |
|---|---|---|
| Rust (rustc, opt-level 3) | 0.016 s | 0.97× |
| CyborgPL (CCA, final build) | 0.017 s | 1.00× |
| C (clang -O2) | 0.017 s | 1.05× |
| Go | 0.023 s | 1.36× |
| Java | 0.043 s | 2.60× |
| Python (CPython) | 0.533 s | 32.11× |

## lists (N = 20,000,000)

| Language | Time | vs CyborgPL |
|---|---|---|
| CyborgPL (CCA, final build) | 0.031 s | 1.00× |
| C (clang -O2) | 0.036 s | 1.18× |
| Rust (rustc, opt-level 3) | 0.037 s | 1.18× |
| Go | 0.071 s | 2.30× |
| Java | 0.111 s | 3.61× |
| Python (CPython) | 0.854 s | 27.65× |

## sieve (N = 100,000,000)

| Language | Time | vs CyborgPL |
|---|---|---|
| C (clang -O2) | 0.304 s | 0.71× |
| Go | 0.334 s | 0.78× |
| Python (CPython) | 0.359 s | 0.84× |
| Rust (rustc, opt-level 3) | 0.362 s | 0.85× |
| Java | 0.390 s | 0.92× |
| CyborgPL (CCA, final build) | 0.426 s | 1.00× |

## Notes

- Run while another session (RRx2-TV) was using most of the machine's cores, so the
  threaded versions (every `sum`, and CyborgPL's sliced loops) had fewer free cores than
  the machine has. Rerun on a quiet machine for clean numbers.
- CyborgPL's `sieve` is the slowest here: CCA's last change before the pause (f497869)
  made its sliced inner loop slower, as `bench/RESULTS.md` notes (0.26 s before it).
- Python's `sieve` is fast because slice assignment and `bytearray.count` run in C: the
  proper Python way hands the inner loop to the interpreter's own C code.
