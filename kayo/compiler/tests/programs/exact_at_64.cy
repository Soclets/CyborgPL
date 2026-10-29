# Plain math is exact, and "Precog still uses narrow sizes where it proves
# they fit" (operators.md): 'i' is at most 'n', an `i32`'s value, so
# `'i' x 'i'` and its `mod` are 64-bit instructions, not 128-bit ones and a
# library call. Widening at every busy block had thrown `'i' </== 'n'` away.
%extern.C proc exit [let $code: i32];
%extern.C proc strlen [let $s: ptr[u8]] -> u64;
MAIN:
    let $s := "ten chars!";
    no_check let $n: i32 := to.i32(strlen(lend $s));
    var $acc: i64 = 0;
    var $i: i64 = 1;
    while $i <= to.i64($n):
        change $acc to ($acc + (($i x $i) mod.floor 7)) mod.floor 1000000007;
        change $i to $i + 1;
    ;while
    no_check my.exit(to.i32($acc));
;MAIN
