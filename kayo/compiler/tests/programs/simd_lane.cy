# One lane of a vector (simd.md: `'v'[1]`, 1-based; check claude found it
# gave garbage): the lane read had no shape in the generated code, the
# assignment was dropped without a word, and the variable was read unset.
# Lanes of each width, a position worked out at run time, and a load whose
# lanes differ, to see the order: 4 + 4 + (200 - 190) + (-5 + 10) +
# 2.5 x 2 + 30 = 58.
%extern.C proc exit [$code: i32];
MAIN:
    let $a: simd[i32, count: 4] := simd.fill(value: 4);
    let $b: simd[u8, count: 16] := simd.fill(value: 200);
    let $c: simd[i64, count: 2] := simd.fill(value: -5);
    let $w: simd[f64, count: 2] := simd.fill(value: 2.5);
    let $xs: std::ls[i32] := [10, 20, 30, 40];
    let $v: simd[i32, count: 4] := simd.load(lend $xs, pos: 1);
    let $k: i64 := 3;
    let $x1: i32 := $a[pos: 1];
    let $x2: u8 := $b[pos: 16];
    let $x3: i64 := $c[pos: 2];
    let $x4: i32 := $a[pos: $k - 1];
    let $x5: f64 := $w[pos: 2];
    let $x6: i32 := $v[pos: $k];
    let $half: i32 := to.i32($x5 x 2.0) ?? 0;
    no_check my.exit($x1 + $x4 + (to.i32($x2) - 190) + to.i32($x3 + 10) + $half + $x6);
;MAIN
