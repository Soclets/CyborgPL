# `mod` on a bin or brain16 takes the divisor's sign (owner, 2026-10-02: a):
# -370 mod 360 is 350, 7.5 mod -2.0 is -0.5, a brain16 -5 mod 3 is 1; a zero
# is 0, never -0 (owner, 2026-10-07: "a"; until then it took the divisor's
# sign, "-0"): 4.0 mod -2.0 is "0"; a tiny negative over 360 that rounds up to
# 360 is 0, always below the divisor (owner: b).
%extern.C proc exit [let $code: i32];
MAIN:
    let $angle: f64 := -370.0;
    let $turn: f64 := 360.0;
    let $a: f64 := 7.5;
    let $b: f64 := -2.0;
    let $h: %bf16 := -5.0;
    let $k: %bf16 := 3.0;
    let $four: f64 := 4.0;
    let $tiny: f64 := -1.0 div.near (10.0 std::pow 20.0);
    let $m1 := to.i32($angle mod.floor $turn) ?? -1;
    let $m2 := to.i32(($a mod.floor $b) x 10.0) ?? -1;
    let $m3 := to.i32($h mod.floor $k) ?? -1;
    let $z := to.stored.str.utf8($four mod.floor $b);
    let $m4 := to.i32(std::str.bytes(lend $z));
    let $m5 := to.i32($tiny mod.floor $turn) ?? -1;
    no_check my.exit(($m1 + $m2 + $m3 x 7 + $m4 x 11 + $m5) mod.floor 256);
;MAIN
