# A division that wraps like the hardware (owner, 2026-10-03): `div.X.wrap`
# gives the true quotient wrapped at the type's width, so the smallest i8
# by -1 is -128 under all four roundings, and an i128's too; an ordinary
# quotient is the true one. Run-time values, through C: 1 + 2 + 4 = 7.
%extern.C proc exit [let $code: i32];
%extern.C proc strlen [let $s: ptr[u8]] -> u64;
MAIN:
    let $s := "x";
    no_check let $n: u64 := strlen(lend $s);
    let $a: i8 := to.i8(-129 + to.i64($n));
    let $m1: i8 := to.i8(-2 + to.i64($n));
    let $m: %i128 := -170141183460469231731687303715884105728 + to.%i128($n) - 1;
    var $code: i32 = 0;
    if (($a div.trunc.wrap $m1) == -128) and (($a div.floor.wrap $m1) == -128) and (($a div.ceil.wrap $m1) == -128) and (($a div.near.wrap $m1) == -128):
        change $code to $code + 1;
    ;if
    let $two: %i128 := to.%i128($n) + 1;
    if (($m div.trunc.wrap to.%i128($m1)) == $m) and (($m div.near.wrap $two) == -85070591730234615865843651857942052864):
        change $code to $code + 2;
    ;if
    let $u: u8 := to.u8($n) + 6;
    if (($u div.ceil.wrap 2) == 4) and (($a div.floor.wrap 3) == -43):
        change $code to $code + 4;
    ;if
    no_check my.exit($code);
;MAIN
