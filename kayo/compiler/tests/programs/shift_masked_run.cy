# shl.masked / shr.masked (owner, 2026-10-07): the count modulo the width (-1 is
# 63), the bits past it cut off, a signed answer read as signed. 254 + 64 + 127
# - 1 - 128 - 64 = 252, as the oracle has it.
%extern.C fn exit [$code: i32];
fn n255 [] -> u8 { return 255; }
fn one [] -> i64 { return 1; }
fn one8 [] -> i8 { return 1; }
fn low8 [] -> i8 { return -128; }
fn minus1 [] -> i32 { return -1; }
MAIN {
    let $up: u8 = n255() shl.masked 1;
    let $far: i64 = one() shl.masked 70;
    let $down: u8 = n255() shr.masked 9;
    let $neg: i64 = one() shl.masked minus1();
    let $sgn: i8 = one8() shl.masked 7;
    let $ar: i8 = low8() shr.masked 9;
    let $a: i32 = to.i32($up) plus to.i32($far) plus to.i32($down);
    let $b: i32 = to.i32($neg shr 63) plus to.i32($sgn) plus to.i32($ar);
    no_check my.exit(($a plus $b) mod.floor 256);
}
