# A loop inside a loop, then a narrowing Precog cannot settle by ranges.
# The inner loop's twelve trips used up the unrolling's levels, the outer
# loop's way round was dropped, and layer 3 saw no way out of it: every
# path to the narrowing impossible, so `unsat` "proved" it (check claude,
# seed 6313). 'v' is 1 x 100000 with one word, which no i16 holds, so
# the program stops at the narrowing.
MAIN [$argc: i32, $argv: ptr[ptr[u8]]] {
    let $v: i64 = to.i64($argc) times 100000;
    let mut $w: i32 = 8;
    while $w > 0 {
        change $w = $w minus 1;
        for $i: i8 [from: -13, to: -36, by: -2] {
        }
    }
    let $z: i16 = to.i16($v);
}
