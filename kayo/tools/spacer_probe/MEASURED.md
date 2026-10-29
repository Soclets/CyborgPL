# Spacer in Precog, measured

Made by `measure.py` (its docstring says how). Spacer's budget: `rlimit` 10_000_000 of Z3's steps per loop question. Times are the median of three builds, one at a time, read from each build's report (the Precog phase, and the whole build).

**Sites newly proven: 1**, across the programs below.

## Every program Spacer was asked about

| program | unproven off | unproven on | Precog off, s | Precog on, s | build off, s | build on, s | questions | spent the budget |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| examples/args.cy | 1 | 1 | 5.51 | 5.5 | 5.59 | 5.56 | 2 | 0 |
| examples/gradebook.cy | 2 | 2 | 1.79 | 2.78 | 1.86 | 2.85 | 2 | 2 |
| examples/list_page_aligned.cy | 1 | 1 | 0.803 | 0.92 | 0.863 | 0.981 | 1 | 0 |
| kayo/compiler/tests/programs/channels_handoff.cy | 1 | 1 | 0.153 | 0.168 | 0.21 | 0.226 | 1 | 0 |
| kayo/compiler/tests/programs/channels_whichever.cy | 5 | 5 | 0.522 | 0.721 | 0.582 | 0.781 | 3 | 0 |
| kayo/compiler/tests/programs/incl_bytes_change.cy | 1 | 1 | 0.16 | 0.176 | 0.217 | 0.232 | 1 | 0 |
| kayo/compiler/tests/programs/iterator_owned.cy | 4 | 4 | 1.31 | 1.34 | 1.37 | 1.39 | 4 | 0 |
| kayo/compiler/tests/programs/iterator_walk.cy | 5 | 5 | 2.07 | 2.14 | 2.13 | 2.2 | 6 | 0 |
| kayo/compiler/tests/programs/lend_mut_through.cy | 1 | 1 | 0.049 | 0.103 | 0.104 | 0.159 | 1 | 0 |
| kayo/compiler/tests/programs/map_get_owned_copy.cy | 1 | 1 | 0.572 | 0.659 | 0.643 | 0.731 | 3 | 0 |
| kayo/compiler/tests/programs/map_walk.cy | 2 | 2 | 2.42 | 2.48 | 2.48 | 2.54 | 3 | 0 |
| kayo/compiler/tests/programs/msl_narrowed_cross.cy | 1 | 1 | 1.17 | 1.28 | 1.3 | 1.42 | 1 | 0 |
| kayo/compiler/tests/programs/msl_page_aligned.cy | 2 | 2 | 1.86 | 2.06 | 2.03 | 2.22 | 3 | 0 |
| kayo/compiler/tests/programs/msl_whole_pages.cy | 1 | 1 | 1.05 | 2.82 | 1.18 | 2.95 | 3 | 2 |
| kayo/compiler/tests/programs/nested_loop_exit.cy | 1 | 1 | 0.455 | 0.731 | 0.513 | 0.788 | 2 | 0 |
| kayo/compiler/tests/programs/pairs_two_kinds.cy | 2 | 2 | 0.137 | 0.219 | 0.212 | 0.293 | 5 | 0 |
| kayo/compiler/tests/programs/par_lent_param.cy | 1 | 1 | 3.62 | 9.11 | 3.68 | 9.18 | 1 | 1 |
| kayo/compiler/tests/programs/par_trap.cy | 1 | 1 | 1.53 | 2.26 | 1.58 | 2.32 | 1 | 1 |
| kayo/compiler/tests/programs/past_the_inner_unrolling.cy | 1 | 1 | 0.669 | 0.806 | 0.73 | 0.868 | 2 | 0 |
| kayo/compiler/tests/programs/shl_in_a_loop.cy | 1 | 1 | 0.048 | 0.057 | 0.105 | 0.113 | 1 | 0 |
| kayo/compiler/tests/programs/two_loops_in_a_row.cy | 2 | 2 | 0.203 | 0.363 | 0.258 | 0.419 | 4 | 0 |
| kayo/compiler/tests/programs/vacuous_after_exact_quotient.cy | 3 | 2 | 1.5 | 2.18 | 1.56 | 2.24 | 2 | 0 |
| **all 22** | **40** | **39** | **27.60** | **38.87** | **29.20** | **40.46** | **52** | **6** |

## At a budget of 100,000,000 steps

Only the programs where a loop question spent the default budget.

| program | unproven | Precog, s (median of 3) | build, s | questions that spent the budget | same every run |
|---|---:|---:|---:|---:|---|
| examples/gradebook.cy | 2 | 16.36 | 16.43 | 2 | True |
| kayo/compiler/tests/programs/msl_whole_pages.cy | 1 | 14.51 | 14.64 | 2 | True |
| kayo/compiler/tests/programs/par_lent_param.cy | 1 | 53.08 | 53.15 | 1 | True |
| kayo/compiler/tests/programs/par_trap.cy | 1 | 6.95 | 7.0 | 1 | True |

## examples/args.cy

| | Spacer off | Spacer on |
|---|---:|---:|
| sites | 6 | 6 |
| unproven (checked at run time) | 1 | 1 |
| Precog, s (median of 3) | 5.51 | 5.5 |
| whole build, s (median of 3) | 5.59 | 5.56 |
| Precog, s, each run | 5.51, 5.53, 5.47 | 5.5, 5.61, 5.5 |
| the same sites and log every run | True | True |
| loop questions put to Spacer | | 2 (2 loops) |
| questions that spent the budget | | 0 (0 loops) |

<details><summary>Spacer's whole log</summary>

```
site line 18 (in MAIN), loop at line 8: found a way through its clauses to the bad value
site line 18 (in MAIN), loop at line 14: not asked: a trip round the loop could not be put into words
```
</details>

## examples/gradebook.cy

| | Spacer off | Spacer on |
|---|---:|---:|
| sites | 17 | 17 |
| unproven (checked at run time) | 2 | 2 |
| Precog, s (median of 3) | 1.79 | 2.78 |
| whole build, s (median of 3) | 1.86 | 2.85 |
| Precog, s, each run | 1.79, 1.81, 1.75 | 2.85, 2.78, 2.69 |
| the same sites and log every run | True | True |
| loop questions put to Spacer | | 2 (1 loops) |
| questions that spent the budget | | 2 (1 loops) |

<details><summary>Spacer's whole log</summary>

```
site line 15 (in average), loop at line 14: spent its step budget
site line 15 (in average), loop at line 14: spent its step budget
```
</details>

## examples/list_page_aligned.cy

| | Spacer off | Spacer on |
|---|---:|---:|
| sites | 11 | 11 |
| unproven (checked at run time) | 1 | 1 |
| Precog, s (median of 3) | 0.803 | 0.92 |
| whole build, s (median of 3) | 0.863 | 0.981 |
| Precog, s, each run | 0.8034, 0.8001, 0.8052999999999999 | 0.9202, 0.9148, 0.9258 |
| the same sites and log every run | True | True |
| loop questions put to Spacer | | 1 (1 loops) |
| questions that spent the budget | | 0 (0 loops) |

<details><summary>Spacer's whole log</summary>

```
site line 8 (in total), loop at line 7: found a way through its clauses to the bad value
```
</details>

## kayo/compiler/tests/programs/channels_handoff.cy

| | Spacer off | Spacer on |
|---|---:|---:|
| sites | 11 | 11 |
| unproven (checked at run time) | 1 | 1 |
| Precog, s (median of 3) | 0.153 | 0.168 |
| whole build, s (median of 3) | 0.21 | 0.226 |
| Precog, s, each run | 0.15280000000000002, 0.1524, 0.15319999999999998 | 0.1687, 0.1666, 0.1681 |
| the same sites and log every run | True | True |
| loop questions put to Spacer | | 1 (1 loops) |
| questions that spent the budget | | 0 (0 loops) |

<details><summary>Spacer's whole log</summary>

```
site line 17 (in MAIN), loop at line 15: found a way through its clauses to the bad value
```
</details>

## kayo/compiler/tests/programs/channels_whichever.cy

| | Spacer off | Spacer on |
|---|---:|---:|
| sites | 30 | 30 |
| unproven (checked at run time) | 5 | 5 |
| Precog, s (median of 3) | 0.522 | 0.721 |
| whole build, s (median of 3) | 0.582 | 0.781 |
| Precog, s, each run | 0.526, 0.5225, 0.519 | 0.7215, 0.7212000000000001, 0.7156 |
| the same sites and log every run | True | True |
| loop questions put to Spacer | | 3 (1 loops) |
| questions that spent the budget | | 0 (0 loops) |

<details><summary>Spacer's whole log</summary>

```
site line 26 (in MAIN), loop at line 23: found a way through its clauses to the bad value
site line 26 (in MAIN), loop at line 23: found a way through its clauses to the bad value
site line 29 (in MAIN), loop at line 23: found a way through its clauses to the bad value
```
</details>

## kayo/compiler/tests/programs/incl_bytes_change.cy

| | Spacer off | Spacer on |
|---|---:|---:|
| sites | 22 | 22 |
| unproven (checked at run time) | 1 | 1 |
| Precog, s (median of 3) | 0.16 | 0.176 |
| whole build, s (median of 3) | 0.217 | 0.232 |
| Precog, s, each run | 0.1621, 0.15969999999999998, 0.1598 | 0.1757, 0.1747, 0.1764 |
| the same sites and log every run | True | True |
| loop questions put to Spacer | | 1 (1 loops) |
| questions that spent the budget | | 0 (0 loops) |

<details><summary>Spacer's whole log</summary>

```
site line 22 (in MAIN), loop at line 20: found a way through its clauses to the bad value
```
</details>

## kayo/compiler/tests/programs/iterator_owned.cy

| | Spacer off | Spacer on |
|---|---:|---:|
| sites | 15 | 15 |
| unproven (checked at run time) | 4 | 4 |
| Precog, s (median of 3) | 1.31 | 1.34 |
| whole build, s (median of 3) | 1.37 | 1.39 |
| Precog, s, each run | 1.32, 1.3, 1.31 | 1.33, 1.35, 1.34 |
| the same sites and log every run | True | True |
| loop questions put to Spacer | | 4 (2 loops) |
| questions that spent the budget | | 0 (0 loops) |

<details><summary>Spacer's whole log</summary>

```
site line 23 (in MAIN), loop at line 22: found a way through its clauses to the bad value
site line 23 (in MAIN), loop at line 22: found a way through its clauses to the bad value
site line 27 (in MAIN), loop at line 22: found a way through its clauses to the bad value
site line 27 (in MAIN), loop at line 26: found a way through its clauses to the bad value
```
</details>

## kayo/compiler/tests/programs/iterator_walk.cy

| | Spacer off | Spacer on |
|---|---:|---:|
| sites | 20 | 20 |
| unproven (checked at run time) | 5 | 5 |
| Precog, s (median of 3) | 2.07 | 2.14 |
| whole build, s (median of 3) | 2.13 | 2.2 |
| Precog, s, each run | 2.07, 2.14, 2.07 | 2.17, 2.14, 2.1 |
| the same sites and log every run | True | True |
| loop questions put to Spacer | | 6 (3 loops) |
| questions that spent the budget | | 0 (0 loops) |

<details><summary>Spacer's whole log</summary>

```
site line 42 (in MAIN), loop at line 38: found a way through its clauses to the bad value
site line 45 (in MAIN), loop at line 38: found a way through its clauses to the bad value
site line 45 (in MAIN), loop at line 44: found a way through its clauses to the bad value
site line 53 (in MAIN), loop at line 38: found a way through its clauses to the bad value
site line 53 (in MAIN), loop at line 44: found a way through its clauses to the bad value
site line 53 (in MAIN), loop at line 48: found a way through its clauses to the bad value
```
</details>

## kayo/compiler/tests/programs/lend_mut_through.cy

| | Spacer off | Spacer on |
|---|---:|---:|
| sites | 9 | 9 |
| unproven (checked at run time) | 1 | 1 |
| Precog, s (median of 3) | 0.049 | 0.103 |
| whole build, s (median of 3) | 0.104 | 0.159 |
| Precog, s, each run | 0.048600000000000004, 0.049, 0.048799999999999996 | 0.103, 0.1035, 0.1034 |
| the same sites and log every run | True | True |
| loop questions put to Spacer | | 1 (1 loops) |
| questions that spent the budget | | 0 (0 loops) |

<details><summary>Spacer's whole log</summary>

```
site line 11 (in MAIN), loop at line 8: found a way through its clauses to the bad value
```
</details>

## kayo/compiler/tests/programs/map_get_owned_copy.cy

| | Spacer off | Spacer on |
|---|---:|---:|
| sites | 32 | 32 |
| unproven (checked at run time) | 1 | 1 |
| Precog, s (median of 3) | 0.572 | 0.659 |
| whole build, s (median of 3) | 0.643 | 0.731 |
| Precog, s, each run | 0.5717000000000001, 0.5709, 0.584 | 0.6627000000000001, 0.6518999999999999, 0.6594 |
| the same sites and log every run | True | True |
| loop questions put to Spacer | | 3 (3 loops) |
| questions that spent the budget | | 0 (0 loops) |

<details><summary>Spacer's whole log</summary>

```
site line 23 (in MAIN), loop at line 11: found a way through its clauses to the bad value
site line 23 (in MAIN), loop at line 15: found a way through its clauses to the bad value
site line 23 (in MAIN), loop at line 16: not asked: a trip round the loop could not be put into words
```
</details>

## kayo/compiler/tests/programs/map_walk.cy

| | Spacer off | Spacer on |
|---|---:|---:|
| sites | 17 | 17 |
| unproven (checked at run time) | 2 | 2 |
| Precog, s (median of 3) | 2.42 | 2.48 |
| whole build, s (median of 3) | 2.48 | 2.54 |
| Precog, s, each run | 2.42, 2.42, 2.42 | 2.49, 2.48, 2.48 |
| the same sites and log every run | True | True |
| loop questions put to Spacer | | 3 (2 loops) |
| questions that spent the budget | | 0 (0 loops) |

<details><summary>Spacer's whole log</summary>

```
site line 10 (in MAIN), loop at line 9: found a way through its clauses to the bad value
site line 14 (in MAIN), loop at line 9: found a way through its clauses to the bad value
site line 14 (in MAIN), loop at line 13: found a way through its clauses to the bad value
```
</details>

## kayo/compiler/tests/programs/msl_narrowed_cross.cy

| | Spacer off | Spacer on |
|---|---:|---:|
| sites | 25 | 25 |
| unproven (checked at run time) | 1 | 1 |
| Precog, s (median of 3) | 1.17 | 1.28 |
| whole build, s (median of 3) | 1.3 | 1.42 |
| Precog, s, each run | 1.17, 1.17, 1.17 | 1.29, 1.28, 1.28 |
| the same sites and log every run | True | True |
| loop questions put to Spacer | | 1 (1 loops) |
| questions that spent the budget | | 0 (0 loops) |

<details><summary>Spacer's whole log</summary>

```
site line 13 (in total), loop at line 12: found a way through its clauses to the bad value
```
</details>

## kayo/compiler/tests/programs/msl_page_aligned.cy

| | Spacer off | Spacer on |
|---|---:|---:|
| sites | 21 | 21 |
| unproven (checked at run time) | 2 | 2 |
| Precog, s (median of 3) | 1.86 | 2.06 |
| whole build, s (median of 3) | 2.03 | 2.22 |
| Precog, s, each run | 1.86, 1.81, 1.91 | 2.06, 2.05, 2.06 |
| the same sites and log every run | True | True |
| loop questions put to Spacer | | 3 (2 loops) |
| questions that spent the budget | | 0 (0 loops) |

<details><summary>Spacer's whole log</summary>

```
site line 21 (in MAIN), loop at line 20: found a way through its clauses to the bad value
site line 25 (in MAIN), loop at line 20: found a way through its clauses to the bad value
site line 25 (in MAIN), loop at line 24: found a way through its clauses to the bad value
```
</details>

## kayo/compiler/tests/programs/msl_whole_pages.cy

| | Spacer off | Spacer on |
|---|---:|---:|
| sites | 15 | 15 |
| unproven (checked at run time) | 1 | 1 |
| Precog, s (median of 3) | 1.05 | 2.82 |
| whole build, s (median of 3) | 1.18 | 2.95 |
| Precog, s, each run | 1.05, 1.04, 1.05 | 3.26, 2.79, 2.82 |
| the same sites and log every run | True | True |
| loop questions put to Spacer | | 3 (3 loops) |
| questions that spent the budget | | 2 (2 loops) |

<details><summary>Spacer's whole log</summary>

```
site line 20 (in MAIN), loop at line 6: spent its step budget
site line 20 (in MAIN), loop at line 10: found a way through its clauses to the bad value
site line 20 (in MAIN), loop at line 18: spent its step budget
```
</details>

## kayo/compiler/tests/programs/nested_loop_exit.cy

| | Spacer off | Spacer on |
|---|---:|---:|
| sites | 13 | 13 |
| unproven (checked at run time) | 1 | 1 |
| Precog, s (median of 3) | 0.455 | 0.731 |
| whole build, s (median of 3) | 0.513 | 0.788 |
| Precog, s, each run | 0.4327, 0.4568, 0.45489999999999997 | 0.7313, 0.7421, 0.7126 |
| the same sites and log every run | True | True |
| loop questions put to Spacer | | 2 (2 loops) |
| questions that spent the budget | | 0 (0 loops) |

<details><summary>Spacer's whole log</summary>

```
site line 15 (in MAIN), loop at line 10: found a way through its clauses to the bad value
site line 15 (in MAIN), loop at line 12: not asked: a trip round the loop could not be put into words
```
</details>

## kayo/compiler/tests/programs/pairs_two_kinds.cy

| | Spacer off | Spacer on |
|---|---:|---:|
| sites | 34 | 34 |
| unproven (checked at run time) | 2 | 2 |
| Precog, s (median of 3) | 0.137 | 0.219 |
| whole build, s (median of 3) | 0.212 | 0.293 |
| Precog, s, each run | 0.134, 0.1389, 0.137 | 0.2189, 0.21830000000000002, 0.2286 |
| the same sites and log every run | True | True |
| loop questions put to Spacer | | 5 (3 loops) |
| questions that spent the budget | | 0 (0 loops) |

<details><summary>Spacer's whole log</summary>

```
site line 20 (in walk), loop at line 12: found a way through its clauses to the bad value
site line 20 (in walk), loop at line 18: found a way through its clauses to the bad value
site line 24 (in walk), loop at line 12: found a way through its clauses to the bad value
site line 24 (in walk), loop at line 18: found a way through its clauses to the bad value
site line 24 (in walk), loop at line 23: found a way through its clauses to the bad value
```
</details>

## kayo/compiler/tests/programs/par_lent_param.cy

| | Spacer off | Spacer on |
|---|---:|---:|
| sites | 20 | 20 |
| unproven (checked at run time) | 1 | 1 |
| Precog, s (median of 3) | 3.62 | 9.11 |
| whole build, s (median of 3) | 3.68 | 9.18 |
| Precog, s, each run | 3.62, 3.74, 3.59 | 9.2, 9.11, 9.11 |
| the same sites and log every run | True | True |
| loop questions put to Spacer | | 1 (1 loops) |
| questions that spent the budget | | 1 (1 loops) |

<details><summary>Spacer's whole log</summary>

```
site line 8 (in total), loop at line 7: spent its step budget
```
</details>

## kayo/compiler/tests/programs/par_trap.cy

| | Spacer off | Spacer on |
|---|---:|---:|
| sites | 10 | 10 |
| unproven (checked at run time) | 1 | 1 |
| Precog, s (median of 3) | 1.53 | 2.26 |
| whole build, s (median of 3) | 1.58 | 2.32 |
| Precog, s, each run | 1.53, 1.52, 1.53 | 2.24, 2.28, 2.26 |
| the same sites and log every run | True | True |
| loop questions put to Spacer | | 1 (1 loops) |
| questions that spent the budget | | 1 (1 loops) |

<details><summary>Spacer's whole log</summary>

```
site line 9 (in MAIN), loop at line 8: spent its step budget
```
</details>

## kayo/compiler/tests/programs/past_the_inner_unrolling.cy

| | Spacer off | Spacer on |
|---|---:|---:|
| sites | 24 | 24 |
| unproven (checked at run time) | 1 | 1 |
| Precog, s (median of 3) | 0.669 | 0.806 |
| whole build, s (median of 3) | 0.73 | 0.868 |
| Precog, s, each run | 0.6717000000000001, 0.6686, 0.6623 | 0.8055, 0.8124, 0.8062999999999999 |
| the same sites and log every run | True | True |
| loop questions put to Spacer | | 2 (2 loops) |
| questions that spent the budget | | 0 (0 loops) |

<details><summary>Spacer's whole log</summary>

```
site line 47 (in MAIN), loop at line 45: found a way through its clauses to the bad value
site line 47 (in MAIN), loop at line 46: not asked: a trip round the loop could not be put into words
```
</details>

## kayo/compiler/tests/programs/shl_in_a_loop.cy

| | Spacer off | Spacer on |
|---|---:|---:|
| sites | 13 | 13 |
| unproven (checked at run time) | 1 | 1 |
| Precog, s (median of 3) | 0.048 | 0.057 |
| whole build, s (median of 3) | 0.105 | 0.113 |
| Precog, s, each run | 0.045700000000000005, 0.0482, 0.0698 | 0.0565, 0.0567, 0.0541 |
| the same sites and log every run | True | True |
| loop questions put to Spacer | | 1 (1 loops) |
| questions that spent the budget | | 0 (0 loops) |

<details><summary>Spacer's whole log</summary>

```
site line 14 (in f1), loop at line 10: found a way through its clauses to the bad value
```
</details>

## kayo/compiler/tests/programs/two_loops_in_a_row.cy

| | Spacer off | Spacer on |
|---|---:|---:|
| sites | 9 | 9 |
| unproven (checked at run time) | 2 | 2 |
| Precog, s (median of 3) | 0.203 | 0.363 |
| whole build, s (median of 3) | 0.258 | 0.419 |
| Precog, s, each run | 0.2007, 0.2025, 0.2027 | 0.3629, 0.3633, 0.3638 |
| the same sites and log every run | True | True |
| loop questions put to Spacer | | 4 (2 loops) |
| questions that spent the budget | | 0 (0 loops) |

<details><summary>Spacer's whole log</summary>

```
site line 19 (in MAIN), loop at line 11: found a way through its clauses to the bad value
site line 19 (in MAIN), loop at line 15: found a way through its clauses to the bad value
site line 19 (in MAIN), loop at line 11: found a way through its clauses to the bad value
site line 19 (in MAIN), loop at line 15: found a way through its clauses to the bad value
```
</details>

## kayo/compiler/tests/programs/vacuous_after_exact_quotient.cy

| | Spacer off | Spacer on |
|---|---:|---:|
| sites | 33 | 33 |
| unproven (checked at run time) | 3 | 2 |
| Precog, s (median of 3) | 1.5 | 2.18 |
| whole build, s (median of 3) | 1.56 | 2.24 |
| Precog, s, each run | 1.5, 1.5, 1.5 | 2.18, 2.18, 2.18 |
| the same sites and log every run | True | True |
| loop questions put to Spacer | | 2 (2 loops) |
| questions that spent the budget | | 0 (0 loops) |

Newly proven:

- site line 28 (in emit), loop at line 27: confirmed: (bvsle (concat #b0 $k) (concat #b0 len($ds))); the site is proven

<details><summary>Spacer's whole log</summary>

```
site line 28 (in emit), loop at line 22: found a way through its clauses to the bad value
site line 28 (in emit), loop at line 27: confirmed: (bvsle (concat #b0 $k) (concat #b0 len($ds))); the site is proven
```
</details>

