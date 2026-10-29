# Parallel `for` folds of every kind (owner, 2026-09-30: c): sums and
# products of integers, bins and decis run in slices, each folding its own
# trips, the partials combined in slice order at the end. The values are
# exact, so the rounding the order may change never shows here.
%extern.C proc exit [let $code: i32];
MAIN:
    let $n: i64 := 300000;
    let $bs: std::ls[f64] := std::ls.fill(value: 0.5, count: $n);
    var $bsum: f64 = 0.0;
    for $x in lend $bs:
        change $bsum to $bsum + $x;
    ;for
    let $q: %d64 := 0.25;
    let $ds: std::ls[%d64] := std::ls.fill(value: $q, count: $n);
    var $dsum: %d64 = 0;
    for $x in lend $ds:
        change $dsum to $dsum + $x;
    ;for
    var $dprod: %d64 = 2;
    for $y in lend $ds:
        change $dprod to $dprod x 1;
    ;for
    var $sign: i64 = 1;
    for $i: i64 [from: 1, to: $n, by: 1]:
        change $sign to $sign x (($i mod.floor 2) x 2 - 1);
    ;for
    let $b: i32 := to.i32($bsum div.near 1000.0) ?? 1;
    let $d: i32 := to.i32(($dsum x $dprod) div.near 1000) ?? 1;
    no_check my.exit($b + $d + to.i32($sign + 2));
;MAIN
