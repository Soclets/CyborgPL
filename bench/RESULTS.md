# Benchmark results

Machine: arm64 (Apple Silicon, 10 cores: 4 performance, 6 efficiency), Darwin 27.0.0. Best of 3 runs, wall-clock from process start to exit (startup included); runs past 60 s are timeouts. Every answer is checked against C's, and every one agreed.

## 2026-10-19, later: run time with cheat and no_cheat (owner, through website claude)

The owner's words, relayed by website claude: "Ask design claude to run a fresh one, with cheat and no_cheat".
`python3 bench/run.py cyborgpl cyborgpl1 cyborgpl_nc cyborgpl1_nc c fortran go zig rust`; kayo `CyDT f84c8a7a
KAYO\n` (it holds every kayo commit to date, the last 67aa5472); final builds only (every site proven); best of 3;
every answer agreed with C; load average about 2.2 (compiler, check and std claude paused).

- **Cheat** is the program as written. kayo may do anything that gives the right answer (owner, 2026-10-18: any pass
  is allowed): fib, for one, is a 93-entry Precog table, every answer worked out while building, so a run reads one entry.
- **no_cheat** is the same program with `no_cheat` in front of the work (`no_cheat let 'code' := run[] otherwise 2;`
  in `MAIN`, files `{b}_no_cheat.cy`). It covers every call down, so the work written really runs: nothing folded
  to its answer, dropped, or shared between look-alike calls, and no tables. Inlining, SIMD and cores stay allowed.
  It is applied the same way to all four programs.

| | sum | fib | lists | sieve |
|---|---|---|---|---|
| CyborgPL, 3 threads | 0.033 s | 0.002 s | 0.039 s | 0.166 s |
| CyborgPL, 1 thread | 0.091 s | 0.002 s | 0.041 s | 0.302 s |
| CyborgPL no_cheat, 3 threads | 0.050 s | 0.012 s | 0.040 s | 0.204 s |
| CyborgPL no_cheat, 1 thread | 0.142 s | 0.012 s | 0.079 s | 0.341 s |
| C (clang -O3) | 0.094 s | 0.017 s | 0.042 s | 0.413 s |
| Rust (opt-level=3) | 0.174 s | 0.017 s | 0.053 s | 0.397 s |
| Go | 0.145 s | 0.024 s | 0.111 s | 0.386 s |
| Zig (ReleaseFast) | 0.148 s | 0.016 s | 0.071 s | 0.367 s |
| Fortran (gfortran -O3) | 0.135 s | 0.003 s | 0.091 s | 0.541 s |

Each language's time over single-threaded no_cheat CyborgPL's (above 1 means CyborgPL is faster):

| | sum | fib | lists | sieve |
|---|---|---|---|---|
| C | 0.66× | 1.42× | 0.53× | 1.21× |
| Rust | 1.23× | 1.42× | 0.67× | 1.16× |
| Go | 1.02× | 2.00× | 1.41× | 1.13× |
| Zig | 1.04× | 1.33× | 0.90× | 1.08× |
| Fortran | 0.95× | 0.25× | 1.15× | 1.59× |

What no_cheat costs, its time over the cheat build's: 3 threads sum 1.52×, fib 6.00×, lists 1.03×, sieve 1.23×;
1 thread sum 1.56×, fib 6.00×, lists 1.93×, sieve 1.13×.

- **no_cheat fib is the like-for-like call-speed number**, measured: its time grows 1.604× per N (N = 38 to 43, one
  thread), clang's 1.609×, φ being 1.618, so it makes every call; it is still about 1.4× faster than C's from
  inlining. The cheat build's 0.002 s is the table.
- **Fortran's fib is not like-for-like either**: gfortran -O3 gives the right answers (checked at N = 30, 38, 43, 50
  against C) in flat time, 0.004 s at both N = 38 and N = 43, so it does far less work than the recursion written.
  How, design claude did not look.
- **Both no_cheat rows use the cores**: CPU over wall time 2.86 for sum on 3 threads, 0.99 on 1.
- **Where no_cheat costs more than it should**, kayo gaps (owed to compiler claude when CyborgPL resumes; it is paused):
  - sum (and likely sieve): under `no_cheat` kayo keeps the loop's step `by 1` as a value worked out at run time, so
    the loop takes the general form (signed step, 128-bit trip-count arithmetic; seen in the machine code). Knowing
    the step is not skipping work, so this is slower than the ruling asks.
  - lists on one thread, 1.93×: design claude's reading, not verified in the IR: the cheat build sums `'xs'` once
    for the 10 identical rounds, which `no_cheat` rightly forbids; C's 0.042 s suggests clang does the same.
  - The build report under `no_cheat` lists some functions and loops twice, says sum's loop "stays in order" though
    it is split across cores, and gives fib's missing table the wrong reason ("a check … is not proven" instead of
    `no_cheat`).

## 2026-10-19: compile time, source to final executable (owner, through website claude)

For the website's "Don't script, don't iterate, but fucking wait." section. `python3 bench/compile_time.py`; kayo
`CyDT f84c8a7a KAYO\n`; best of 3 cold builds per program, wall-clock, one build at a time, load average about 2.
Go and Zig get a fresh, empty cache for every build (their standard libraries compiled again each time); clang,
rustc, gfortran and kayo keep no cache. CyborgPL's is a final build: Precog proves every check site before LLVM runs.

| Language | sum | fib | lists | sieve | total |
|---|---|---|---|---|---|
| CyborgPL (`kayo build --final`) | 2.11 s | 0.10 s | 5.48 s | 10.49 s | **18.18 s** |
| Zig (`-O ReleaseFast`, empty cache) | 4.38 s | 4.39 s | 4.55 s | 4.52 s | 17.85 s |
| Go (`go build`, empty `GOCACHE`) | 1.28 s | 1.26 s | 1.24 s | 1.25 s | 5.02 s |
| Rust (`rustc -C opt-level=3`) | 0.08 s | 0.08 s | 0.10 s | 0.08 s | 0.33 s |
| Fortran (`gfortran -O3`) | 0.07 s | 0.08 s | 0.08 s | 0.07 s | 0.29 s |
| C (`clang -O3`) | 0.04 s | 0.04 s | 0.04 s | 0.04 s | 0.16 s |

- CyborgPL is the slowest to build, about 110× C on these four, as designed ("very slow at the development, fast &
  safe at the final product"); its time goes where the program has the most to prove: sieve's 22 sites with a
  list and nested loops take 10.5 s, fib's 8 take 0.1 s.
- Zig's total is close only because an empty cache makes it rebuild its standard library every time; Go likewise.
  With warm caches both would drop far below; kayo has no cache to warm.
- Run times for the same programs are in the sections below; the fastest builds (C, Rust) are not the fastest programs.

## 2026-10-18: CyborgPL against C, Rust, Go, Zig and Fortran (owner: "run a fresh one")

kayo `CyDT 1d7a651a KAYO\n`, built fresh in the design clone (release, with the recursive-inlining LLVM pass);
final builds only; best of 3; every answer agreed with C. Languages chosen by the owner: "Do C, Fortran, Go,
CyborgPL, Zig, and Rust."

| | sum | fib | lists | sieve |
|---|---|---|---|---|
| CyborgPL, 3 threads | 0.033 s | **0.002 s** | 0.035 s | **0.242 s** |
| CyborgPL, 1 thread | 0.088 s | 0.003 s | 0.038 s | 0.354 s |
| C (clang -O3) | 0.089 s | 0.014 s | 0.037 s | 0.382 s |
| Rust (opt-level=3) | 0.163 s | 0.016 s | 0.048 s | 0.374 s |
| Go | 0.137 s | 0.023 s | 0.098 s | 0.352 s |
| Zig (ReleaseFast) | 0.138 s | 0.016 s | 0.065 s | 0.346 s |
| Fortran (gfortran -O3) | 0.127 s | 0.006 s | 0.084 s | 0.520 s |

Each language's time over single-threaded CyborgPL's (above 1 means CyborgPL is faster):

| | sum | fib | lists | sieve |
|---|---|---|---|---|
| C | 1.01× | 4.67× | 0.97× | 1.08× |
| Rust | 1.85× | 5.33× | 1.26× | 1.06× |
| Go | 1.56× | 7.67× | 2.58× | 0.99× |
| Zig | 1.57× | 5.33× | 1.71× | 0.98× |
| Fortran | 1.44× | 2.00× | 2.21× | 1.47× |

- **fib is the change since 10-17**: 0.016 s then, 0.003 s now on one thread, from kayo's own LLVM pass inlining
  self-recursion four levels deep. It now beats GCC's Fortran (0.006 s), which won fib last time.
  **But it is less work, not the same work faster** (checked 2026-10-18 for the owner and Learning Zig): the answers are
  right for N = 30 to 65, yet kayo's time grows about 1.30× per N (N=60 0.082 s, N=65 0.308 s) where clang's grows about
  1.5× (N=40 0.145 s, N=44 0.987 s; kayo 0.002 s there). The optimized IR has `fib` call itself 10 times, at n-5 to n-10,
  a recurrence growing about 1.36× per N. Design claude's reading, unverified in LLVM's pass logs: with four levels
  inlined, LLVM computes repeated pure calls once. Legal, the answer is the same, but fib is no longer a like-for-like
  call-speed test for CyborgPL. Final builds confirmed (fib 8/8 sites proven, sieve 22/22), and `THREAD_CAP=1` holds
  the 1-thread row to one core (sieve CPU/wall 0.99, against 2.45 at 3).
- Single-threaded, CyborgPL ties C on sum and lists and leads everything else there.
- **Sieve is a near tie on one thread**: Zig 0.346 s and Go 0.352 s against CyborgPL's 0.354 s, within run-to-run noise.

## 2026-10-17: CyborgPL against Fortran (owner: "CyborgPL vs Fortran bench now")

kayo `CyDT c3918678+ KAYO\n` (compiler claude's build); final builds only; best of 3; every answer agreed with C.
Fortran raised from `-O2` to `-O3` first, to match C.

| | sum | fib | lists | sieve |
|---|---|---|---|---|
| CyborgPL, 3 threads | 0.032 s | 0.017 s | 0.036 s | 0.239 s |
| CyborgPL, 1 thread | 0.092 s | 0.016 s | 0.037 s | 0.350 s |
| C (clang -O3) | 0.092 s | 0.015 s | 0.037 s | 0.382 s |
| Fortran (gfortran -O3) | 0.131 s | **0.006 s** | 0.082 s | 0.528 s |

Fortran wins only fib, and that is mostly GCC's back end, not Fortran: the same `fib.c` built with GCC 16
`-O3` took 0.009 s against clang's 0.016 s (best of 5), gfortran's 0.003 s. GCC turns the double recursion
into much less work than LLVM does. On the other three, single-threaded CyborgPL ties C and beats Fortran.

## 2026-10-15, third run: C and C++ at -O3, Zomp with the faster counting loops

Run at 15:37 (+07) with `ZOMP=… ZAG_BENCH_THREADS=10 python3 run.py zag zag1 c python rust go` (Zomp's release
build of `73bad7b9`, `ZDK 1 Zomp`, runtime current; every session paused). Two changes since the second run:
**C and C++ are built with `-O3`**, the level Zomp always uses (owner, asked "If our is O3, are the others O3
too?", then "yes"; they were `-O2`), and Rust's label now says `opt-level=3`, which it always used; and Zomp's
`db17747d` (compiler claude, following design claude's sieve question): a counting loop with a computed step
runs one test a trip instead of two, and its split across threads is decided by a multiply, not a 128-bit
division per prime. Every case built final and agreed with C.

**The 1-thread row is the fair comparison** (the 10-thread row is not like for like). Each language's time over
single-threaded Zag's (above 1 means Zag is faster):

  | | sum | fib | lists | sieve |
  |---|---|---|---|---|
  | C (clang -O3) | 0.99× | 0.94× | 0.95× | 1.10× |
  | Rust (opt-level=3) | 1.86× | 1.00× | 1.23× | 1.12× |
  | Go | 1.54× | 1.44× | 2.49× | 1.01× |
  | Python | 168.58× | 32.50× | 224.92× | 26.09× |

- **Sieve: Zag now leads C by 10% and Rust by 12%, and ties Go** (0.355 s against 0.389, 0.397, 0.357).
- **C at -O3 is a hair ahead on fib and lists** (0.015 against 0.016, 0.037 against 0.039); both are 1–2 ms on
  runs this short, at the edge of what best-of-3 resolves. Sum ties.
- Rust and Go as before: Zag ahead of Rust on sum, lists and sieve, level on fib; ahead of Go on all four.
- The 10-thread sum came out 0.030 s, against 0.021 s in the two runs before; the single-thread rows did not
  move, so likely run-to-run variance in spreading a 0.02-second job over 10 threads.

### sum (N = 300,000,000)

| Language | Time | vs Zag |
|---|---|---|
| Zag (Zomp, final build, 10 threads) | 0.030 s | 1.00× |
| C (clang -O3) | 0.092 s | 3.05× |
| Zag (Zomp, final build, 1 thread) | 0.093 s | 3.06× |
| Go | 0.143 s | 4.73× |
| Rust (rustc opt-level=3) | 0.173 s | 5.74× |
| Python (CPython) | 15.678 s | 518.72× |

### fib (N = 35)

| Language | Time | vs Zag |
|---|---|---|
| C (clang -O3) | 0.015 s | 0.88× |
| Zag (Zomp, final build, 1 thread) | 0.016 s | 0.92× |
| Rust (rustc opt-level=3) | 0.016 s | 0.97× |
| Zag (Zomp, final build, 10 threads) | 0.017 s | 1.00× |
| Go | 0.023 s | 1.38× |
| Python (CPython) | 0.520 s | 30.67× |

### lists (N = 20,000,000)

| Language | Time | vs Zag |
|---|---|---|
| Zag (Zomp, final build, 10 threads) | 0.034 s | 1.00× |
| C (clang -O3) | 0.037 s | 1.10× |
| Zag (Zomp, final build, 1 thread) | 0.039 s | 1.15× |
| Rust (rustc opt-level=3) | 0.048 s | 1.43× |
| Go | 0.097 s | 2.87× |
| Python (CPython) | 8.772 s | 258.50× |

### sieve (N = 100,000,000)

| Language | Time | vs Zag |
|---|---|---|
| Zag (Zomp, final build, 10 threads) | 0.228 s | 1.00× |
| Zag (Zomp, final build, 1 thread) | 0.355 s | 1.55× |
| Go | 0.357 s | 1.56× |
| C (clang -O3) | 0.389 s | 1.70× |
| Rust (rustc opt-level=3) | 0.397 s | 1.74× |
| Python (CPython) | 9.261 s | 40.57× |

## 2026-10-15, second run: Zag against C, Python, Go and Rust, all final builds

Run at 14:48 (+07) with `ZOMP=… ZAG_BENCH_THREADS=10 python3 run.py zag zag1 c python rust go` (Zomp's release
build of `cd41f34d`, runtime rebuilt, in compiler claude's clone), with every session paused and nothing else
running. The runner, rewritten the same day, reported **every case built as asked (Zag as final builds only) and
agreed with C**. This replaces the morning run's sieve, which was a working build (below).

**The nuance** (owner 2026-10-06: "note the nuance"): the 10-thread row is not like for like, since Zag spreads a
proven `for` over the cores by itself and the others are plain single-threaded code. **The 1-thread row is the
fair comparison.** Each language's time over single-threaded Zag's (above 1 means Zag is faster):

  | | sum | fib | lists | sieve |
  |---|---|---|---|---|
  | C (clang -O2) | 1.03× | 1.00× | 0.97× | 1.01× |
  | Rust (rustc -O) | 1.85× | 1.00× | 1.26× | 0.99× |
  | Go | 1.55× | 1.31× | 2.51× | 0.95× |
  | Python | 173.99× | 31.62× | 230.92× | 25.18× |

- **Zag ties C on all four**, and Rust on fib and sieve; it beats Rust on sum and lists, and Go on three.
- **Sieve, now a real final build, ties C and Rust** (0.375 s against 0.378 and 0.372). **Go is ~5% ahead of all
  three**, as compiler claude measured: Go's lead is over C as much as over Zag, most likely in how it gets and
  zeroes the 100 MB array.
- **Why sieve had fallen behind** (compiler claude, after first misdating it): a regression from `c9631fd3`
  (2026-10-14 11:33): a true fact added to every prover question, that a counting loop's counter stays on its
  side of its first value, made the nested counting loops too heavy for the solvers' budget, so the final build
  was refused and the old runner timed the working build. Fixed in `0f3851b2`: each site is asked first without
  that fact. So the 2026-10-12 and 2026-10-13 sieve rows were most likely real final builds after all; only runs
  with Zomp from `c9631fd3` to `0f3851b2`, the morning run's among them, were not.
- With 10 threads: sum 0.021 s (4.33× its single thread), sieve 0.231 s (1.62×).

### sum (N = 300,000,000)

| Language | Time | vs Zag |
|---|---|---|
| Zag (Zomp, final build, 10 threads) | 0.021 s | 1.00× |
| Zag (Zomp, final build, 1 thread) | 0.091 s | 4.24× |
| C (clang -O2) | 0.094 s | 4.37× |
| Go | 0.141 s | 6.57× |
| Rust (rustc -O) | 0.168 s | 7.83× |
| Python (CPython) | 15.833 s | 737.95× |

### fib (N = 35)

| Language | Time | vs Zag |
|---|---|---|
| Zag (Zomp, final build, 1 thread) | 0.016 s | 0.94× |
| C (clang -O2) | 0.016 s | 0.96× |
| Rust (rustc -O) | 0.016 s | 0.97× |
| Zag (Zomp, final build, 10 threads) | 0.016 s | 1.00× |
| Go | 0.021 s | 1.30× |
| Python (CPython) | 0.506 s | 30.77× |

### lists (N = 20,000,000)

| Language | Time | vs Zag |
|---|---|---|
| Zag (Zomp, final build, 10 threads) | 0.029 s | 1.00× |
| C (clang -O2) | 0.038 s | 1.31× |
| Zag (Zomp, final build, 1 thread) | 0.039 s | 1.32× |
| Rust (rustc -O) | 0.049 s | 1.67× |
| Go | 0.098 s | 3.36× |
| Python (CPython) | 9.006 s | 308.31× |

### sieve (N = 100,000,000)

| Language | Time | vs Zag |
|---|---|---|
| Zag (Zomp, final build, 10 threads) | 0.231 s | 1.00× |
| Go | 0.355 s | 1.54× |
| Rust (rustc -O) | 0.372 s | 1.61× |
| Zag (Zomp, final build, 1 thread) | 0.375 s | 1.62× |
| C (clang -O2) | 0.378 s | 1.64× |
| Python (CPython) | 9.443 s | 40.84× |

## 2026-10-15: Zag against C, Python, Go and Rust

Run at 14:10 (+07) with `ZOMP=… CYBORG_BENCH_THREADS=10 python3 run.py zag zag1 c python rust go` (Zomp's release
build of `ab80b214` in compiler claude's clone), with every session paused and nothing else running. The owner, through website claude, approved the numbers for the new site's Home and
then asked for this re-run: "rerun". **The first run under the Zag name**, with the compiler as Zomp and the
programs in today's syntax (`.zg`, `std::`).

**The nuance** (owner 2026-10-06: "note the nuance"):

- **The 10-thread row is not like for like.** Zag spreads a proven `for` loop over the cores by itself; the C,
  Rust, Go and Python programs are the plain single-threaded code anyone would write first. **The 1-thread row is
  the fair comparison.**
- On it, single-threaded Zag against each, as their time over Zag's (above 1 means Zag is faster):

  | | sum | fib | lists | sieve |
  |---|---|---|---|---|
  | C (clang -O2) | 1.04× | 1.06× | 1.05× | (0.99×, working build) |
  | Rust (rustc -O) | 1.88× | 1.00× | 1.30× | (0.96×, working build) |
  | Go | 1.55× | 1.44× | 2.62× | (0.92×, working build) |
  | Python | 178.62× | 32.25× | 219.70× | (23.20×, working build) |

  So Zag ties C on sum, fib and lists, beats Rust on sum and lists and ties it on fib, and beats Go on all three.
- **Correction, the same day: the sieve rows are not a final build.** Asked by the owner why Zag was slower on
  sieve, design claude found Zomp refuses sieve's final build ("the position can be past the end", the inner
  loop's `change 'comp'[pos to.uint64['m' plus 1]] = true;`; 21 of 22 sites proven), and `run.py` then timed the
  working build without saying so. (Compiler claude first dated the refusal to `0f273b45`, which would have made
  the 2026-10-12 and 2026-10-13 sieve rows working builds too; it then found it **was** a regression, from
  `c9631fd3` on 2026-10-14, fixed in `0f3851b2`; see the second run above.) And the check
  is worth only ~1–3%: on 9 interleaved single-thread runs, Zomp as benched 0.445 s median, Zomp with the store
  unchecked 0.438, C 0.439, **Go 0.407**. Zomp ties C either way; **Go is ~7% ahead of C as much as of Zomp**,
  most likely in how Go gets and zeroes the 100 MB array, not in the loop. `run.py` now times Zag only as a
  final build and reports a refused one (`--allow-working` to time it anyway, labelled), and writes its tables
  to `latest.md` instead of over this file. Sum, fib and lists were final builds.

### sum (N = 300,000,000)

| Language | Time | vs Zag |
|---|---|---|
| Zag (Zomp, final build, 10 threads) | 0.021 s | 1.00× |
| Zag (Zomp, final build, 1 thread) | 0.092 s | 4.32× |
| C (clang -O2) | 0.096 s | 4.50× |
| Go | 0.143 s | 6.69× |
| Rust (rustc -O) | 0.173 s | 8.10× |
| Python (CPython) | 16.433 s | 770.56× |

### fib (N = 35)

| Language | Time | vs Zag |
|---|---|---|
| Zag (Zomp, final build, 1 thread) | 0.016 s | 0.91× |
| Rust (rustc -O) | 0.016 s | 0.93× |
| C (clang -O2) | 0.017 s | 0.96× |
| Zag (Zomp, final build, 10 threads) | 0.017 s | 1.00× |
| Go | 0.023 s | 1.30× |
| Python (CPython) | 0.516 s | 29.82× |

### lists (N = 20,000,000)

| Language | Time | vs Zag |
|---|---|---|
| Zag (Zomp, final build, 10 threads) | 0.036 s | 1.00× |
| Zag (Zomp, final build, 1 thread) | 0.040 s | 1.13× |
| C (clang -O2) | 0.042 s | 1.18× |
| Rust (rustc -O) | 0.052 s | 1.46× |
| Go | 0.105 s | 2.92× |
| Python (CPython) | 8.788 s | 244.78× |

### sieve (N = 100,000,000)

| Language | Time | vs Zag |
|---|---|---|
| Zag (Zomp, **working build**: the final build was refused, 10 threads) | 0.238 s | 1.00× |
| Go | 0.362 s | 1.52× |
| Rust (rustc -O) | 0.379 s | 1.59× |
| C (clang -O2) | 0.391 s | 1.64× |
| Zag (Zomp, **working build**: the final build was refused, 1 thread) | 0.394 s | 1.65× |
| Python (CPython) | 9.141 s | 38.36× |

## 2026-10-13: Nerve against LuaJIT

> **Note (2026-10-15):** this run's Nerve sieve rows were, as far as compiler claude could establish, real final builds: the refusal that made a later run time a working build began on 2026-10-14 (`c9631fd3`). See the 2026-10-15 sections.

Run at 08:25 (+07) with `python3 run.py nerve nerve1 c luajit` (the Synapse release build of 08:25 in compiler claude's clone, at `9af932c5`), with nothing else building. The owner: "Bench Nerve against LuaJIT. By AI stereotypes, it should beat Nerve." Nerve is held to 3 threads (design claude's core share), not 10; C is there as the reference every answer is checked against.

**LuaJIT does not beat Nerve on any of the four.** Against single-threaded Nerve, the fair row, LuaJIT is 6.76× slower on sum, 2.02× on fib, 4.63× on lists and 1.96× on sieve (3.18× against Nerve's 3 threads). Why, as design claude reads it (not measured): LuaJIT is still a JIT on a dynamically typed language: its numbers are doubles unless it can prove otherwise, its tables carry type checks the trace has to guard, and it starts compiling only once a loop is hot. Nerve's programs are compiled ahead of time with every type known.

- **The 3-thread row is not like for like**, as before: Nerve spreads a proven `for` over the cores by itself, while the other programs are the plain single-threaded code.
- **fib is LuaJIT's best case** (2.02×): it is all calls, and LuaJIT's traces handle recursion well.
- **sieve**: single-threaded Nerve came out a little behind C in this run (0.414 s against 0.383), where on 2026-10-12 they tied (0.373 against 0.370). Looked into after the owner's "Nerve got fucked on sieve": the same two programs run 15 times each, taking turns, gave Nerve a median of 0.407 s and C 0.412 s (best 0.380 against 0.388), so Nerve is about 1% ahead. The gap was noise: sieve spends its time on memory, and its runs spread from 0.38 s to 0.44 s, more than best-of-3 smooths out.

### sum (N = 300,000,000)

| Language | Time | vs Nerve |
|---|---|---|
| Nerve (Synapse, final build, 3 threads) | 0.033 s | 1.00× |
| Nerve (Synapse, final build, 1 thread) | 0.093 s | 2.79× |
| C (clang -O2) | 0.095 s | 2.87× |
| LuaJIT | 0.627 s | 18.85× |

### fib (N = 35)

| Language | Time | vs Nerve |
|---|---|---|
| Nerve (Synapse, final build, 3 threads) | 0.016 s | 1.00× |
| Nerve (Synapse, final build, 1 thread) | 0.016 s | 1.02× |
| C (clang -O2) | 0.017 s | 1.06× |
| LuaJIT | 0.033 s | 2.06× |

### lists (N = 20,000,000)

| Language | Time | vs Nerve |
|---|---|---|
| Nerve (Synapse, final build, 1 thread) | 0.037 s | 1.00× |
| Nerve (Synapse, final build, 3 threads) | 0.037 s | 1.00× |
| C (clang -O2) | 0.038 s | 1.02× |
| LuaJIT | 0.173 s | 4.63× |

### sieve (N = 100,000,000)

| Language | Time | vs Nerve |
|---|---|---|
| Nerve (Synapse, final build, 3 threads) | 0.255 s | 1.00× |
| C (clang -O2) | 0.383 s | 1.50× |
| Nerve (Synapse, final build, 1 thread) | 0.414 s | 1.62× |
| LuaJIT | 0.812 s | 3.18× |

## 2026-10-12: Nerve against C, Python, Go and Rust

> **Note (2026-10-15):** this run's Nerve sieve rows were, as far as compiler claude could establish, real final builds: the refusal that made a later run time a working build began on 2026-10-14 (`c9631fd3`). See the 2026-10-15 sections.

Run at 17:53 (+07) with `CYBORG_BENCH_THREADS=10 python3 run.py nerve nerve1 c python rust go` (the Synapse release build of 14:04 in compiler claude's clone, committed as `2f8af0b0` at 14:08), with nothing else running. The owner asked, through website claude: "Could you ask design Claude to bench Nerve against C, Python, Go, and Rust?", then chose the uncapped run: "(a)". The first run under the Nerve name, and the first with Python and Rust; the Nerve programs are in today's syntax (`for 'i': int64 from 1 to 'n' by 1`, `int64[min 0, max 1_000_000_000]`).

**The nuance** (noted as in the last run, owner 2026-10-06: "note the nuance"):

- **The 10-thread row is not like for like.** Nerve spreads a proven `for` loop over the cores by itself; the C, Rust, Go and Python programs are the plain single-threaded code anyone would write first. **The 1-thread row is the fair comparison.** On it Nerve ties C on fib and sieve, is a little ahead on sum (0.087 s against 0.090) and a hair behind on lists (0.038 against 0.037), all within run-to-run noise. It beats Rust on sum (Rust 1.85× slower), lists (1.24×) and fib (1.07×), and ties it on sieve (0.373 against 0.370). It beats Go on three; on sieve Go is faster (0.353 against 0.373), as last time.
- **Python is the interpreted one:** CPython is 24× (sieve) to 239× (lists) slower than single-threaded Nerve, and 739× slower than Nerve's 10 threads on sum.
- **fib cannot be split:** it is all function calls, so every Nerve row is one core.
- **sieve is held back by memory, not cores:** 10 threads take it from 0.373 s to 0.227 s, while sum goes from 0.087 s to 0.022 s.
- **Rust's sum** is slower than C's here (0.161 s against 0.090), with `rustc -C opt-level=3` on the same loop; not looked into.

### sum (N = 300,000,000)

| Language | Time | vs Nerve |
|---|---|---|
| Nerve (Synapse, final build, 10 threads) | 0.022 s | 1.00× |
| Nerve (Synapse, final build, 1 thread) | 0.087 s | 3.92× |
| C (clang -O2) | 0.090 s | 4.09× |
| Go | 0.137 s | 6.21× |
| Rust (rustc -O) | 0.161 s | 7.30× |
| Python (CPython) | 16.335 s | 738.74× |

### fib (N = 35)

| Language | Time | vs Nerve |
|---|---|---|
| C (clang -O2) | 0.015 s | 0.91× |
| Nerve (Synapse, final build, 1 thread) | 0.015 s | 0.91× |
| Rust (rustc -O) | 0.016 s | 0.98× |
| Nerve (Synapse, final build, 10 threads) | 0.017 s | 1.00× |
| Go | 0.021 s | 1.29× |
| Python (CPython) | 0.511 s | 30.71× |

### lists (N = 20,000,000)

| Language | Time | vs Nerve |
|---|---|---|
| Nerve (Synapse, final build, 10 threads) | 0.029 s | 1.00× |
| C (clang -O2) | 0.037 s | 1.27× |
| Nerve (Synapse, final build, 1 thread) | 0.038 s | 1.32× |
| Rust (rustc -O) | 0.047 s | 1.64× |
| Go | 0.096 s | 3.30× |
| Python (CPython) | 9.084 s | 313.02× |

### sieve (N = 100,000,000)

| Language | Time | vs Nerve |
|---|---|---|
| Nerve (Synapse, final build, 10 threads) | 0.227 s | 1.00× |
| Go | 0.353 s | 1.56× |
| Rust (rustc -O) | 0.370 s | 1.63× |
| Nerve (Synapse, final build, 1 thread) | 0.373 s | 1.65× |
| C (clang -O2) | 0.376 s | 1.66× |
| Python (CPython) | 9.100 s | 40.13× |

## 2026-10-06: CyborgPL against C and Go

Run at 20:36 (+07) with `CYBORG_BENCH_THREADS=10 python3 run.py cyborgpl cyborgpl1 c go`, after the day's cores revision (CCA at `c5179dbd`), with nothing else running (owner: "Bench 1v2, uncapped").

**The nuance** (owner: "note the nuance"):

- **The parallel rows are not like for like.** CyborgPL spreads a proven `for` loop over the cores by itself (`design/parallelism.md`); the C and Go programs are the plain single-threaded code anyone would write first, with no threads added by hand. So the 10-thread row shows what CyborgPL gives for free, not what C or Go could do with work. **The 1-thread row is the fair comparison**: on it CyborgPL ties or beats C on all four (sum 0.091 s against 0.094, fib and lists equal, sieve 0.379 against 0.384, though the capped run below had it behind on sieve, 0.431 against 0.407, so that one is within run-to-run noise), and beats Go on three; on sieve Go's single thread is faster (0.352 s against 0.379).
- **fib cannot be split**: it is all function calls, so every row is one core, and CyborgPL ties C.
- **sieve is held back by memory, not cores**: going from 3 threads to 10 took it only from 0.257 s to 0.231 s, while sum halved (0.043 s to 0.022 s).
- **Startup is included, but it is small**: all four are compiled ahead of time. Measured the same way with N = 1 (best of 30, launching the process included): C 1.35 ms, CyborgPL 1.52 ms on 1 thread and 1.48 ms on 10, Go 1.94 ms (its runtime starts up first). That is a fixed ~1.5–2 ms in every row, about a tenth of the 16–22 ms runs (fib, sum) and almost nothing in sieve.

### sum (N = 300,000,000)

| Language | Time | vs CyborgPL |
|---|---|---|
| CyborgPL (CCA, final build, 10 threads) | 0.022 s | 1.00× |
| CyborgPL (CCA, final build, 1 thread) | 0.091 s | 4.14× |
| C (clang -O2) | 0.094 s | 4.27× |
| Go | 0.141 s | 6.41× |

### fib (N = 35)

| Language | Time | vs CyborgPL |
|---|---|---|
| C (clang -O2) | 0.016 s | 0.91× |
| CyborgPL (CCA, final build, 1 thread) | 0.016 s | 0.91× |
| CyborgPL (CCA, final build, 10 threads) | 0.017 s | 1.00× |
| Go | 0.022 s | 1.25× |

### lists (N = 20,000,000)

| Language | Time | vs CyborgPL |
|---|---|---|
| CyborgPL (CCA, final build, 10 threads) | 0.029 s | 1.00× |
| C (clang -O2) | 0.038 s | 1.29× |
| CyborgPL (CCA, final build, 1 thread) | 0.038 s | 1.30× |
| Go | 0.096 s | 3.30× |

### sieve (N = 100,000,000)

| Language | Time | vs CyborgPL |
|---|---|---|
| CyborgPL (CCA, final build, 10 threads) | 0.231 s | 1.00× |
| Go | 0.352 s | 1.52× |
| CyborgPL (CCA, final build, 1 thread) | 0.379 s | 1.64× |
| C (clang -O2) | 0.384 s | 1.66× |

## 2026-10-06: the same, capped at 3 threads

Run at about 20:00 (+07) with `python3 run.py cyborgpl cyborgpl1 c go`, this session's share of the cores (3) while other sessions were working.

### sum (N = 300,000,000)

| Language | Time | vs CyborgPL |
|---|---|---|
| CyborgPL (CCA, final build, 3 threads) | 0.043 s | 1.00× |
| CyborgPL (CCA, final build, 1 thread) | 0.093 s | 2.17× |
| C (clang -O2) | 0.101 s | 2.35× |
| Go | 0.144 s | 3.37× |

### fib (N = 35)

| Language | Time | vs CyborgPL |
|---|---|---|
| CyborgPL (CCA, final build, 1 thread) | 0.017 s | 0.95× |
| C (clang -O2) | 0.017 s | 0.98× |
| CyborgPL (CCA, final build, 3 threads) | 0.018 s | 1.00× |
| Go | 0.025 s | 1.41× |

### lists (N = 20,000,000)

| Language | Time | vs CyborgPL |
|---|---|---|
| CyborgPL (CCA, final build, 3 threads) | 0.037 s | 1.00× |
| CyborgPL (CCA, final build, 1 thread) | 0.038 s | 1.03× |
| C (clang -O2) | 0.041 s | 1.11× |
| Go | 0.122 s | 3.31× |

### sieve (N = 100,000,000)

| Language | Time | vs CyborgPL |
|---|---|---|
| CyborgPL (CCA, final build, 3 threads) | 0.257 s | 1.00× |
| Go | 0.394 s | 1.53× |
| C (clang -O2) | 0.407 s | 1.58× |
| CyborgPL (CCA, final build, 1 thread) | 0.431 s | 1.68× |

## 2026-09-30: every language installed (before the cores revision)

The CyborgPL measured here predates the 2026-10-03 operator words and the 2026-10-06 cores revision; its programs no longer parse as written then (`bench/` was updated to today's spellings on 2026-10-06), so these rows are history. 3 threads and 1.

### sum (N = 300,000,000)

| Language | Time | vs CyborgPL |
|---|---|---|
| CyborgPL (CCA, final build, 3 threads) | 0.040 s | 1.00× |
| C (clang -O2) | 0.095 s | 2.34× |
| C++ (clang++ -O2) | 0.095 s | 2.35× |
| CyborgPL (CCA, final build, 1 thread) | 0.098 s | 2.44× |
| Fortran (gfortran -O2) | 0.133 s | 3.30× |
| Go | 0.142 s | 3.52× |
| Swift (-O) | 0.143 s | 3.54× |
| Zig (ReleaseFast) | 0.147 s | 3.64× |
| Lust | 0.151 s | 3.77× |
| Assembly (AArch64, hand-written) | 0.153 s | 3.82× |
| Dart (AOT) | 0.156 s | 3.86× |
| C# (.NET) | 0.159 s | 3.93× |
| Java | 0.168 s | 4.17× |
| Rust (rustc -O) | 0.169 s | 4.19× |
| Kotlin (JVM) | 0.175 s | 4.32× |
| Julia | 0.187 s | 4.62× |
| OCaml (ocamlopt) | 0.211 s | 5.22× |
| Nim (release) | 0.224 s | 5.54× |
| PyPy | 0.258 s | 6.38× |
| TypeScript (tsc + Node) | 0.360 s | 8.91× |
| JavaScript (Node) | 0.364 s | 9.02× |
| LuaJIT | 0.628 s | 15.54× |
| Erlang (escript) | 0.828 s | 20.51× |
| Elixir | 0.861 s | 21.32× |
| Lua | 1.508 s | 37.33× |
| Perl | 6.287 s | 155.64× |
| Ruby | 6.632 s | 164.19× |
| Python (CPython) | 16.650 s | 412.23× |
| R | 27.499 s | 680.81× |

### fib (N = 35)

| Language | Time | vs CyborgPL |
|---|---|---|
| Fortran (gfortran -O2) | 0.004 s | 0.23× |
| Rust (rustc -O) | 0.016 s | 0.90× |
| Zig (ReleaseFast) | 0.016 s | 0.92× |
| C++ (clang++ -O2) | 0.016 s | 0.93× |
| C (clang -O2) | 0.017 s | 0.97× |
| CyborgPL (CCA, final build, 3 threads) | 0.018 s | 1.00× |
| CyborgPL (CCA, final build, 1 thread) | 0.018 s | 1.03× |
| OCaml (ocamlopt) | 0.022 s | 1.27× |
| Go | 0.023 s | 1.31× |
| Swift (-O) | 0.025 s | 1.41× |
| Assembly (AArch64, hand-written) | 0.025 s | 1.39× |
| LuaJIT | 0.033 s | 1.89× |
| C# (.NET) | 0.038 s | 2.14× |
| Nim (release) | 0.041 s | 2.36× |
| Java | 0.044 s | 2.49× |
| Dart (AOT) | 0.047 s | 2.66× |
| Kotlin (JVM) | 0.051 s | 2.92× |
| TypeScript (tsc + Node) | 0.060 s | 3.44× |
| JavaScript (Node) | 0.064 s | 3.64× |
| PyPy | 0.065 s | 3.73× |
| Ruby | 0.098 s | 5.58× |
| Julia | 0.101 s | 5.75× |
| Lust | 0.118 s | 6.56× |
| Lua | 0.211 s | 12.08× |
| Erlang (escript) | 0.216 s | 12.36× |
| Elixir | 0.230 s | 13.15× |
| Python (CPython) | 0.505 s | 28.86× |
| Perl | 2.051 s | 117.14× |
| R | 3.095 s | 176.80× |

### lists (N = 20,000,000)

| Language | Time | vs CyborgPL |
|---|---|---|
| CyborgPL (CCA, final build, 3 threads) | 0.038 s | 1.00× |
| C (clang -O2) | 0.040 s | 1.04× |
| CyborgPL (CCA, final build, 1 thread) | 0.042 s | 1.10× |
| Rust (rustc -O) | 0.053 s | 1.37× |
| C++ (clang++ -O2) | 0.064 s | 1.67× |
| Zig (ReleaseFast) | 0.070 s | 1.83× |
| Assembly (AArch64, hand-written) | 0.077 s | 2.03× |
| Fortran (gfortran -O2) | 0.092 s | 2.40× |
| Swift (-O) | 0.107 s | 2.80× |
| Go | 0.108 s | 2.81× |
| C# (.NET) | 0.123 s | 3.19× |
| LuaJIT | 0.186 s | 4.83× |
| TypeScript (tsc + Node) | 0.236 s | 6.16× |
| JavaScript (Node) | 0.239 s | 6.22× |
| Julia | 0.240 s | 6.26× |
| Dart (AOT) | 0.241 s | 6.27× |
| Nim (release) | 0.244 s | 6.37× |
| OCaml (ocamlopt) | 0.265 s | 6.91× |
| PyPy | 0.288 s | 7.51× |
| Java | 0.361 s | 9.40× |
| Kotlin (JVM) | 0.368 s | 9.59× |
| Lust | 0.620 s | 16.32× |
| Lua | 0.626 s | 16.32× |
| Perl | 3.310 s | 86.25× |
| R | 3.568 s | 92.97× |
| Elixir | 4.105 s | 106.97× |
| Ruby | 4.882 s | 127.23× |
| Erlang (escript) | 6.259 s | 163.10× |
| Python (CPython) | 9.075 s | 236.51× |

### sieve (N = 100,000,000)

| Language | Time | vs CyborgPL |
|---|---|---|
| Zig (ReleaseFast) | 0.365 s | 0.74× |
| C (clang -O2) | 0.382 s | 0.78× |
| C++ (clang++ -O2) | 0.387 s | 0.79× |
| Go | 0.394 s | 0.80× |
| Dart (AOT) | 0.396 s | 0.80× |
| C# (.NET) | 0.396 s | 0.80× |
| Assembly (AArch64, hand-written) | 0.399 s | 0.81× |
| Rust (rustc -O) | 0.404 s | 0.82× |
| Nim (release) | 0.405 s | 0.82× |
| Java | 0.410 s | 0.83× |
| Kotlin (JVM) | 0.410 s | 0.83× |
| OCaml (ocamlopt) | 0.437 s | 0.89× |
| Julia | 0.443 s | 0.90× |
| TypeScript (tsc + Node) | 0.455 s | 0.92× |
| JavaScript (Node) | 0.460 s | 0.93× |
| Swift (-O) | 0.480 s | 0.98× |
| CyborgPL (CCA, final build, 3 threads) | 0.492 s | 1.00× |
| Fortran (gfortran -O2) | 0.548 s | 1.11× |
| CyborgPL (CCA, final build, 1 thread) | 0.569 s | 1.16× |
| PyPy | 0.660 s | 1.34× |
| LuaJIT | 0.860 s | 1.75× |
| Lust | 2.867 s | 5.83× |
| Lua | 3.203 s | 6.51× |
| Erlang (escript) | 4.990 s | 10.14× |
| Ruby | 7.750 s | 15.74× |
| Python (CPython) | 9.288 s | 18.87× |
| R | 9.320 s | 18.93× |
| Elixir | 11.237 s | 22.83× |
| Perl | 19.944 s | 40.51× |

The sieve's `for` ran in 0.26 s on 3 threads with CCA just before f497869, which sped up
`lists` by handing slices a copy of a shared list's header; with it, 0.44 s. Noted for
compiler claude when CyborgPL resumes (paused 2026-09-30 until RRx2-TV exists).

The Assembly rows were measured later, on 2026-10-01, with C alongside (C: sum 0.102 s, fib
0.017 s, lists 0.044 s, sieve 0.436 s), while another session used most of the machine, so
they are noisier than the rest. The hand-written loops use a plain hardware divide for
`mod 7` and `mod 1000`, where clang turns it into a faster multiply (`umulh`, checked in its output for `sum`) and handles four numbers per trip.
