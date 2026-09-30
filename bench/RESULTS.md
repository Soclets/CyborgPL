# Benchmark results

Machine: arm64, Darwin 27.0.0; run 2026-09-30 by `bench/run.py`. Best of 3 runs, wall-clock from process start to exit (startup included); runs past 60 s are timeouts. Every answer is checked against C's.

## sum (N = 300,000,000)

| Language | Time | vs CyborgPL |
|---|---|---|
| Fortran (gfortran -O2) | 0.629 s | 0.93× |
| CyborgPL (CCA, final build) | 0.673 s | 1.00× |
| Go | 0.745 s | 1.11× |
| Java | 0.790 s | 1.17× |
| C# (.NET) | 0.792 s | 1.18× |
| Swift (-O) | 0.799 s | 1.19× |
| Kotlin (JVM) | 0.800 s | 1.19× |
| C++ (clang++ -O2) | 0.807 s | 1.20× |
| Lust | 0.808 s | 1.20× |
| C (clang -O2) | 0.808 s | 1.20× |
| Rust (rustc -O) | 0.809 s | 1.20× |
| Dart (AOT) | 0.845 s | 1.26× |
| Nim (release) | 0.862 s | 1.28× |
| PyPy | 0.863 s | 1.28× |
| Julia | 0.904 s | 1.34× |
| TypeScript (tsc + Node) | 0.935 s | 1.39× |
| JavaScript (Node) | 0.935 s | 1.39× |
| OCaml (ocamlopt) | 0.991 s | 1.47× |
| Zig (ReleaseFast) | 1.088 s | 1.62× |
| Elixir | 1.294 s | 1.92× |
| LuaJIT | 1.350 s | 2.00× |
| Erlang (escript) | 1.450 s | 2.15× |
| Lua | 2.366 s | 3.51× |
| Ruby | 7.737 s | 11.49× |
| Perl | 8.070 s | 11.98× |
| Python (CPython) | 18.939 s | 28.13× |
| R | 37.225 s | 55.28× |

## fib (N = 35)

| Language | Time | vs CyborgPL |
|---|---|---|
| Fortran (gfortran -O2) | 0.003 s | 0.19× |
| Rust (rustc -O) | 0.015 s | 0.90× |
| C++ (clang++ -O2) | 0.016 s | 0.92× |
| Zig (ReleaseFast) | 0.017 s | 0.98× |
| CyborgPL (CCA, final build) | 0.017 s | 1.00× |
| C (clang -O2) | 0.018 s | 1.05× |
| Go | 0.022 s | 1.30× |
| OCaml (ocamlopt) | 0.022 s | 1.32× |
| Swift (-O) | 0.026 s | 1.53× |
| LuaJIT | 0.033 s | 1.94× |
| C# (.NET) | 0.037 s | 2.18× |
| Nim (release) | 0.040 s | 2.35× |
| Java | 0.042 s | 2.48× |
| Dart (AOT) | 0.047 s | 2.79× |
| Kotlin (JVM) | 0.048 s | 2.85× |
| TypeScript (tsc + Node) | 0.061 s | 3.57× |
| JavaScript (Node) | 0.062 s | 3.66× |
| PyPy | 0.065 s | 3.83× |
| Ruby | 0.098 s | 5.78× |
| Julia | 0.098 s | 5.79× |
| Lust | 0.118 s | 6.97× |
| Erlang (escript) | 0.212 s | 12.50× |
| Lua | 0.214 s | 12.60× |
| Elixir | 0.223 s | 13.12× |
| Python (CPython) | 0.535 s | 31.51× |
| Perl | 2.033 s | 119.75× |
| R | 3.127 s | 184.17× |

## lists (N = 20,000,000)

| Language | Time | vs CyborgPL |
|---|---|---|
| CyborgPL (CCA, final build) | 0.041 s | 1.00× |
| C (clang -O2) | 0.041 s | 1.01× |
| Rust (rustc -O) | 0.050 s | 1.22× |
| C++ (clang++ -O2) | 0.066 s | 1.64× |
| Zig (ReleaseFast) | 0.067 s | 1.66× |
| Fortran (gfortran -O2) | 0.095 s | 2.34× |
| Go | 0.105 s | 2.59× |
| Swift (-O) | 0.115 s | 2.84× |
| C# (.NET) | 0.126 s | 3.12× |
| LuaJIT | 0.181 s | 4.46× |
| JavaScript (Node) | 0.235 s | 5.81× |
| TypeScript (tsc + Node) | 0.236 s | 5.81× |
| Julia | 0.239 s | 5.89× |
| Nim (release) | 0.245 s | 6.05× |
| Dart (AOT) | 0.252 s | 6.22× |
| OCaml (ocamlopt) | 0.266 s | 6.57× |
| PyPy | 0.286 s | 7.07× |
| Java | 0.343 s | 8.46× |
| Kotlin (JVM) | 0.371 s | 9.15× |
| Lua | 0.610 s | 15.05× |
| Lust | 0.621 s | 15.34× |
| Perl | 3.378 s | 83.38× |
| R | 3.591 s | 88.65× |
| Elixir | 3.946 s | 97.43× |
| Ruby | 4.931 s | 121.75× |
| Erlang (escript) | 6.115 s | 150.96× |
| Python (CPython) | 8.960 s | 221.20× |

## sieve (N = 100,000,000)

| Language | Time | vs CyborgPL |
|---|---|---|
| Go | 0.372 s | 0.95× |
| Zig (ReleaseFast) | 0.374 s | 0.96× |
| C# (.NET) | 0.375 s | 0.96× |
| Rust (rustc -O) | 0.380 s | 0.97× |
| C (clang -O2) | 0.390 s | 1.00× |
| CyborgPL (CCA, final build) | 0.391 s | 1.00× |
| Nim (release) | 0.394 s | 1.01× |
| Kotlin (JVM) | 0.397 s | 1.02× |
| C++ (clang++ -O2) | 0.400 s | 1.02× |
| Java | 0.403 s | 1.03× |
| Dart (AOT) | 0.421 s | 1.08× |
| OCaml (ocamlopt) | 0.426 s | 1.09× |
| Julia | 0.434 s | 1.11× |
| JavaScript (Node) | 0.446 s | 1.14× |
| TypeScript (tsc + Node) | 0.446 s | 1.14× |
| Swift (-O) | 0.492 s | 1.26× |
| Fortran (gfortran -O2) | 0.542 s | 1.39× |
| PyPy | 0.643 s | 1.65× |
| LuaJIT | 0.865 s | 2.21× |
| Lust | 2.866 s | 7.34× |
| Lua | 3.234 s | 8.28× |
| Erlang (escript) | 4.968 s | 12.72× |
| Ruby | 7.781 s | 19.92× |
| R | 9.450 s | 24.20× |
| Python (CPython) | 9.504 s | 24.34× |
| Elixir | 11.053 s | 28.30× |
| Perl | 19.788 s | 50.67× |
