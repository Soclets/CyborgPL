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
