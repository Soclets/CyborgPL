# Testing — `test fn` and `must` (owner, 2026-09-29)

```
fn add [$a: i32, $b: i32] -> i32 { return $a plus $b; }

test fn adds_up [] {
    must add(2, 3) equal_to 5;                        # what must hold
    let $n := to.i32("42") otherwise 0;
    must $n equal_to 42;
    must.fail to.i32["abc"];                  # it must fail
    must.fail.not_a_number to.i32("abc");     # …with exactly this reason
    let $xs: std::list[i32] = [1, 2, 3];
    must.stop $xs[pos: 99];                      # a check site must fire
}
```

`synapse test` (kayo runs and tests on its own, 2026-09-29) runs every `test fn`
in the order written, each on its own, and reports each one, then a
count; it exits 0 when all pass.

- **A test is a function marked `test`** (owner: a): `test fn name [] { … }`,
  like `pub` in front. It takes no parameters, returns nothing and cannot
  fail; it is never `pub`, nothing calls it, and a normal build leaves it
  out. Proposed alternatives: every function in a `module tests { }`; a
  `#test` file mode.
- **`must` states what must hold** (owner: "the fam"), and only inside a
  `test fn` (in a program, the prover and `check` do this job, at compile
  time). A comparison that does not hold shows both sides: "the left side
  is 6, and it must be `==` 5".
- **`must.fail`** expects a failure — Cyborg's named `fail` / `otherwise`
  kind — and needs no `otherwise`; **`must.fail.<reason>`** expects that
  reason. **`must.stop`** expects a check site to fire (out of range,
  overflow, division by zero), which in a test run is a run-time check.
  Cyborg has exactly those two ways of going wrong.

How it was reached: compiler claude asked what a test is. Proposed: (a) a
folder of programs with expected output, needing no new syntax; (b) test
functions with assertions; (c) both. Design claude recommended c; the owner
asked about `assert_eq`, then chose b. For the assertion: `expect` was
proposed (the owner: it "sounds like an optimization hint", as C's
`__builtin_expect` is), then `assert`, `must`, `ensure`; the owner offered
`must_be` / `must-be` (`-` is an operator; a two-word keyword is written
with a dot, `must.be`, which reads well only between two values), then
`must.` — a family, narrowed with a dot as `and.then` is.

## Oracle

`oracle --test file-or-Kit` runs the tests. `examples/testing.cy` (two
of its tests fail on purpose; `.test-out` holds the report); reject tests
must_outside_test, test_fn_params, test_fn_called, must_fail_not_a_call.
A failed comparison's sides are worked out a second time for the message
(the oracle's reading).
