# Julia against CyborgPL, one core each

arm64, Darwin 27.0.0; run 2026-10-02 by design claude, best of 3 after a warm-up,
wall-clock from start to exit. Both capped to one thread (the owner: "cap to one core,
then"): Julia `-t 1 --gcthreads=1`, CyborgPL `CYBORG_THREADS=1`. Every answer agrees
with C's (`sum`'s from its closed form, the rest from the single-threaded C programs).
RRx2-TV had two solver runs going at the time, so expect some noise.

| Program | Julia | CyborgPL | Julia ÷ CyborgPL |
|---|---|---|---|
| sum (N = 300,000,000) | 0.248 s | 0.094 s | 2.64× |
| fib (N = 35) | 0.107 s | 0.018 s | 5.94× |
| lists (N = 20,000,000) | 0.169 s | 0.043 s | 3.93× |
| sieve (N = 100,000,000) | 0.478 s | 0.588 s | 0.81× |

Julia starts and stops in 0.076 s with nothing to run, and compiles each function the
first time it is called, inside the timed run. Without the 0.076 s the ratios are 1.83×,
1.72×, 2.16× and 0.68×: Julia's compiled code is about twice as slow here, apart from
the sieve, where it wins outright. CyborgPL's sieve is the known slow one since CCA's
last change before the pause (f497869; `bench/RESULTS.md`).

The Julia programs are each folder's `.jl`; `run.py` runs them with `-t auto` for the
uncapped proper bench.
