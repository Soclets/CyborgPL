# The proper benchmark: results

Newest first. The runner writes `latest.md`; this file is kept by hand.

## 2026-10-15: Zag, C, Rust and Go, proper and parallel

Run at 17:47 (+07) with `ZOMP=… python3 run.py` (Zomp's release build of compiler claude's clone, `ZDK 1
Zomp`), every session paused. The owner: "Do benchmark that uses proper Zag, proper C, proper Rust, and proper
Go. Make para versions too." Each program now has a **proper** version (one thread, what an expert writes first)
and a **parallel** one (every core, with the language's own threads: pthreads, `std::thread::scope`, goroutines,
Zag's `for para` and `concur`). C is at `-O3`. Every case built (Zag as final builds, every site proven) and
agreed with C.

| | sum | fib | lists | sieve |
|---|---|---|---|---|
| **Zag, proper, 1 thread** | 0.091 s | 0.016 s | 0.038 s | 0.349 s |
| C, proper | 0.093 s | 0.015 s | 0.036 s | 0.301 s |
| Rust, proper | 0.115 s | 0.016 s | 0.034 s | 0.354 s |
| Go, proper | 0.144 s | 0.023 s | 0.068 s | 0.335 s |
| **Zag, proper as written** (Zomp splits proven loops itself) | 0.020 s | 0.016 s | 0.030 s | 0.221 s |
| **Zag, parallel** | 0.021 s | 0.008 s | 0.024 s | 0.225 s |
| C, parallel | 0.020 s | 0.008 s | 0.023 s | 0.124 s |
| Rust, parallel | 0.024 s | 0.008 s | 0.023 s | 0.120 s |
| Go, parallel | 0.040 s | 0.007 s | 0.023 s | 0.120 s |

- **One thread:** Zag ties C on sum, fib and lists, and beats Go on all four. **Sieve is not like for like**:
  C's, Rust's and Go's proper sieves cross off only up to √N and count in a second pass, while Zag's (the one
  from `bench/`) walks every p to N and counts as it goes. That, more than the compiler, is C's 14% there.
- **Zag's proper code, as written,** already runs sum on every core (0.020 s, level with C's hand-threaded
  version) without the writer asking: Zomp proved the loop safe to split.
- **In parallel,** Zag is level with C and Rust on sum, fib and lists. **Sieve is the gap** (0.225 s against
  0.12): the others use a segmented sieve, each thread crossing off its own part of the array, while Zag's
  parallel sieve splits each prime's crossing-off loop across the cores, prime by prime, which pays the split
  thousands of times and walks the whole array each time. A segmented sieve needs threads writing disjoint
  parts of one list, which `for para` refuses unless the prover can see the parts never meet.

### sum (N = 300,000,000)

| Version | Time |
|---|---|
| Zag, proper, as written (Zomp splits proven loops itself) | 0.020 s |
| C, parallel | 0.020 s |
| Zag, parallel | 0.021 s |
| Rust, parallel | 0.024 s |
| Go, parallel | 0.040 s |
| Zag, proper, 1 thread | 0.091 s |
| C (clang -O3), proper | 0.093 s |
| Rust (rustc opt-level=3), proper | 0.115 s |
| Go, proper | 0.144 s |

### fib (N = 35)

| Version | Time |
|---|---|
| Go, parallel | 0.007 s |
| Rust, parallel | 0.008 s |
| C, parallel | 0.008 s |
| Zag, parallel | 0.008 s |
| C (clang -O3), proper | 0.015 s |
| Zag, proper, 1 thread | 0.016 s |
| Rust (rustc opt-level=3), proper | 0.016 s |
| Zag, proper, as written (Zomp splits proven loops itself) | 0.016 s |
| Go, proper | 0.023 s |

### lists (N = 20,000,000)

| Version | Time |
|---|---|
| Rust, parallel | 0.023 s |
| C, parallel | 0.023 s |
| Go, parallel | 0.023 s |
| Zag, parallel | 0.024 s |
| Zag, proper, as written (Zomp splits proven loops itself) | 0.030 s |
| Rust (rustc opt-level=3), proper | 0.034 s |
| C (clang -O3), proper | 0.036 s |
| Zag, proper, 1 thread | 0.038 s |
| Go, proper | 0.068 s |

### sieve (N = 100,000,000)

| Version | Time |
|---|---|
| Rust, parallel | 0.120 s |
| Go, parallel | 0.120 s |
| C, parallel | 0.124 s |
| Zag, proper, as written (Zomp splits proven loops itself) | 0.221 s |
| Zag, parallel | 0.225 s |
| C (clang -O3), proper | 0.301 s |
| Go, proper | 0.335 s |
| Zag, proper, 1 thread | 0.349 s |
| Rust (rustc opt-level=3), proper | 0.354 s |

## 2026-10-01: the first run (CyborgPL)


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
