# Point 3: copy against no-copy, measured

The owner, 2026-10-25: "How about we let it measure?" (design claude's brief: lists of 1 KB to 64 MB, a
trivial kernel, copy and no-copy for `lend` and `lend.mut`, what page alignment costs, the crossover).
kayo still copies; the owner chooses from these.

**How.** `cargo run --release -j3 --offline -p kayo-metal-bench`, one core, on this Mac (Apple M5, 16 KB pages,
macOS 27). Each run is a whole block run as `cyborg_msl_run` does it — device, library, pipeline, queue,
encode, commit, wait, copy back — with a one-thread kernel adding 1 to one byte (`kernel.metal`), so the
transfer shows rather than the work. Copy: the list from `malloc`, `newBufferWithBytes` into a shared
buffer, and for `lend.mut` copied back. No-copy: the list page-aligned (`posix_memalign`, its length
rounded up to a page), wrapped by `newBufferWithBytesNoCopy`. 101 runs a size after 3 warm-ups, the four
ways interleaved in each round; each run asserts the kernel's write reached the list exactly when it
should. Whole-run medians jump between about 350 and 1000 µs with the GPU's power state, so they are
given as median / best; the buffer part (making the buffer, and the copy back) is steady, its median.

**Read.** No-copy's buffer part wins from 16 KB, one page (about half of copying's); below it copying is
about 0.7 µs cheaper. Under about 1 MB either is lost in the block's fixed setup (about 300 µs and up);
at 1 MB no-copy saves 60–80 µs, at 64 MB 3.5 ms (`lend`) and 6.8 ms (`lend.mut`) a block. Allocating
aligned costs nothing; growing aligned copies, up to 10× realloc's time at 64 MB — but `malloc` here
already returned page-aligned blocks for every size of 16 KB and up (its behaviour, not a promise).
A list padded to a page wastes up to 94% of it at 1 KB. The fixed setup costs more than either way of
transfer below about 1 MB: a block's pipeline kept from one run to the next would save more there.

## The run sent to design claude (2026-10-25)

## Block runs (101 runs each after 3 warm-up, the four ways interleaved, median / best, microseconds)
| size | copy, lend | no-copy, lend | copy, lend.mut | no-copy, lend.mut | buffer part: copy lend / no-copy lend / copy lend.mut / no-copy lend.mut |
|---|---|---|---|---|---|
| 1 KB | 1012 / 341 | 419 / 340 | 397 / 347 | 1020 / 337 | 1.6 / 2.4 / 1.8 / 2.3 |
| 4 KB | 382 / 298 | 424 / 306 | 385 / 325 | 993 / 319 | 1.5 / 2.2 / 1.8 / 2.2 |
| 16 KB | 384 / 335 | 395 / 320 | 385 / 306 | 993 / 336 | 3.9 / 2.0 / 4.3 / 2.1 |
| 64 KB | 388 / 337 | 394 / 304 | 398 / 336 | 1010 / 344 | 6.2 / 2.0 / 7.4 / 2.0 |
| 1 MB | 1202 / 404 | 415 / 344 | 1123 / 419 | 1066 / 974 | 60.0 / 2.3 / 74.7 / 2.4 |
| 64 MB | 7939 / 5561 | 2019 / 1101 | 11142 / 8002 | 2124 / 1859 | 3524.7 / 7.6 / 6843.0 / 9.1 |

## Allocating and growing (31 runs, median, microseconds)
| size | malloc | page-aligned | grow by doubling from 8 B: realloc | grow: page-aligned (new, copy, free) | malloc already page-aligned? |
|---|---|---|---|---|---|
| 1 KB | 0.04 | 0.04 | 0.25 | 0.21 | 2/16 |
| 4 KB | 0.04 | 0.04 | 0.29 | 0.25 | 4/16 |
| 16 KB | 0.04 | 0.04 | 0.58 | 0.54 | 16/16 |
| 64 KB | 0.12 | 0.12 | 1.33 | 1.00 | 16/16 |
| 1 MB | 0.12 | 0.12 | 2.42 | 15.58 | 16/16 |
| 64 MB | 0.08 | 0.08 | 123.63 | 1213.50 | 16/16 |

## Memory a page-aligned list takes (page 16384 B)
| list | rounded to | wasted |
|---|---|---|
| 8 B | 16384 B | 16376 B (100%) |
| 64 B | 16384 B | 16320 B (100%) |
| 1024 B | 16384 B | 15360 B (94%) |
| 4096 B | 16384 B | 12288 B (75%) |
| 16384 B | 16384 B | 0 B (0%) |
| 16385 B | 32768 B | 16383 B (50%) |
| 1048576 B | 1048576 B | 0 B (0%) |

## Rerun from this folder (2026-10-25)

## Block runs (101 runs each after 3 warm-up, the four ways interleaved, median / best, microseconds)
| size | copy, lend | no-copy, lend | copy, lend.mut | no-copy, lend.mut | buffer part: copy lend / no-copy lend / copy lend.mut / no-copy lend.mut |
|---|---|---|---|---|---|
| 1 KB | 378 / 322 | 377 / 326 | 380 / 331 | 397 / 309 | 1.5 / 2.2 / 1.6 / 2.2 |
| 4 KB | 369 / 307 | 384 / 327 | 365 / 325 | 436 / 329 | 1.6 / 2.2 / 1.8 / 2.2 |
| 16 KB | 404 / 308 | 420 / 332 | 388 / 328 | 435 / 335 | 4.0 / 2.3 / 4.4 / 2.4 |
| 64 KB | 493 / 352 | 472 / 333 | 514 / 341 | 1024 / 346 | 7.2 / 2.3 / 8.5 / 2.5 |
| 1 MB | 1293 / 413 | 696 / 348 | 1153 / 440 | 1173 / 996 | 64.2 / 3.0 / 80.1 / 2.5 |
| 64 MB | 7666 / 5171 | 1941 / 1043 | 10313 / 7599 | 2033 / 1695 | 3607.0 / 8.3 / 6823.4 / 9.5 |

## Allocating and growing (31 runs, median, microseconds)
| size | malloc | page-aligned | grow by doubling from 8 B: realloc | grow: page-aligned (new, copy, free) | malloc already page-aligned? |
|---|---|---|---|---|---|
| 1 KB | 0.04 | 0.04 | 0.17 | 0.17 | 3/16 |
| 4 KB | 0.04 | 0.04 | 0.25 | 0.21 | 5/16 |
| 16 KB | 0.04 | 0.04 | 0.42 | 0.38 | 16/16 |
| 64 KB | 0.08 | 0.08 | 1.00 | 0.92 | 16/16 |
| 1 MB | 0.08 | 0.08 | 2.33 | 17.33 | 16/16 |
| 64 MB | 0.08 | 0.04 | 120.17 | 1275.21 | 16/16 |

## Memory a page-aligned list takes (page 16384 B)
| list | rounded to | wasted |
|---|---|---|
| 8 B | 16384 B | 16376 B (100%) |
| 64 B | 16384 B | 16320 B (100%) |
| 1024 B | 16384 B | 15360 B (94%) |
| 4096 B | 16384 B | 12288 B (75%) |
| 16384 B | 16384 B | 0 B (0%) |
| 16385 B | 32768 B | 16383 B (50%) |
| 1048576 B | 1048576 B | 0 B (0%) |
