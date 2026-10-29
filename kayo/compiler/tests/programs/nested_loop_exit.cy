# A loop inside a loop, then a narrowing Precog cannot settle by ranges.
# The inner loop's twelve trips used up the unrolling's levels, the outer
# loop's way round was dropped, and layer 3 saw no way out of it: every
# path to the narrowing impossible, so `unsat` "proved" it (check claude,
# seed 6313). 'v' is 1 x 100000 with one word, which no i16 holds, so
# the program stops at the narrowing.
MAIN [let $argc: i32, let $argv: ptr[ptr[u8]]]:
    let $v: i64 := to.i64($argc) x 100000;
    var $w: i32 = 8;
    while $w > 0:
        change $w to $w - 1;
        for $i: i8 [from: -13, to: -36, by: -2]:
        ;for
    ;while
    let $z: i16 := to.i16($v);
;MAIN
