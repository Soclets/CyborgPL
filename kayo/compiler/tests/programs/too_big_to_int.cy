# A whole bin or deci too big for the integer fails `out_of_range`, beside
# `not_whole` (owner, 2026-09-30: a): one `otherwise` handles both, as the
# oracle's examples/too_big_to_int.cpls prints "-1 -2 42 99". 42, then 100
# for -1, 50 for -2 and 20 for 99: 212.
%extern.C proc exit [let $code: i32];
MAIN:
    let $b: f64 := 10000000000.0;
    let $d: %d128 := 99999999999999999999.0;
    let $small: %d64 := 42.0;
    let $x := to.i32($b) ?? -1;
    let $y := to.i64($d x 1000) ?? -2;
    let $z := to.i64($small) ?? -3;
    let $w := to.i32($d) ?? 99;
    var $code: i64 = $z;
    if $x == -1:
        change $code to $code + 100;
    ;if
    if $y == -2:
        change $code to $code + 50;
    ;if
    if $w == 99:
        change $code to $code + 20;
    ;if
    no_check my.exit(to.i32($code));
;MAIN
