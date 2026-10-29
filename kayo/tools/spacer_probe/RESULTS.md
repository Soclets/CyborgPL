# Z3's Spacer, measured by hand (owner, 2026-10-25: "yes" to measuring before deciding)

A first measurement, not the encoder: three of night_trains' loops written as Constrained Horn Clauses by hand, as
an encoder would write them, then Spacer (`z3 fp.engine=spacer`, `rlimit=50000000`, the step budget layer 3 uses
in kind) asked for an invariant, and each invariant it gives taken as a guess and checked in 64-bit bitvectors by
Z3 and CVC5 (`confirm_*.smt2`). `sh run.sh` runs them all. Z3 5.1.0, CVC5 as installed, this Mac.

## night_trains' sites checked at run time (174 sites, 154 proven, 20 checked; kayo 9cf75fc5)

| sites | where | what an invariant would need | probed |
|---|---|---|---|
| 1 | quickest_from, `$net.stations[pos: $pick_at]` | `pick_at = 0 ∧ pick_minutes = -1`, or `1 ≤ pick_at ≤ n` | pick_*.smt2 |
| 6 | soonest_first, `$out[pos: $i]`, `$out[pos: $i - 1]` | `1 ≤ i ≤ length` (the swaps keep the length) | sort_*.smt2 |
| 1 | std's sits_at, `$text[pos: $k]` | `pos ≤ k`, `pos + m - 1 ≤ n` (layer 3 ran out of steps) | sits_*.smt2 |
| 1 | journey, a position | not probed; likely the same shape | — |
| 11 | sums into `i32` (total_of, quickest_from's minutes and km, next, book, show_booking, board, reached_by) | none: each adds values read from the timetable or a count of them, and no bound in the program stops the sum passing `i32`'s top | — |

So about 9 of the 20 are the kind an invariant proves; the other 11 can overflow on some input, and no invariant
finder proves them (they stay checked at run time, which is right).

## Spacer

| loop | integers | 64-bit bitvectors |
|---|---|---|
| pick (quickest_from) | invariant, 0.03 s | unknown at the step budget, 9.1 s |
| sits_at | invariant, 0.02 s | unknown at the step budget, 8.6 s |
| sort (soonest_first) | invariant, 0.02 s | invariant (`i ≤ len`), 0.07 s |

The integer invariants are not proofs by themselves: integers do not wrap. Read in 64-bit bitvectors, both fail at
the very top of the range (`i = n = 2^63 - 1`, `k = 2^64 - 1`), and both hold, by Z3 and by CVC5 (unsat), once
the loops' own `+ 1` and `$pos + $m - 1` are assumed to have passed their overflow checks, as layer 3 already
assumes for checks walked before a site. So the shape that works here is: encode in integers, ask Spacer, then
confirm the invariant in bitvectors with the program's own earlier checks assumed passed, by both solvers.

## Not measured yet

Build time with Spacer in kayo, and the count across the examples and the suites: that needs the encoder (a
loop's state, entry, trip and site as Horn clauses, from the flow graph beside layer 3's bitvector encoder), and
the confirmation step. The three probes cost under 0.1 s each in integers and 9 s each when Spacer fails in
bitvectors, so integers first.

## Measured in kayo (owner's go, 2026-10-29)

Spacer is now built into Precog, as the 2026-10-25 decision shapes it:
1. **Shapes first.** Spacer runs only for a site still unproven after the shapes and the whole layer 3 ladder, and only
   for the loops on some way to it.
2. **A step budget, never a clock.** Z3's `rlimit`, `SPACER_RLIMIT` = 10,000,000 per loop question. `KAYO_SPACER_RLIMIT`
   sets another one for measuring.
3. **Confirmed by both solvers.** Every invariant it gives is confirmed by Z3 *and* CVC5 before it is used: it holds on
   every way into the loop, and one trip round keeps it. The site is then asked again with the invariant assumed at
   the loop's head.
4. **On by default.** `KAYO_NO_SPACER=1` turns it off, and `KAYO_SPACER_LOG=1` prints each loop question and its
   outcome.

The clauses are written in bit-vectors, as Precog's questions are. They come from three walks of the flow graph, with
every head standing for every trip:
- from the function's entry to the ways into the loop;
- once round the loop from its head;
- from its head to the site.

`measure.py` made the figures; `MEASURED.md` has every program, every question and every invariant.

- **Screened:** every example (221) and every kayo test program (241), each built once with Spacer on. 7 examples did
  not build, all of them listed in `examples_unbuilt.txt`. 451 programs never asked Spacer anything: no site in them
  is unproven behind a loop, so Spacer costs them nothing.
- **Asked:** 22 programs, with 52 loop questions over 40 unproven sites.
  - 1 confirmed invariant proved its site: `vacuous_after_exact_quotient`, line 28, with `$k ≤ len($ds)`.
  - 41 refuted: Spacer found a way through its clauses to the bad value. The ones read by hand are sites that really
    can fail, which kayo's test programs keep on purpose: sums of values read at run time, `0 - $n` into a `u64`,
    `$argc × 100000` into an `i16`.
  - 6 spent the budget. Five are on sites that can fail (gradebook's and `par_lent_param`'s sums; `par_trap`'s
    `to.i8` of numbers up to 200,000). One looks provable: in `msl_whole_pages`, `$wrong + 1` never passes the
    16,384 elements.
  - 4 were not asked: a trip round the loop could not be put into words.
  - No invariant failed confirmation.
- **Cost:** over those 22 programs, the sum of the medians of three builds:
  - Precog's phase: 27.60 s off, 38.87 s on;
  - whole builds: 29.20 s off, 40.46 s on.
  - The largest single cost was `par_lent_param`'s Precog phase: 3.62 s off, 9.11 s on.
- **A budget 10× larger** (100,000,000) answered none of the 6, and made those builds 4 to 15 times slower than at
  the default (`par_lent_param`: 53 s).
- **The same every time:** in every program the three builds gave the same sites and the same log, off and on.
- **night_trains** proves all 173 of its sites with Spacer off (the Precog work of 2026-10-26), so Spacer is never
  asked there. The "12 of 60 unproven" of 2026-10-25 is gone.

What this suggests, for the owner to decide: in bit-vectors, Spacer finds little that the shapes miss in these
programs, because nearly everything left unproven can really fail. Where it runs out, it runs out in bit-vectors, as
the hand probe above showed for 64-bit loops. Even the lock-step test program's `$j + 1` (two counters moving
together) spent the budget. The other way, clauses in integers with the guess confirmed in bit-vectors, is not built
or measured.

## Integer clauses, confirmed in bit-vectors (owner, 2026-10-29: "Test out number 2 first")

`measure_modes.py` made these figures; `MODES.md` has every program, question and invariant. The programs are the 22
above plus `two_counters.cy` (kept here). Each was built three times per mode, one build at a time; times are medians.
The budget is `rlimit` 10,000,000 in every mode.

The modes:
- **bits**: today's default, unchanged. All 22 programs gave the first measurement's sites and logs exactly.
- **integers** (`KAYO_SPACER_INTEGERS=1`): one loop at a time, as bits does, but over the integers. Each value is read
  as a number, signed or unsigned as the encoder made it, with its type's bounds stated as facts on the values the
  clauses know nothing about. Bit operations and shifts are unknowns. Spacer's invariant is read back into 192-bit
  arithmetic and confirmed by Z3 and CVC5 as before.
- **joint** (`+ KAYO_SPACER_JOINT=1`): every loop on the way to the site as one system, each loop head a cut point
  with its own invariant. In these walks only, a list written out (`[]`) has its length known.
- **+gg** (`KAYO_SPACER_GLOBAL=1`): Spacer's global guidance (`fp.spacer.global`).

| mode | unproven (23 programs) | Precog, s | whole builds, s | questions | spent the budget |
|---|---:|---:|---:|---:|---:|
| off | 42 | 28.36 | 30.02 | 0 | 0 |
| bits | 40 | 40.45 | 42.08 | 54 | 7 |
| integers | 40 | 34.63 | 36.30 | 54 | 6 |
| integers+gg | 39 | 38.04 | 39.71 | 54 | 5 |
| joint | 40 | 36.57 | 38.22 | 36 | 6 |
| joint+gg | 38 | 153.38 | 155.04 | 38 | 2 (+2 that crashed Z3) |
| bits+gg | 40 | 40.61 | 42.25 | 54 | 7 |

- **Beyond bits**, only joint+gg proves more in the 22 programs: msl_whole_pages' `$wrong + 1`, with all three loops'
  invariants confirmed. It and integers+gg also prove `two_counters`' `$j + 1`.
- **Integers alone**, one loop at a time, prove the same sites as bits, in less time (par_lent_param: 3.61 s against
  9.29 s).
- **No integer invariant failed** the bit-vector confirmation.
- **Global guidance crashes Z3 5.1.0 here.** On gradebook's sum, Spacer's global generalizer reaches an internal
  assertion (`spacer_global_generalizer.cpp:337`, "UNEXPECTED CODE WAS REACHED") after 70 s or more of running past
  its `rlimit`. kayo reads that as "Spacer did not say", so nothing is proven by it, but the build took 109 to 119 s
  instead of 3. That is where joint+gg's 153 s total comes from; without gradebook it is 36.0 s.
- **The same every time:** in every mode, every program's three builds gave the same sites and the same log.
