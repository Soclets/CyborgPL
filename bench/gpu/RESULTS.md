# Clang C against Metal — results

Apple M5 (4 performance and 6 efficiency cores, 10 GPU cores), 2026-09-30.
`python3 gpu/run.py`. Every answer matched C on one thread.

| Test | C, 1 thread | C, 3 threads | Metal, whole program | Metal, GPU part |
|---|---|---|---|---|
| sum (N=300000000) | 93.3 ms | 33.1 ms | 171.0 ms | 143.74 ms |
| map (N=100000000) | 50.5 ms | 48.0 ms | 80.5 ms | 6.18 ms |
| mandel (N=4096) | 1228.5 ms | 421.4 ms | 30.5 ms | 4.91 ms |

- **mandel**: much math for each point and little data, the GPU's kind of
  work. The whole Metal program is 40.3× faster than C on one thread and
  13.8× on three; its GPU part alone is about 250×.
- **sum**: the GPU loses. The counter is 64-bit, and Apple GPUs have no
  native 64-bit integer arithmetic, so `i × i mod 7` is emulated.
- **map**: the GPU part takes 6 ms, but starting Metal and filling the list
  on the CPU take about 74 ms of the whole, and C has to fill the list too.
  Too little work per item to pay for the start.
- **Exact math costs little**: Metal's default fast float math gave
  mandel 2837779 against C's 2837759. With `-fmetal-math-mode=safe`, the
  answer is C's and the GPU part is 5.4 ms rather than 5.1 (measured once, by
  hand). run.py builds with safe math.

For Precog's offload threshold (`design/gpu.md`, open): the work per item
and the integer width matter as much as the number of trips.
