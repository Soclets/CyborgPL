# Parallel `for` folds of every kind (owner, 2026-09-30: c): sums and
# products of integers, bins and decis run in slices, each folding its own
# trips, the partials combined in slice order at the end. The values are
# exact, so the rounding the order may change never shows here.
%extern.C fn exit [$code: i32];
MAIN {
    let $n: i64 = 300000;
    let $bs: std::list[f64] = std::list.fill(value: 0.5, count: $n);
    let mut $bsum: f64 = 0.0;
    for $x in lend $bs {
        change $bsum = $bsum plus $x;
    }
    let $q: %d64 = 0.25;
    let $ds: std::list[%d64] = std::list.fill(value: $q, count: $n);
    let mut $dsum: %d64 = 0;
    for $x in lend $ds {
        change $dsum = $dsum plus $x;
    }
    let mut $dprod: %d64 = 2;
    for $y in lend $ds {
        change $dprod = $dprod times 1;
    }
    let mut $sign: i64 = 1;
    for $i: i64 [from: 1, to: $n, by: 1] {
        change $sign = $sign times (($i mod.floor 2) times 2 minus 1);
    }
    let $b: i32 = to.i32($bsum div.near 1000.0) otherwise 1;
    let $d: i32 = to.i32(($dsum times $dprod) div.near 1000) otherwise 1;
    no_check my.exit($b plus $d plus to.i32($sign plus 2));
}
