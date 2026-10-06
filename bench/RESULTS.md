# Benchmark results

Machine: arm64 (Apple Silicon, 10 cores: 4 performance, 6 efficiency), Darwin 27.0.0. Best of 3 runs, wall-clock from process start to exit (startup included); runs past 60 s are timeouts. Every answer is checked against C's, and every one agreed.

## 2026-10-06: CyborgPL against C and Go

Run at 20:36 (+07) with `CYBORG_BENCH_THREADS=10 python3 run.py cyborgpl cyborgpl1 c go`, after the day's cores revision (CCA at `c5179dbd`), with nothing else running (owner: "Bench 1v2, uncapped").

**The nuance** (owner: "note the nuance"):

- **The parallel rows are not like for like.** CyborgPL spreads a proven `for` loop over the cores by itself (`design/parallelism.md`); the C and Go programs are the plain single-threaded code anyone would write first, with no threads added by hand. So the 10-thread row shows what CyborgPL gives for free, not what C or Go could do with work. **The 1-thread row is the fair comparison**: on it CyborgPL ties or beats C on all four (sum 0.091 s against 0.094, fib and lists equal, sieve 0.379 against 0.384, though the capped run below had it behind on sieve, 0.431 against 0.407, so that one is within run-to-run noise), and beats Go on three; on sieve Go's single thread is faster (0.352 s against 0.379).
- **fib cannot be split**: it is all function calls, so every row is one core, and CyborgPL ties C.
- **sieve is held back by memory, not cores**: going from 3 threads to 10 took it only from 0.257 s to 0.231 s, while sum halved (0.043 s to 0.022 s).
- **Startup is included**, so these short runs (tens of milliseconds) also measure how fast each program starts.

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
