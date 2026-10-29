# `div.trunc` and `mod.trunc` round toward zero, C's `/` and `%` (owner,
# 2026-10-02): -7 div.trunc 2 is -3 and -7 mod.trunc 2 is -1, where `div`
# and `mod` floor (-4 and 1). `mod.trunc` takes bins and brain16 too, C's
# `fmod`: -370 mod.trunc 360 is -10. The lowest i8 mod.trunc -1 is 0 and
# no check; div.trunc of it is past the type, like `div`. Each right answer
# adds its own bit to the exit code: 255 when all are right.
%extern.C proc exit [$code: i32];
MAIN:
    let $x: i32 := -7;
    let $y: i32 := 2;
    let $z: i32 := -2;
    let $lo: i8 := -128;
    let $m1: i8 := -1;
    let $u: u16 := 65535;
    let $angle: f64 := -370.0;
    let $turn: f64 := 360.0;
    let $a: f64 := 7.5;
    let $b: f64 := -2.0;
    let $h: %bf16 := -5.0;
    let $k: %bf16 := 3.0;
    var $code: i32 = 0;
    if ($x div.trunc $y) == -3:
        change $code to $code + 1;
    ;if
    if ($x mod.trunc $y) == -1:
        change $code to $code + 2;
    ;if
    if (7 div.trunc $z) == -3:
        change $code to $code + 4;
    ;if
    if (7 mod.trunc $z) == 1:
        change $code to $code + 8;
    ;if
    if (($lo mod.trunc $m1) == 0) and (($u div.trunc 10) == 6553) and (($u mod.trunc 10) == 5):
        change $code to $code + 16;
    ;if
    if ($angle mod.trunc $turn) == -10.0:
        change $code to $code + 32;
    ;if
    if (($a mod.trunc $b) == 1.5) and (($h mod.trunc $k) == -2.0):
        change $code to $code + 64;
    ;if
    # A remainder by 3 of -10 to 10, truncated: -2 to 2, the dividend's sign.
    var $sum: i32 = 0;
    for $i [from: -10, to: 10, by: 1]:
        change $sum to $sum + ($i mod.trunc 3) + ($i div.trunc 3);
    ;for
    if $sum == 0:
        change $code to $code + 128;
    ;if
    no_check my.exit($code);
;MAIN
