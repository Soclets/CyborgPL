# Spacer's modes, measured

Made by `measure_modes.py` (its docstring says how). Budget: `rlimit` 10_000_000 of Z3's steps per loop question, in every mode. Times are the median of three builds, one at a time, from each build's report.

## Every program

| program | unproven off | unproven bits | unproven integers | unproven integers+gg | unproven joint | unproven joint+gg | unproven bits+gg | Precog off, s | Precog bits, s | Precog integers, s | Precog integers+gg, s | Precog joint, s | Precog joint+gg, s | Precog bits+gg, s |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| examples/args.cy | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 5.53 | 5.68 | 5.75 | 5.72 | 7.45 | 8.19 | 5.53 |
| examples/gradebook.cy | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 1.78 | 2.78 | 3.33 | 4.77 | 3.95 | 117.37 | 2.71 |
| examples/list_page_aligned.cy | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 0.81 | 0.922 | 0.824 | 0.819 | 0.821 | 0.818 | 0.935 |
| kayo/compiler/tests/programs/channels_handoff.cy | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 0.153 | 0.167 | 0.162 | 0.163 | 0.164 | 0.163 | 0.168 |
| kayo/compiler/tests/programs/channels_whichever.cy | 5 | 5 | 5 | 5 | 5 | 5 | 5 | 0.54 | 0.732 | 0.557 | 0.558 | 0.555 | 0.559 | 0.766 |
| kayo/compiler/tests/programs/incl_bytes_change.cy | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 0.162 | 0.174 | 0.17 | 0.171 | 0.169 | 0.17 | 0.176 |
| kayo/compiler/tests/programs/iterator_owned.cy | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 1.31 | 1.34 | 1.32 | 1.33 | 1.32 | 1.33 | 1.35 |
| kayo/compiler/tests/programs/iterator_walk.cy | 5 | 5 | 5 | 5 | 5 | 5 | 5 | 2.07 | 2.11 | 2.11 | 2.09 | 2.08 | 2.09 | 2.12 |
| kayo/compiler/tests/programs/lend_mut_through.cy | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 0.049 | 0.103 | 0.068 | 0.068 | 0.067 | 0.069 | 0.102 |
| kayo/compiler/tests/programs/map_get_owned_copy.cy | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 0.572 | 0.656 | 0.613 | 0.612 | 0.597 | 0.598 | 0.66 |
| kayo/compiler/tests/programs/map_walk.cy | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2.42 | 2.49 | 2.44 | 2.43 | 2.45 | 2.56 | 2.62 |
| kayo/compiler/tests/programs/msl_narrowed_cross.cy | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1.16 | 1.28 | 1.17 | 1.18 | 1.19 | 1.18 | 1.29 |
| kayo/compiler/tests/programs/msl_page_aligned.cy | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 1.8 | 2.02 | 1.9 | 1.87 | 1.88 | 1.88 | 2.12 |
| kayo/compiler/tests/programs/msl_whole_pages.cy | 1 | 1 | 1 | 1 | 1 | 0 | 1 | 1.06 | 2.85 | 3.13 | 4.26 | 2.67 | 4.36 | 2.72 |
| kayo/compiler/tests/programs/nested_loop_exit.cy | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 0.442 | 0.7 | 0.541 | 0.548 | 0.529 | 0.547 | 0.698 |
| kayo/compiler/tests/programs/pairs_two_kinds.cy | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 0.131 | 0.209 | 0.175 | 0.181 | 0.158 | 0.155 | 0.207 |
| kayo/compiler/tests/programs/par_lent_param.cy | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 3.58 | 9.29 | 3.61 | 3.61 | 3.69 | 3.62 | 9.12 |
| kayo/compiler/tests/programs/par_trap.cy | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 1.53 | 2.25 | 1.9 | 3.19 | 1.86 | 3.15 | 2.18 |
| kayo/compiler/tests/programs/past_the_inner_unrolling.cy | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 0.641 | 0.77 | 0.696 | 0.707 | 0.781 | 0.786 | 0.769 |
| kayo/compiler/tests/programs/shl_in_a_loop.cy | 1 | 1 | 1 | 1 | 1 | 1 | 1 | 0.043 | 0.053 | 0.052 | 0.051 | 0.052 | 0.052 | 0.053 |
| kayo/compiler/tests/programs/two_loops_in_a_row.cy | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 0.203 | 0.364 | 0.244 | 0.244 | 0.234 | 0.235 | 0.36 |
| kayo/compiler/tests/programs/vacuous_after_exact_quotient.cy | 3 | 2 | 2 | 2 | 2 | 2 | 2 | 1.5 | 2.18 | 1.66 | 1.66 | 1.67 | 1.67 | 2.17 |
| kayo/tools/spacer_probe/two_counters.cy | 2 | 1 | 1 | 0 | 1 | 0 | 1 | 0.874 | 1.33 | 2.21 | 1.81 | 2.23 | 1.83 | 1.79 |
| **all** | **42** | **40** | **40** | **39** | **40** | **38** | **40** | **28.36** | **40.45** | **34.63** | **38.04** | **36.57** | **153.38** | **40.61** |

| mode | unproven | Precog, s | whole builds, s | loop questions | spent the budget |
|---|---:|---:|---:|---:|---:|
| off | 42 | 28.36 | 30.02 | 0 | 0 |
| bits | 40 | 40.45 | 42.08 | 54 | 7 |
| integers | 40 | 34.63 | 36.30 | 54 | 6 |
| integers+gg | 39 | 38.04 | 39.71 | 54 | 5 |
| joint | 40 | 36.57 | 38.22 | 36 | 6 |
| joint+gg | 38 | 153.38 | 155.04 | 38 | 2 |
| bits+gg | 40 | 40.61 | 42.25 | 54 | 7 |

## examples/args.cy

| | off | bits | integers | integers+gg | joint | joint+gg | bits+gg |
|---|---:|---:|---:|---:|---:|---:|---:|
| unproven | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| Precog, s | 5.53 | 5.68 | 5.75 | 5.72 | 7.45 | 8.19 | 5.53 |
| whole build, s | 5.6 | 5.74 | 5.81 | 5.78 | 7.51 | 8.25 | 5.59 |
| spent the budget | 0 | 0 | 0 | 0 | 1 | 1 | 0 |
| the same every run | True | True | True | True | True | True | True |

<details><summary>Spacer's logs</summary>

bits:
```
site line 18 (in MAIN), loop at line 8: found a way through its clauses to the bad value
site line 18 (in MAIN), loop at line 14: not asked: a trip round the loop could not be put into words
```
integers:
```
site line 18 (in MAIN), loop at line 8: found a way through its clauses to the bad value
site line 18 (in MAIN), loop at line 14: not asked: a trip round the loop could not be put into words
```
integers+gg:
```
site line 18 (in MAIN), loop at line 8: found a way through its clauses to the bad value
site line 18 (in MAIN), loop at line 14: not asked: a trip round the loop could not be put into words
```
joint:
```
site line 18 (in MAIN), loop at line 8: spent its step budget
```
joint+gg:
```
site line 18 (in MAIN), loop at line 8: spent its step budget
```
bits+gg:
```
site line 18 (in MAIN), loop at line 8: found a way through its clauses to the bad value
site line 18 (in MAIN), loop at line 14: not asked: a trip round the loop could not be put into words
```
</details>

## examples/gradebook.cy

| | off | bits | integers | integers+gg | joint | joint+gg | bits+gg |
|---|---:|---:|---:|---:|---:|---:|---:|
| unproven | 2 | 2 | 2 | 2 | 2 | 2 | 2 |
| Precog, s | 1.78 | 2.78 | 3.33 | 4.77 | 3.95 | 117.37 | 2.71 |
| whole build, s | 1.85 | 2.85 | 3.4 | 4.84 | 4.02 | 117.44 | 2.78 |
| spent the budget | 0 | 2 | 2 | 2 | 2 | 0 | 2 |
| the same every run | True | True | True | True | True | True | True |

<details><summary>Spacer's logs</summary>

bits:
```
site line 15 (in average), loop at line 14: spent its step budget
site line 15 (in average), loop at line 14: spent its step budget
```
integers:
```
site line 15 (in average), loop at line 14: spent its step budget
site line 15 (in average), loop at line 14: spent its step budget
```
integers+gg:
```
site line 15 (in average), loop at line 14: spent its step budget
site line 15 (in average), loop at line 14: spent its step budget
```
joint:
```
site line 15 (in average), loop at line 14: spent its step budget
site line 15 (in average), loop at line 14: spent its step budget
```
joint+gg:
```
site line 15 (in average), loop at line 14: not asked: Spacer did not say: 
site line 15 (in average), loop at line 14: not asked: Spacer did not say: 
```
bits+gg:
```
site line 15 (in average), loop at line 14: spent its step budget
site line 15 (in average), loop at line 14: spent its step budget
```
</details>

## examples/list_page_aligned.cy

| | off | bits | integers | integers+gg | joint | joint+gg | bits+gg |
|---|---:|---:|---:|---:|---:|---:|---:|
| unproven | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| Precog, s | 0.81 | 0.922 | 0.824 | 0.819 | 0.821 | 0.818 | 0.935 |
| whole build, s | 0.87 | 0.983 | 0.885 | 0.881 | 0.882 | 0.879 | 0.995 |
| spent the budget | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| the same every run | True | True | True | True | True | True | True |

<details><summary>Spacer's logs</summary>

bits:
```
site line 8 (in total), loop at line 7: found a way through its clauses to the bad value
```
integers:
```
site line 8 (in total), loop at line 7: found a way through its clauses to the bad value
```
integers+gg:
```
site line 8 (in total), loop at line 7: found a way through its clauses to the bad value
```
joint:
```
site line 8 (in total), loop at line 7: found a way through its clauses to the bad value
```
joint+gg:
```
site line 8 (in total), loop at line 7: found a way through its clauses to the bad value
```
bits+gg:
```
site line 8 (in total), loop at line 7: found a way through its clauses to the bad value
```
</details>

## kayo/compiler/tests/programs/channels_handoff.cy

| | off | bits | integers | integers+gg | joint | joint+gg | bits+gg |
|---|---:|---:|---:|---:|---:|---:|---:|
| unproven | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| Precog, s | 0.153 | 0.167 | 0.162 | 0.163 | 0.164 | 0.163 | 0.168 |
| whole build, s | 0.21 | 0.225 | 0.221 | 0.222 | 0.221 | 0.221 | 0.226 |
| spent the budget | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| the same every run | True | True | True | True | True | True | True |

<details><summary>Spacer's logs</summary>

bits:
```
site line 17 (in MAIN), loop at line 15: found a way through its clauses to the bad value
```
integers:
```
site line 17 (in MAIN), loop at line 15: found a way through its clauses to the bad value
```
integers+gg:
```
site line 17 (in MAIN), loop at line 15: found a way through its clauses to the bad value
```
joint:
```
site line 17 (in MAIN), loop at line 15: found a way through its clauses to the bad value
```
joint+gg:
```
site line 17 (in MAIN), loop at line 15: found a way through its clauses to the bad value
```
bits+gg:
```
site line 17 (in MAIN), loop at line 15: found a way through its clauses to the bad value
```
</details>

## kayo/compiler/tests/programs/channels_whichever.cy

| | off | bits | integers | integers+gg | joint | joint+gg | bits+gg |
|---|---:|---:|---:|---:|---:|---:|---:|
| unproven | 5 | 5 | 5 | 5 | 5 | 5 | 5 |
| Precog, s | 0.54 | 0.732 | 0.557 | 0.558 | 0.555 | 0.559 | 0.766 |
| whole build, s | 0.6 | 0.792 | 0.617 | 0.618 | 0.616 | 0.619 | 0.827 |
| spent the budget | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| the same every run | True | True | True | True | True | True | True |

<details><summary>Spacer's logs</summary>

bits:
```
site line 26 (in MAIN), loop at line 23: found a way through its clauses to the bad value
site line 26 (in MAIN), loop at line 23: found a way through its clauses to the bad value
site line 29 (in MAIN), loop at line 23: found a way through its clauses to the bad value
```
integers:
```
site line 26 (in MAIN), loop at line 23: found a way through its clauses to the bad value
site line 26 (in MAIN), loop at line 23: found a way through its clauses to the bad value
site line 29 (in MAIN), loop at line 23: found a way through its clauses to the bad value
```
integers+gg:
```
site line 26 (in MAIN), loop at line 23: found a way through its clauses to the bad value
site line 26 (in MAIN), loop at line 23: found a way through its clauses to the bad value
site line 29 (in MAIN), loop at line 23: found a way through its clauses to the bad value
```
joint:
```
site line 26 (in MAIN), loop at line 23: found a way through its clauses to the bad value
site line 26 (in MAIN), loop at line 23: found a way through its clauses to the bad value
site line 29 (in MAIN), loop at line 23: found a way through its clauses to the bad value
```
joint+gg:
```
site line 26 (in MAIN), loop at line 23: found a way through its clauses to the bad value
site line 26 (in MAIN), loop at line 23: found a way through its clauses to the bad value
site line 29 (in MAIN), loop at line 23: found a way through its clauses to the bad value
```
bits+gg:
```
site line 26 (in MAIN), loop at line 23: found a way through its clauses to the bad value
site line 26 (in MAIN), loop at line 23: found a way through its clauses to the bad value
site line 29 (in MAIN), loop at line 23: found a way through its clauses to the bad value
```
</details>

## kayo/compiler/tests/programs/incl_bytes_change.cy

| | off | bits | integers | integers+gg | joint | joint+gg | bits+gg |
|---|---:|---:|---:|---:|---:|---:|---:|
| unproven | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| Precog, s | 0.162 | 0.174 | 0.17 | 0.171 | 0.169 | 0.17 | 0.176 |
| whole build, s | 0.218 | 0.23 | 0.226 | 0.226 | 0.226 | 0.226 | 0.232 |
| spent the budget | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| the same every run | True | True | True | True | True | True | True |

<details><summary>Spacer's logs</summary>

bits:
```
site line 22 (in MAIN), loop at line 20: found a way through its clauses to the bad value
```
integers:
```
site line 22 (in MAIN), loop at line 20: found a way through its clauses to the bad value
```
integers+gg:
```
site line 22 (in MAIN), loop at line 20: found a way through its clauses to the bad value
```
joint:
```
site line 22 (in MAIN), loop at line 20: found a way through its clauses to the bad value
```
joint+gg:
```
site line 22 (in MAIN), loop at line 20: found a way through its clauses to the bad value
```
bits+gg:
```
site line 22 (in MAIN), loop at line 20: found a way through its clauses to the bad value
```
</details>

## kayo/compiler/tests/programs/iterator_owned.cy

| | off | bits | integers | integers+gg | joint | joint+gg | bits+gg |
|---|---:|---:|---:|---:|---:|---:|---:|
| unproven | 4 | 4 | 4 | 4 | 4 | 4 | 4 |
| Precog, s | 1.31 | 1.34 | 1.32 | 1.33 | 1.32 | 1.33 | 1.35 |
| whole build, s | 1.37 | 1.4 | 1.38 | 1.39 | 1.38 | 1.39 | 1.41 |
| spent the budget | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| the same every run | True | True | True | True | True | True | True |

<details><summary>Spacer's logs</summary>

bits:
```
site line 23 (in MAIN), loop at line 22: found a way through its clauses to the bad value
site line 23 (in MAIN), loop at line 22: found a way through its clauses to the bad value
site line 27 (in MAIN), loop at line 22: found a way through its clauses to the bad value
site line 27 (in MAIN), loop at line 26: found a way through its clauses to the bad value
```
integers:
```
site line 23 (in MAIN), loop at line 22: found a way through its clauses to the bad value
site line 23 (in MAIN), loop at line 22: found a way through its clauses to the bad value
site line 27 (in MAIN), loop at line 22: found a way through its clauses to the bad value
site line 27 (in MAIN), loop at line 26: found a way through its clauses to the bad value
```
integers+gg:
```
site line 23 (in MAIN), loop at line 22: found a way through its clauses to the bad value
site line 23 (in MAIN), loop at line 22: found a way through its clauses to the bad value
site line 27 (in MAIN), loop at line 22: found a way through its clauses to the bad value
site line 27 (in MAIN), loop at line 26: found a way through its clauses to the bad value
```
joint:
```
site line 23 (in MAIN), loop at line 22: found a way through its clauses to the bad value
site line 23 (in MAIN), loop at line 22: found a way through its clauses to the bad value
site line 27 (in MAIN), loop at line 22: found a way through its clauses to the bad value
```
joint+gg:
```
site line 23 (in MAIN), loop at line 22: found a way through its clauses to the bad value
site line 23 (in MAIN), loop at line 22: found a way through its clauses to the bad value
site line 27 (in MAIN), loop at line 22: found a way through its clauses to the bad value
```
bits+gg:
```
site line 23 (in MAIN), loop at line 22: found a way through its clauses to the bad value
site line 23 (in MAIN), loop at line 22: found a way through its clauses to the bad value
site line 27 (in MAIN), loop at line 22: found a way through its clauses to the bad value
site line 27 (in MAIN), loop at line 26: found a way through its clauses to the bad value
```
</details>

## kayo/compiler/tests/programs/iterator_walk.cy

| | off | bits | integers | integers+gg | joint | joint+gg | bits+gg |
|---|---:|---:|---:|---:|---:|---:|---:|
| unproven | 5 | 5 | 5 | 5 | 5 | 5 | 5 |
| Precog, s | 2.07 | 2.11 | 2.11 | 2.09 | 2.08 | 2.09 | 2.12 |
| whole build, s | 2.13 | 2.17 | 2.17 | 2.15 | 2.14 | 2.15 | 2.17 |
| spent the budget | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| the same every run | True | True | True | True | True | True | True |

<details><summary>Spacer's logs</summary>

bits:
```
site line 42 (in MAIN), loop at line 38: found a way through its clauses to the bad value
site line 45 (in MAIN), loop at line 38: found a way through its clauses to the bad value
site line 45 (in MAIN), loop at line 44: found a way through its clauses to the bad value
site line 53 (in MAIN), loop at line 38: found a way through its clauses to the bad value
site line 53 (in MAIN), loop at line 44: found a way through its clauses to the bad value
site line 53 (in MAIN), loop at line 48: found a way through its clauses to the bad value
```
integers:
```
site line 42 (in MAIN), loop at line 38: found a way through its clauses to the bad value
site line 45 (in MAIN), loop at line 38: found a way through its clauses to the bad value
site line 45 (in MAIN), loop at line 44: found a way through its clauses to the bad value
site line 53 (in MAIN), loop at line 38: found a way through its clauses to the bad value
site line 53 (in MAIN), loop at line 44: found a way through its clauses to the bad value
site line 53 (in MAIN), loop at line 48: found a way through its clauses to the bad value
```
integers+gg:
```
site line 42 (in MAIN), loop at line 38: found a way through its clauses to the bad value
site line 45 (in MAIN), loop at line 38: found a way through its clauses to the bad value
site line 45 (in MAIN), loop at line 44: found a way through its clauses to the bad value
site line 53 (in MAIN), loop at line 38: found a way through its clauses to the bad value
site line 53 (in MAIN), loop at line 44: found a way through its clauses to the bad value
site line 53 (in MAIN), loop at line 48: found a way through its clauses to the bad value
```
joint:
```
site line 42 (in MAIN), loop at line 38: found a way through its clauses to the bad value
site line 45 (in MAIN), loop at line 38: found a way through its clauses to the bad value
site line 53 (in MAIN), loop at line 38: found a way through its clauses to the bad value
```
joint+gg:
```
site line 42 (in MAIN), loop at line 38: found a way through its clauses to the bad value
site line 45 (in MAIN), loop at line 38: found a way through its clauses to the bad value
site line 53 (in MAIN), loop at line 38: found a way through its clauses to the bad value
```
bits+gg:
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

| | off | bits | integers | integers+gg | joint | joint+gg | bits+gg |
|---|---:|---:|---:|---:|---:|---:|---:|
| unproven | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| Precog, s | 0.049 | 0.103 | 0.068 | 0.068 | 0.067 | 0.069 | 0.102 |
| whole build, s | 0.105 | 0.159 | 0.122 | 0.122 | 0.122 | 0.124 | 0.158 |
| spent the budget | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| the same every run | True | True | True | True | True | True | True |

<details><summary>Spacer's logs</summary>

bits:
```
site line 11 (in MAIN), loop at line 8: found a way through its clauses to the bad value
```
integers:
```
site line 11 (in MAIN), loop at line 8: found a way through its clauses to the bad value
```
integers+gg:
```
site line 11 (in MAIN), loop at line 8: found a way through its clauses to the bad value
```
joint:
```
site line 11 (in MAIN), loop at line 8: found a way through its clauses to the bad value
```
joint+gg:
```
site line 11 (in MAIN), loop at line 8: found a way through its clauses to the bad value
```
bits+gg:
```
site line 11 (in MAIN), loop at line 8: found a way through its clauses to the bad value
```
</details>

## kayo/compiler/tests/programs/map_get_owned_copy.cy

| | off | bits | integers | integers+gg | joint | joint+gg | bits+gg |
|---|---:|---:|---:|---:|---:|---:|---:|
| unproven | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| Precog, s | 0.572 | 0.656 | 0.613 | 0.612 | 0.597 | 0.598 | 0.66 |
| whole build, s | 0.643 | 0.728 | 0.684 | 0.684 | 0.67 | 0.672 | 0.733 |
| spent the budget | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| the same every run | True | True | True | True | True | True | True |

<details><summary>Spacer's logs</summary>

bits:
```
site line 23 (in MAIN), loop at line 11: found a way through its clauses to the bad value
site line 23 (in MAIN), loop at line 15: found a way through its clauses to the bad value
site line 23 (in MAIN), loop at line 16: not asked: a trip round the loop could not be put into words
```
integers:
```
site line 23 (in MAIN), loop at line 11: found a way through its clauses to the bad value
site line 23 (in MAIN), loop at line 15: found a way through its clauses to the bad value
site line 23 (in MAIN), loop at line 16: not asked: a trip round the loop could not be put into words
```
integers+gg:
```
site line 23 (in MAIN), loop at line 11: found a way through its clauses to the bad value
site line 23 (in MAIN), loop at line 15: found a way through its clauses to the bad value
site line 23 (in MAIN), loop at line 16: not asked: a trip round the loop could not be put into words
```
joint:
```
site line 23 (in MAIN), loop at line 11: found a way through its clauses to the bad value
```
joint+gg:
```
site line 23 (in MAIN), loop at line 11: found a way through its clauses to the bad value
```
bits+gg:
```
site line 23 (in MAIN), loop at line 11: found a way through its clauses to the bad value
site line 23 (in MAIN), loop at line 15: found a way through its clauses to the bad value
site line 23 (in MAIN), loop at line 16: not asked: a trip round the loop could not be put into words
```
</details>

## kayo/compiler/tests/programs/map_walk.cy

| | off | bits | integers | integers+gg | joint | joint+gg | bits+gg |
|---|---:|---:|---:|---:|---:|---:|---:|
| unproven | 2 | 2 | 2 | 2 | 2 | 2 | 2 |
| Precog, s | 2.42 | 2.49 | 2.44 | 2.43 | 2.45 | 2.56 | 2.62 |
| whole build, s | 2.48 | 2.55 | 2.51 | 2.5 | 2.51 | 2.62 | 2.68 |
| spent the budget | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| the same every run | True | True | True | True | True | True | True |

<details><summary>Spacer's logs</summary>

bits:
```
site line 10 (in MAIN), loop at line 9: found a way through its clauses to the bad value
site line 14 (in MAIN), loop at line 9: found a way through its clauses to the bad value
site line 14 (in MAIN), loop at line 13: found a way through its clauses to the bad value
```
integers:
```
site line 10 (in MAIN), loop at line 9: found a way through its clauses to the bad value
site line 14 (in MAIN), loop at line 9: found a way through its clauses to the bad value
site line 14 (in MAIN), loop at line 13: found a way through its clauses to the bad value
```
integers+gg:
```
site line 10 (in MAIN), loop at line 9: found a way through its clauses to the bad value
site line 14 (in MAIN), loop at line 9: found a way through its clauses to the bad value
site line 14 (in MAIN), loop at line 13: found a way through its clauses to the bad value
```
joint:
```
site line 10 (in MAIN), loop at line 9: found a way through its clauses to the bad value
site line 14 (in MAIN), loop at line 9: found a way through its clauses to the bad value
```
joint+gg:
```
site line 10 (in MAIN), loop at line 9: found a way through its clauses to the bad value
site line 14 (in MAIN), loop at line 9: found a way through its clauses to the bad value
```
bits+gg:
```
site line 10 (in MAIN), loop at line 9: found a way through its clauses to the bad value
site line 14 (in MAIN), loop at line 9: found a way through its clauses to the bad value
site line 14 (in MAIN), loop at line 13: found a way through its clauses to the bad value
```
</details>

## kayo/compiler/tests/programs/msl_narrowed_cross.cy

| | off | bits | integers | integers+gg | joint | joint+gg | bits+gg |
|---|---:|---:|---:|---:|---:|---:|---:|
| unproven | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| Precog, s | 1.16 | 1.28 | 1.17 | 1.18 | 1.19 | 1.18 | 1.29 |
| whole build, s | 1.29 | 1.41 | 1.31 | 1.32 | 1.32 | 1.32 | 1.43 |
| spent the budget | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| the same every run | True | True | True | True | True | True | True |

<details><summary>Spacer's logs</summary>

bits:
```
site line 13 (in total), loop at line 12: found a way through its clauses to the bad value
```
integers:
```
site line 13 (in total), loop at line 12: found a way through its clauses to the bad value
```
integers+gg:
```
site line 13 (in total), loop at line 12: found a way through its clauses to the bad value
```
joint:
```
site line 13 (in total), loop at line 12: found a way through its clauses to the bad value
```
joint+gg:
```
site line 13 (in total), loop at line 12: found a way through its clauses to the bad value
```
bits+gg:
```
site line 13 (in total), loop at line 12: found a way through its clauses to the bad value
```
</details>

## kayo/compiler/tests/programs/msl_page_aligned.cy

| | off | bits | integers | integers+gg | joint | joint+gg | bits+gg |
|---|---:|---:|---:|---:|---:|---:|---:|
| unproven | 2 | 2 | 2 | 2 | 2 | 2 | 2 |
| Precog, s | 1.8 | 2.02 | 1.9 | 1.87 | 1.88 | 1.88 | 2.12 |
| whole build, s | 1.96 | 2.18 | 2.08 | 2.04 | 2.05 | 2.05 | 2.29 |
| spent the budget | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| the same every run | True | True | True | True | True | True | True |

<details><summary>Spacer's logs</summary>

bits:
```
site line 21 (in MAIN), loop at line 20: found a way through its clauses to the bad value
site line 25 (in MAIN), loop at line 20: found a way through its clauses to the bad value
site line 25 (in MAIN), loop at line 24: found a way through its clauses to the bad value
```
integers:
```
site line 21 (in MAIN), loop at line 20: found a way through its clauses to the bad value
site line 25 (in MAIN), loop at line 20: found a way through its clauses to the bad value
site line 25 (in MAIN), loop at line 24: found a way through its clauses to the bad value
```
integers+gg:
```
site line 21 (in MAIN), loop at line 20: found a way through its clauses to the bad value
site line 25 (in MAIN), loop at line 20: found a way through its clauses to the bad value
site line 25 (in MAIN), loop at line 24: found a way through its clauses to the bad value
```
joint:
```
site line 21 (in MAIN), loop at line 20: found a way through its clauses to the bad value
site line 25 (in MAIN), loop at line 20: found a way through its clauses to the bad value
```
joint+gg:
```
site line 21 (in MAIN), loop at line 20: found a way through its clauses to the bad value
site line 25 (in MAIN), loop at line 20: found a way through its clauses to the bad value
```
bits+gg:
```
site line 21 (in MAIN), loop at line 20: found a way through its clauses to the bad value
site line 25 (in MAIN), loop at line 20: found a way through its clauses to the bad value
site line 25 (in MAIN), loop at line 24: found a way through its clauses to the bad value
```
</details>

## kayo/compiler/tests/programs/msl_whole_pages.cy

| | off | bits | integers | integers+gg | joint | joint+gg | bits+gg |
|---|---:|---:|---:|---:|---:|---:|---:|
| unproven | 1 | 1 | 1 | 1 | 1 | 0 | 1 |
| Precog, s | 1.06 | 2.85 | 3.13 | 4.26 | 2.67 | 4.36 | 2.72 |
| whole build, s | 1.21 | 2.98 | 3.26 | 4.4 | 2.8 | 4.5 | 2.85 |
| spent the budget | 0 | 2 | 2 | 2 | 1 | 0 | 2 |
| the same every run | True | True | True | True | True | True | True |

**joint+gg**: 1 newly proven
- proven: site line 20 (in MAIN), loop at line 6: confirmed: (let ((a!1 (or (not (<= (+ len($xs) _7) 9223372036854775806)) (not (<= $i 16383)) (not (>= (+ len($xs) _7) 2147483647)))) (a!2 (not (>= (+ $i (* (- 1) _6)) 0))) (a!3 (not (>= (+ $i (* (- 1) len($xs))) (- 9223372036854759422)))) (a!4 (not (<= (+ $i (* (- 1) len($xs))) (- 2147467263)))) (a!5 (not (<= (+ $i (* (- 1) len($xs)) (* (- 1) _7)) (- 2147467264)))) (a!6 (not (>= (+ $i (* (- 1) len($xs)) (* (- 1) _7)) (- 9223372036854759423))))) (and a!1 (or a!2 (not (>= len($xs) 2147483647)) (not (<= len($xs) 9223372036854775806))) (or (not (>= len($xs) 2147483647)) (not (>= $i 16384)) (not (<= len($xs) 9223372036854775806))) (or a!3 (not (>= $i 16382)) (not (= _6 16384)) (not (= _7 1)) a!4 (not (<= $i 16383))) (or a!5 a!6))); the site is proven
- proven: site line 20 (in MAIN), loop at line 10: confirmed: (let ((a!1 (not (>= (+ $round (* (- 1) _20) (* (- 1) _19)) (- 1))))) (and (or (not (>= len($xs) 2147483648)) a!1) (or (not (<= len($xs) 9223372036854775807)) (not (>= len($xs) 2147483648))) (or (not (>= $round 2)) (not (>= len($xs) 2147483648))))); the site is proven
- proven: site line 20 (in MAIN), loop at line 18: confirmed: (not (<= (+ _37 (* (- 1) $wrong) (* (- 1) _36)) (- 2147483647))); the site is proven

<details><summary>Spacer's logs</summary>

bits:
```
site line 20 (in MAIN), loop at line 6: spent its step budget
site line 20 (in MAIN), loop at line 10: found a way through its clauses to the bad value
site line 20 (in MAIN), loop at line 18: spent its step budget
```
integers:
```
site line 20 (in MAIN), loop at line 6: spent its step budget
site line 20 (in MAIN), loop at line 10: found a way through its clauses to the bad value
site line 20 (in MAIN), loop at line 18: spent its step budget
```
integers+gg:
```
site line 20 (in MAIN), loop at line 6: spent its step budget
site line 20 (in MAIN), loop at line 10: found a way through its clauses to the bad value
site line 20 (in MAIN), loop at line 18: spent its step budget
```
joint:
```
site line 20 (in MAIN), loop at line 6: spent its step budget
```
joint+gg:
```
site line 20 (in MAIN), loop at line 6: confirmed: (let ((a!1 (or (not (<= (+ len($xs) _7) 9223372036854775806)) (not (<= $i 16383)) (not (>= (+ len($xs) _7) 2147483647)))) (a!2 (not (>= (+ $i (* (- 1) _6)) 0))) (a!3 (not (>= (+ $i (* (- 1) len($xs))) (- 9223372036854759422)))) (a!4 (not (<= (+ $i (* (- 1) len($xs))) (- 2147467263)))) (a!5 (not (<= (+ $i (* (- 1) len($xs)) (* (- 1) _7)) (- 2147467264)))) (a!6 (not (>= (+ $i (* (- 1) len($xs)) (* (- 1) _7)) (- 9223372036854759423))))) (and a!1 (or a!2 (not (>= len($xs) 2147483647)) (not (<= len($xs) 9223372036854775806))) (or (not (>= len($xs) 2147483647)) (not (>= $i 16384)) (not (<= len($xs) 9223372036854775806))) (or a!3 (not (>= $i 16382)) (not (= _6 16384)) (not (= _7 1)) a!4 (not (<= $i 16383))) (or a!5 a!6))); the site is proven
site line 20 (in MAIN), loop at line 10: confirmed: (let ((a!1 (not (>= (+ $round (* (- 1) _20) (* (- 1) _19)) (- 1))))) (and (or (not (>= len($xs) 2147483648)) a!1) (or (not (<= len($xs) 9223372036854775807)) (not (>= len($xs) 2147483648))) (or (not (>= $round 2)) (not (>= len($xs) 2147483648))))); the site is proven
site line 20 (in MAIN), loop at line 18: confirmed: (not (<= (+ _37 (* (- 1) $wrong) (* (- 1) _36)) (- 2147483647))); the site is proven
```
bits+gg:
```
site line 20 (in MAIN), loop at line 6: spent its step budget
site line 20 (in MAIN), loop at line 10: found a way through its clauses to the bad value
site line 20 (in MAIN), loop at line 18: spent its step budget
```
</details>

## kayo/compiler/tests/programs/nested_loop_exit.cy

| | off | bits | integers | integers+gg | joint | joint+gg | bits+gg |
|---|---:|---:|---:|---:|---:|---:|---:|
| unproven | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| Precog, s | 0.442 | 0.7 | 0.541 | 0.548 | 0.529 | 0.547 | 0.698 |
| whole build, s | 0.499 | 0.754 | 0.596 | 0.604 | 0.587 | 0.602 | 0.753 |
| spent the budget | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| the same every run | True | True | True | True | True | True | True |

<details><summary>Spacer's logs</summary>

bits:
```
site line 15 (in MAIN), loop at line 10: found a way through its clauses to the bad value
site line 15 (in MAIN), loop at line 12: not asked: a trip round the loop could not be put into words
```
integers:
```
site line 15 (in MAIN), loop at line 10: found a way through its clauses to the bad value
site line 15 (in MAIN), loop at line 12: not asked: a trip round the loop could not be put into words
```
integers+gg:
```
site line 15 (in MAIN), loop at line 10: found a way through its clauses to the bad value
site line 15 (in MAIN), loop at line 12: not asked: a trip round the loop could not be put into words
```
joint:
```
site line 15 (in MAIN), loop at line 10: found a way through its clauses to the bad value
```
joint+gg:
```
site line 15 (in MAIN), loop at line 10: found a way through its clauses to the bad value
```
bits+gg:
```
site line 15 (in MAIN), loop at line 10: found a way through its clauses to the bad value
site line 15 (in MAIN), loop at line 12: not asked: a trip round the loop could not be put into words
```
</details>

## kayo/compiler/tests/programs/pairs_two_kinds.cy

| | off | bits | integers | integers+gg | joint | joint+gg | bits+gg |
|---|---:|---:|---:|---:|---:|---:|---:|
| unproven | 2 | 2 | 2 | 2 | 2 | 2 | 2 |
| Precog, s | 0.131 | 0.209 | 0.175 | 0.181 | 0.158 | 0.155 | 0.207 |
| whole build, s | 0.203 | 0.281 | 0.249 | 0.257 | 0.231 | 0.226 | 0.278 |
| spent the budget | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| the same every run | True | True | True | True | True | True | True |

<details><summary>Spacer's logs</summary>

bits:
```
site line 20 (in walk), loop at line 12: found a way through its clauses to the bad value
site line 20 (in walk), loop at line 18: found a way through its clauses to the bad value
site line 24 (in walk), loop at line 12: found a way through its clauses to the bad value
site line 24 (in walk), loop at line 18: found a way through its clauses to the bad value
site line 24 (in walk), loop at line 23: found a way through its clauses to the bad value
```
integers:
```
site line 20 (in walk), loop at line 12: found a way through its clauses to the bad value
site line 20 (in walk), loop at line 18: found a way through its clauses to the bad value
site line 24 (in walk), loop at line 12: found a way through its clauses to the bad value
site line 24 (in walk), loop at line 18: found a way through its clauses to the bad value
site line 24 (in walk), loop at line 23: found a way through its clauses to the bad value
```
integers+gg:
```
site line 20 (in walk), loop at line 12: found a way through its clauses to the bad value
site line 20 (in walk), loop at line 18: found a way through its clauses to the bad value
site line 24 (in walk), loop at line 12: found a way through its clauses to the bad value
site line 24 (in walk), loop at line 18: found a way through its clauses to the bad value
site line 24 (in walk), loop at line 23: found a way through its clauses to the bad value
```
joint:
```
site line 20 (in walk), loop at line 12: found a way through its clauses to the bad value
site line 24 (in walk), loop at line 12: found a way through its clauses to the bad value
```
joint+gg:
```
site line 20 (in walk), loop at line 12: found a way through its clauses to the bad value
site line 24 (in walk), loop at line 12: found a way through its clauses to the bad value
```
bits+gg:
```
site line 20 (in walk), loop at line 12: found a way through its clauses to the bad value
site line 20 (in walk), loop at line 18: found a way through its clauses to the bad value
site line 24 (in walk), loop at line 12: found a way through its clauses to the bad value
site line 24 (in walk), loop at line 18: found a way through its clauses to the bad value
site line 24 (in walk), loop at line 23: found a way through its clauses to the bad value
```
</details>

## kayo/compiler/tests/programs/par_lent_param.cy

| | off | bits | integers | integers+gg | joint | joint+gg | bits+gg |
|---|---:|---:|---:|---:|---:|---:|---:|
| unproven | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| Precog, s | 3.58 | 9.29 | 3.61 | 3.61 | 3.69 | 3.62 | 9.12 |
| whole build, s | 3.65 | 9.36 | 3.68 | 3.68 | 3.76 | 3.69 | 9.19 |
| spent the budget | 0 | 1 | 0 | 0 | 0 | 0 | 1 |
| the same every run | True | True | True | True | True | True | True |

<details><summary>Spacer's logs</summary>

bits:
```
site line 8 (in total), loop at line 7: spent its step budget
```
integers:
```
site line 8 (in total), loop at line 7: found a way through its clauses to the bad value
```
integers+gg:
```
site line 8 (in total), loop at line 7: found a way through its clauses to the bad value
```
joint:
```
site line 8 (in total), loop at line 7: found a way through its clauses to the bad value
```
joint+gg:
```
site line 8 (in total), loop at line 7: found a way through its clauses to the bad value
```
bits+gg:
```
site line 8 (in total), loop at line 7: spent its step budget
```
</details>

## kayo/compiler/tests/programs/par_trap.cy

| | off | bits | integers | integers+gg | joint | joint+gg | bits+gg |
|---|---:|---:|---:|---:|---:|---:|---:|
| unproven | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| Precog, s | 1.53 | 2.25 | 1.9 | 3.19 | 1.86 | 3.15 | 2.18 |
| whole build, s | 1.59 | 2.31 | 1.96 | 3.25 | 1.92 | 3.2 | 2.23 |
| spent the budget | 0 | 1 | 1 | 1 | 1 | 1 | 1 |
| the same every run | True | True | True | True | True | True | True |

<details><summary>Spacer's logs</summary>

bits:
```
site line 9 (in MAIN), loop at line 8: spent its step budget
```
integers:
```
site line 9 (in MAIN), loop at line 8: spent its step budget
```
integers+gg:
```
site line 9 (in MAIN), loop at line 8: spent its step budget
```
joint:
```
site line 9 (in MAIN), loop at line 8: spent its step budget
```
joint+gg:
```
site line 9 (in MAIN), loop at line 8: spent its step budget
```
bits+gg:
```
site line 9 (in MAIN), loop at line 8: spent its step budget
```
</details>

## kayo/compiler/tests/programs/past_the_inner_unrolling.cy

| | off | bits | integers | integers+gg | joint | joint+gg | bits+gg |
|---|---:|---:|---:|---:|---:|---:|---:|
| unproven | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| Precog, s | 0.641 | 0.77 | 0.696 | 0.707 | 0.781 | 0.786 | 0.769 |
| whole build, s | 0.699 | 0.829 | 0.755 | 0.765 | 0.84 | 0.845 | 0.829 |
| spent the budget | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| the same every run | True | True | True | True | True | True | True |

<details><summary>Spacer's logs</summary>

bits:
```
site line 47 (in MAIN), loop at line 45: found a way through its clauses to the bad value
site line 47 (in MAIN), loop at line 46: not asked: a trip round the loop could not be put into words
```
integers:
```
site line 47 (in MAIN), loop at line 45: found a way through its clauses to the bad value
site line 47 (in MAIN), loop at line 46: not asked: a trip round the loop could not be put into words
```
integers+gg:
```
site line 47 (in MAIN), loop at line 45: found a way through its clauses to the bad value
site line 47 (in MAIN), loop at line 46: not asked: a trip round the loop could not be put into words
```
joint:
```
site line 47 (in MAIN), loop at line 45: found a way through its clauses to the bad value
```
joint+gg:
```
site line 47 (in MAIN), loop at line 45: found a way through its clauses to the bad value
```
bits+gg:
```
site line 47 (in MAIN), loop at line 45: found a way through its clauses to the bad value
site line 47 (in MAIN), loop at line 46: not asked: a trip round the loop could not be put into words
```
</details>

## kayo/compiler/tests/programs/shl_in_a_loop.cy

| | off | bits | integers | integers+gg | joint | joint+gg | bits+gg |
|---|---:|---:|---:|---:|---:|---:|---:|
| unproven | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| Precog, s | 0.043 | 0.053 | 0.052 | 0.051 | 0.052 | 0.052 | 0.053 |
| whole build, s | 0.096 | 0.106 | 0.106 | 0.105 | 0.105 | 0.106 | 0.106 |
| spent the budget | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| the same every run | True | True | True | True | True | True | True |

<details><summary>Spacer's logs</summary>

bits:
```
site line 14 (in f1), loop at line 10: found a way through its clauses to the bad value
```
integers:
```
site line 14 (in f1), loop at line 10: found a way through its clauses to the bad value
```
integers+gg:
```
site line 14 (in f1), loop at line 10: found a way through its clauses to the bad value
```
joint:
```
site line 14 (in f1), loop at line 10: found a way through its clauses to the bad value
```
joint+gg:
```
site line 14 (in f1), loop at line 10: found a way through its clauses to the bad value
```
bits+gg:
```
site line 14 (in f1), loop at line 10: found a way through its clauses to the bad value
```
</details>

## kayo/compiler/tests/programs/two_loops_in_a_row.cy

| | off | bits | integers | integers+gg | joint | joint+gg | bits+gg |
|---|---:|---:|---:|---:|---:|---:|---:|
| unproven | 2 | 2 | 2 | 2 | 2 | 2 | 2 |
| Precog, s | 0.203 | 0.364 | 0.244 | 0.244 | 0.234 | 0.235 | 0.36 |
| whole build, s | 0.258 | 0.42 | 0.299 | 0.299 | 0.289 | 0.291 | 0.414 |
| spent the budget | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| the same every run | True | True | True | True | True | True | True |

<details><summary>Spacer's logs</summary>

bits:
```
site line 19 (in MAIN), loop at line 11: found a way through its clauses to the bad value
site line 19 (in MAIN), loop at line 15: found a way through its clauses to the bad value
site line 19 (in MAIN), loop at line 11: found a way through its clauses to the bad value
site line 19 (in MAIN), loop at line 15: found a way through its clauses to the bad value
```
integers:
```
site line 19 (in MAIN), loop at line 11: found a way through its clauses to the bad value
site line 19 (in MAIN), loop at line 15: found a way through its clauses to the bad value
site line 19 (in MAIN), loop at line 11: found a way through its clauses to the bad value
site line 19 (in MAIN), loop at line 15: found a way through its clauses to the bad value
```
integers+gg:
```
site line 19 (in MAIN), loop at line 11: found a way through its clauses to the bad value
site line 19 (in MAIN), loop at line 15: found a way through its clauses to the bad value
site line 19 (in MAIN), loop at line 11: found a way through its clauses to the bad value
site line 19 (in MAIN), loop at line 15: found a way through its clauses to the bad value
```
joint:
```
site line 19 (in MAIN), loop at line 11: found a way through its clauses to the bad value
site line 19 (in MAIN), loop at line 11: found a way through its clauses to the bad value
```
joint+gg:
```
site line 19 (in MAIN), loop at line 11: found a way through its clauses to the bad value
site line 19 (in MAIN), loop at line 11: found a way through its clauses to the bad value
```
bits+gg:
```
site line 19 (in MAIN), loop at line 11: found a way through its clauses to the bad value
site line 19 (in MAIN), loop at line 15: found a way through its clauses to the bad value
site line 19 (in MAIN), loop at line 11: found a way through its clauses to the bad value
site line 19 (in MAIN), loop at line 15: found a way through its clauses to the bad value
```
</details>

## kayo/compiler/tests/programs/vacuous_after_exact_quotient.cy

| | off | bits | integers | integers+gg | joint | joint+gg | bits+gg |
|---|---:|---:|---:|---:|---:|---:|---:|
| unproven | 3 | 2 | 2 | 2 | 2 | 2 | 2 |
| Precog, s | 1.5 | 2.18 | 1.66 | 1.66 | 1.67 | 1.67 | 2.17 |
| whole build, s | 1.56 | 2.23 | 1.71 | 1.72 | 1.73 | 1.73 | 2.23 |
| spent the budget | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| the same every run | True | True | True | True | True | True | True |

**bits**: 1 newly proven
- proven: site line 28 (in emit), loop at line 27: confirmed: (bvsle (concat #b0 $k) (concat #b0 len($ds))); the site is proven

**integers**: 1 newly proven
- proven: site line 28 (in emit), loop at line 27: confirmed: (not (>= (+ $k (* (- 1) len($ds))) 1)); the site is proven

**integers+gg**: 1 newly proven
- proven: site line 28 (in emit), loop at line 27: confirmed: (not (>= (+ $k (* (- 1) len($ds))) 1)); the site is proven

**joint**: 1 newly proven
- proven: site line 28 (in emit), loop at line 22: confirmed: true; the site is proven
- proven: site line 28 (in emit), loop at line 27: confirmed: (not (>= (+ $k (* (- 1) len($ds))) 1)); the site is proven

**joint+gg**: 1 newly proven
- proven: site line 28 (in emit), loop at line 22: confirmed: true; the site is proven
- proven: site line 28 (in emit), loop at line 27: confirmed: (not (>= (+ $k (* (- 1) len($ds))) 1)); the site is proven

**bits+gg**: 1 newly proven
- proven: site line 28 (in emit), loop at line 27: confirmed: (bvsle (concat #b0 $k) (concat #b0 len($ds))); the site is proven

<details><summary>Spacer's logs</summary>

bits:
```
site line 28 (in emit), loop at line 22: found a way through its clauses to the bad value
site line 28 (in emit), loop at line 27: confirmed: (bvsle (concat #b0 $k) (concat #b0 len($ds))); the site is proven
```
integers:
```
site line 28 (in emit), loop at line 22: found a way through its clauses to the bad value
site line 28 (in emit), loop at line 27: confirmed: (not (>= (+ $k (* (- 1) len($ds))) 1)); the site is proven
```
integers+gg:
```
site line 28 (in emit), loop at line 22: found a way through its clauses to the bad value
site line 28 (in emit), loop at line 27: confirmed: (not (>= (+ $k (* (- 1) len($ds))) 1)); the site is proven
```
joint:
```
site line 28 (in emit), loop at line 22: confirmed: true; the site is proven
site line 28 (in emit), loop at line 27: confirmed: (not (>= (+ $k (* (- 1) len($ds))) 1)); the site is proven
```
joint+gg:
```
site line 28 (in emit), loop at line 22: confirmed: true; the site is proven
site line 28 (in emit), loop at line 27: confirmed: (not (>= (+ $k (* (- 1) len($ds))) 1)); the site is proven
```
bits+gg:
```
site line 28 (in emit), loop at line 22: found a way through its clauses to the bad value
site line 28 (in emit), loop at line 27: confirmed: (bvsle (concat #b0 $k) (concat #b0 len($ds))); the site is proven
```
</details>

## kayo/tools/spacer_probe/two_counters.cy

| | off | bits | integers | integers+gg | joint | joint+gg | bits+gg |
|---|---:|---:|---:|---:|---:|---:|---:|
| unproven | 2 | 1 | 1 | 0 | 1 | 0 | 1 |
| Precog, s | 0.874 | 1.33 | 2.21 | 1.81 | 2.23 | 1.83 | 1.79 |
| whole build, s | 0.929 | 1.39 | 2.27 | 1.86 | 2.29 | 1.89 | 1.85 |
| spent the budget | 0 | 1 | 1 | 0 | 1 | 0 | 1 |
| the same every run | True | True | True | True | True | True | True |

**bits**: 1 newly proven
- proven: site line 10 (in gap), loop at line 6: confirmed: (let ((a!1 (= (concat ((_ extract 31 31) $i) ((_ extract 31 31) $i) $i) (bvadd #b0000000000000000000000000000000001 (concat ((_ extract 31 31) $j) ((_ extract 31 31) $j) $j))))) (not a!1)); the site is proven

**integers**: 1 newly proven
- proven: site line 10 (in gap), loop at line 6: confirmed: (not (>= (+ $i (* (- 1) $j)) 1)); the site is proven

**integers+gg**: 2 newly proven
- proven: site line 8 (in gap), loop at line 6: confirmed: (let ((a!1 (not (>= (+ $j (* (- 1) $i)) 1))) (a!2 (not (>= (+ $n $j (* (- 1) $i)) 2147483648)))) (or (not (<= $n 2147483647)) (not (>= $n (- 2147483648))) (not (<= $j 2147483647)) a!1 a!2 (not (>= $i 0)))); the site is proven
- proven: site line 10 (in gap), loop at line 6: confirmed: (not (>= (+ $i (* (- 1) $j)) 1)); the site is proven

**joint**: 1 newly proven
- proven: site line 10 (in gap), loop at line 6: confirmed: (not (>= (+ $i (* (- 1) $j)) 1)); the site is proven

**joint+gg**: 2 newly proven
- proven: site line 8 (in gap), loop at line 6: confirmed: (let ((a!1 (not (<= (+ $i (* (- 1) $n) (* (- 1) $j)) (- 2147483648)))) (a!2 (not (<= (+ $i (* (- 1) $j)) (- 1))))) (or (not (<= $n 2147483647)) (not (>= $n (- 2147483648))) (not (<= $j 2147483647)) a!1 a!2 (not (>= $i 0)))); the site is proven
- proven: site line 10 (in gap), loop at line 6: confirmed: (not (>= (+ $i (* (- 1) $j)) 1)); the site is proven

**bits+gg**: 1 newly proven
- proven: site line 10 (in gap), loop at line 6: confirmed: (let ((a!1 (= (concat ((_ extract 31 31) $i) ((_ extract 31 31) $i) $i) (bvadd #b0000000000000000000000000000000001 (concat ((_ extract 31 31) $j) ((_ extract 31 31) $j) $j))))) (not a!1)); the site is proven

<details><summary>Spacer's logs</summary>

bits:
```
site line 8 (in gap), loop at line 6: spent its step budget
site line 10 (in gap), loop at line 6: confirmed: (let ((a!1 (= (concat ((_ extract 31 31) $i) ((_ extract 31 31) $i) $i) (bvadd #b0000000000000000000000000000000001 (concat ((_ extract 31 31) $j) ((_ extract 31 31) $j) $j))))) (not a!1)); the site is proven
```
integers:
```
site line 8 (in gap), loop at line 6: spent its step budget
site line 10 (in gap), loop at line 6: confirmed: (not (>= (+ $i (* (- 1) $j)) 1)); the site is proven
```
integers+gg:
```
site line 8 (in gap), loop at line 6: confirmed: (let ((a!1 (not (>= (+ $j (* (- 1) $i)) 1))) (a!2 (not (>= (+ $n $j (* (- 1) $i)) 2147483648)))) (or (not (<= $n 2147483647)) (not (>= $n (- 2147483648))) (not (<= $j 2147483647)) a!1 a!2 (not (>= $i 0)))); the site is proven
site line 10 (in gap), loop at line 6: confirmed: (not (>= (+ $i (* (- 1) $j)) 1)); the site is proven
```
joint:
```
site line 8 (in gap), loop at line 6: spent its step budget
site line 10 (in gap), loop at line 6: confirmed: (not (>= (+ $i (* (- 1) $j)) 1)); the site is proven
```
joint+gg:
```
site line 8 (in gap), loop at line 6: confirmed: (let ((a!1 (not (<= (+ $i (* (- 1) $n) (* (- 1) $j)) (- 2147483648)))) (a!2 (not (<= (+ $i (* (- 1) $j)) (- 1))))) (or (not (<= $n 2147483647)) (not (>= $n (- 2147483648))) (not (<= $j 2147483647)) a!1 a!2 (not (>= $i 0)))); the site is proven
site line 10 (in gap), loop at line 6: confirmed: (not (>= (+ $i (* (- 1) $j)) 1)); the site is proven
```
bits+gg:
```
site line 8 (in gap), loop at line 6: spent its step budget
site line 10 (in gap), loop at line 6: confirmed: (let ((a!1 (= (concat ((_ extract 31 31) $i) ((_ extract 31 31) $i) $i) (bvadd #b0000000000000000000000000000000001 (concat ((_ extract 31 31) $j) ((_ extract 31 31) $j) $j))))) (not a!1)); the site is proven
```
</details>

