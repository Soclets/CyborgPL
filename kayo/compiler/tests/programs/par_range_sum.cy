# Sums over a `range` with a constant step: Precog bounds each total by its
# start plus the trips times the largest term (a bin's with room for its
# rounding), so both adds are proven and both loops run in slices.
# 300000 x 0.5 = 150000, and the i x i mod 7 terms total 599999: 600149 mod 256 = 85.
%extern.C proc exit [let $code: i32];
MAIN:
    let $n: i64 := 300000;
    var $half: f64 = 0.0;
    for $i: i64 [from: 1, to: $n, by: 1]:
        change $half to $half + 0.5;
    ;for
    var $acc: i64 = 0;
    for $i: i64 [from: 1, to: $n, by: 1]:
        change $acc to $acc + (($i x $i) mod.floor 7);
    ;for
    let $h: i32 := to.i32($half div.near 1000.0) ?? 1;
    no_check my.exit(to.i32(($acc + to.i64($h)) mod.floor 256));
;MAIN
