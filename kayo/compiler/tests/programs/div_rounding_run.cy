# Every division says how it rounds (owner, 2026-10-02), worked out at run
# time: the values pass through C, so nothing is folded. `floor` down,
# `ceil` up, `trunc` toward zero, `near` to the nearest with a tie to even;
# `mod.X` is what `div.X` leaves. On a bin or deci `div.near` is ordinary
# division and the others give a whole quotient. The smallest i8 by -1
# is 128, exact where it lands. Each right group adds its own bit: 255.
%extern.C proc exit [let $code: i32];
%extern.C proc strlen [let $s: ptr[u8]] -> u64;
MAIN:
    let $s := "x";
    no_check let $n: u64 := strlen(lend $s);
    let $z: i32 := to.i32($n) - 1;
    let $a: i32 := -7 + $z;
    let $b: i32 := 2 + $z;
    let $c: i32 := 5 + $z;
    var $code: i32 = 0;
    if (($a div.floor $b) == -4) and (($a div.ceil $b) == -3) and (($a div.trunc $b) == -3) and (($a div.near $b) == -4):
        change $code to $code + 1;
    ;if
    if (($a mod.floor $b) == 1) and (($a mod.ceil $b) == -1) and (($a mod.trunc $b) == -1) and (($a mod.near $b) == 1):
        change $code to $code + 2;
    ;if
    if (($c div.near $b) == 2) and (($c mod.near $b) == 1) and ((7 div.near $b) == 4):
        change $code to $code + 4;
    ;if
    let $u: u32 := to.u32(7 + $z);
    let $v: u32 := to.u32(2 + $z);
    if (($u div.ceil $v) == 4) and (($u div.near $v) == 4) and (($u div.floor $v) == 3):
        change $code to $code + 8;
    ;if
    let $lo: i8 := to.i8(-128 + $z);
    let $m1: i8 := to.i8(-1 + $z);
    let $big: i16 := $lo div.trunc $m1;
    if ($big == 128) and (($lo mod.near $m1) == 0):
        change $code to $code + 16;
    ;if
    let $one: f64 := to.f64(1 + $z);
    let $x: f64 := 7.5 x $one;
    let $y: f64 := 2.0 x $one;
    if (($x div.near $y) == 3.75) and (($x div.floor $y) == 3.0) and (($x mod.floor $y) == 1.5) and (($x div.ceil $y) == 4.0) and (($x mod.ceil $y) == -0.5):
        change $code to $code + 32;
    ;if
    let $h: %bf16 := to.%bf16(-5 + $z);
    let $k: %bf16 := to.%bf16(3 + $z);
    if (($h div.floor $k) == -2.0) and (($h mod.floor $k) == 1.0) and (($h div.trunc $k) == -1.0) and (($h mod.trunc $k) == -2.0):
        change $code to $code + 64;
    ;if
    let $p: %d64 := -7.5 + to.%d64($z);
    let $q: %d64 := 2.0 + to.%d64($z);
    if (($p div.near $q) == -3.75) and (($p div.floor $q) == -4) and (($p mod.floor $q) == 0.5) and (($p div.trunc $q) == -3) and (($p mod.trunc $q) == -1.5) and (($p mod.ceil $q) == -1.5):
        change $code to $code + 128;
    ;if
    no_check my.exit($code);
;MAIN
