# shl.masked / shr.masked (owner, 2026-10-07): the count modulo the width (-1 is
# 63), the bits past it cut off, a signed answer read as signed. 254 + 64 + 127
# - 1 - 128 - 64 = 252, as the oracle has it.
%extern.C proc exit [$code: i32];
fn n255 [] -> u8:
    ret 255;
;fn n255
fn one [] -> i64:
    ret 1;
;fn one
fn one8 [] -> i8:
    ret 1;
;fn one8
fn low8 [] -> i8:
    ret -128;
;fn low8
fn minus1 [] -> i32:
    ret -1;
;fn minus1
MAIN:
    let $up: u8 := n255() shl.masked 1;
    let $far: i64 := one() shl.masked 70;
    let $down: u8 := n255() shr.masked 9;
    let $neg: i64 := one() shl.masked minus1();
    let $sgn: i8 := one8() shl.masked 7;
    let $ar: i8 := low8() shr.masked 9;
    let $a: i32 := to.i32($up) + to.i32($far) + to.i32($down);
    let $b: i32 := to.i32($neg shr 63) + to.i32($sgn) + to.i32($ar);
    no_check my.exit(($a + $b) mod.floor 256);
;MAIN
