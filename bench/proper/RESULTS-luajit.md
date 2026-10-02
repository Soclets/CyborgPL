# LuaJIT against CyborgPL

arm64, Darwin 27.0.0; run 2026-10-02 by design claude, best of 3 after a warm-up,
wall-clock from start to exit. LuaJIT 2.1 (Homebrew), written the proper LuaJIT way:
plain numeric loops for the tracing JIT, and LuaJIT's built-in FFI arrays for the big
arrays. LuaJIT comes with no threads, so it runs on one core; CyborgPL is shown on one
thread (`CYBORG_THREADS=1`) and on three (design claude's share of the cores). CyborgPL is
CCA as built at the pause (f497869). Every answer agrees with C's. Compiler claude and
check claude had just resumed, so expect some noise.

| Program | LuaJIT | CyborgPL, 1 thread | CyborgPL, 3 threads | LuaJIT ÷ CyborgPL (1 / 3) |
|---|---|---|---|---|
| sum (N = 300,000,000) | 0.622 s | 0.092 s | 0.035 s | 6.75× / 17.72× |
| fib (N = 35) | 0.033 s | 0.017 s | 0.016 s | 1.91× / 2.03× |
| lists (N = 20,000,000) | 0.140 s | 0.045 s | 0.039 s | 3.15× / 3.60× |
| sieve (N = 100,000,000) | 0.361 s | 0.578 s | 0.439 s | 0.62× / 0.82× |

LuaJIT starts and stops in 0.002 s, so unlike Julia its times are nearly all work. Its
numbers are doubles, which makes `sum`'s `mod` slow; the sieve is CyborgPL's known slow
one since f497869 (`bench/RESULTS.md`), and LuaJIT beats it, as Julia did.
