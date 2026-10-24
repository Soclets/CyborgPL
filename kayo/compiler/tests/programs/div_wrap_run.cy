# A division that wraps like the hardware (owner, 2026-10-03): `div.X.wrap`
# gives the true quotient wrapped at the type's width, so the smallest i8
# by -1 is -128 under all four roundings, and an i128's too; an ordinary
# quotient is the true one. Run-time values, through C: 1 + 2 + 4 = 7.
extern.C fn exit [$code: i32];
extern.C fn strlen [$s: ptr[u8]] -> u64;
MAIN {
    let $s := "x";
    no_check let $n: u64 = strlen(lend $s);
    let $a: i8 = to.i8(-129 plus to.i64($n));
    let $m1: i8 = to.i8(-2 plus to.i64($n));
    let $m: i128 = -170141183460469231731687303715884105728 plus to.i128($n) minus 1;
    let mut $code: i32 = 0;
    if (($a div.trunc.wrap $m1) equal_to -128) and (($a div.floor.wrap $m1) equal_to -128) and (($a div.ceil.wrap $m1) equal_to -128) and (($a div.near.wrap $m1) equal_to -128) { change $code = $code plus 1; }
    let $two: i128 = to.i128($n) plus 1;
    if (($m div.trunc.wrap to.i128($m1)) equal_to $m) and (($m div.near.wrap $two) equal_to -85070591730234615865843651857942052864) { change $code = $code plus 2; }
    let $u: u8 = to.u8($n) plus 6;
    if (($u div.ceil.wrap 2) equal_to 4) and (($a div.floor.wrap 3) equal_to -43) { change $code = $code plus 4; }
    no_check my.exit($code);
}
