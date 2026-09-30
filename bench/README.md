# CyborgPL against every other language

The same four programs in CyborgPL and in every other language installed on the
machine that runs them — 26 so far, including Lust. The owner's own languages are
left out; languages the owner only contributed to are in. Results: [`RESULTS.md`](RESULTS.md).

| Program | What it does | N |
|---|---|---|
| `sum` | Σ (i × i mod 7) for i = 1..N, kept below 1 000 000 007 | 300 000 000 |
| `fib` | naive recursive Fibonacci | 35 |
| `lists` | append N numbers to a growable list, then sum it 10 times | 20 000 000 |
| `sieve` | count the primes up to N (sieve of Eratosthenes) | 100 000 000 |

## The rules every version follows

- **N comes from the environment** (`N=…`), so no compiler can work the answer out
  in advance, and **the answer's low byte is the exit code**. The runner checks
  every language's answer against C's; a wrong answer is reported, never timed.
- **Each language writes it the plain, idiomatic way** — its own growable list,
  its own loop — without hand-tuning.
- **Time is wall-clock from process start to exit**, so a language's startup (a
  JVM, Julia's compiler, `dotnet`) counts. Best of 3 runs; a freshly built program
  gets one untimed run first, since macOS scans new programs on their first run.
  A run past 60 seconds is a timeout.

## Where a version differs, and why

- **JavaScript, TypeScript, Lua, LuaJIT and R** hold numbers as doubles, and
  `i × i` passes 2⁵³ during `sum`, so they square `i mod 7` instead — the same
  answer with less work.
- **R** has no growable vector, so `lists` fills a preallocated one.
- **Lust** cannot read the environment or set an exit code, so the runner writes N
  into its source and reads the answer it prints.
- **Elixir and Erlang** have no mutable array: `lists` uses `:array` and `sieve`
  uses `:atomics`.
- **Nim** exits through C's `exit`, since Nim's `quit` caps exit codes at 127.
- **C#** runs as `dotnet sum.dll`: Homebrew's .NET is not where the apphost looks.
- **CyborgPL** is `cca build --final`, the build with every check proven.

## Running it

```
python3 run.py                     # every installed language
python3 run.py cyborgpl c rust     # only these
python3 run.py --quick             # tiny sizes: does everything build and agree?
CCA=/path/to/cca LUST=/path/to/lust python3 run.py
```

It needs CCA built (`cca/target/release/cca`, or `CCA=`), and runs one program at
a time, on one core.

## GPU: Clang C against Metal

`gpu/` is separate: three loops in C (on 1 and 3 threads) and as Metal
kernels, to see what kind of work is worth a GPU. It is plain C and Metal,
not CyborgPL, since CCA has no GPU target yet. It needs Xcode's Metal
Toolchain. `python3 gpu/run.py` (`--quick` for tiny sizes); results in
[`gpu/RESULTS.md`](gpu/RESULTS.md).
