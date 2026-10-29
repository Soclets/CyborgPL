# A check that fires in a slice stops the program as in order (parallel
# `for`, owner 2026-09-30): 'i' into an i8 fails at 128, whichever slice
# gets there first.
%extern.C proc exit [let $code: i32];
MAIN:
    let $n: i64 := 200000;
    var $xs: std::ls[i8] = std::ls.fill(value: 0, count: $n);
    for $i: i64 [from: 1, to: $n, by: 1]:
        change $xs[pos: $i] to to.i8($i);
    ;for
    no_check my.exit(to.i32($xs[pos: 1]));
;MAIN
