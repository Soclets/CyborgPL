# Finding 2026-09-29 (check run, 163 of 300 programs from seed 4201): CCA refuses a fill whose value is
# arithmetic: "expected a value of the declared type (simd[i32, count: 4]), found a simd[integer, count: 1]".
# simd.fill['x'] and simd.fill[to.i32['y']] build; 'x' + 1, ('x' mod 7) - 3 and ... xx 2 do not.
# The oracle runs it.
MAIN {
    let 'x': i32 = 5;
    let 'a': simd[i32, count: 4] = simd.fill(value: 'x' plus 1);
}
