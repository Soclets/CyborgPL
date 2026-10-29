# "Bare numbers alone are worked out exactly at compile time" (operators.md),
# `/` and `mod` too, with no 64-bit window in between: only where the answer
# lands is checked. (2^64 - 1 + 1) mod 1 is 0; it was refused, in a loop and
# out of one (check claude, attack a: 2026-09-24-bare-sum-past-64-in-loop).
%extern.C proc exit [let $code: i32];
MAIN:
    var $v: u64 = 9;
    var $w: u16 = 2;
    loop:
        if $w == 0:
            break;
        ;if
        change $w to $w - 1;
        change $v to (18446744073709551615 + 1) mod.floor 1;
    ;loop
    let $u: u64 := (18446744073709551615 x 4) div.floor 2 - 18446744073709551615 - 1;
    let $t: i8 := ((1000 x 1000) mod.floor 7) - (200 + 100) div.floor 3;
    no_check my.exit(to.i32($v) + to.i32($u mod.floor 256) + to.i32($t) + 100);
;MAIN
