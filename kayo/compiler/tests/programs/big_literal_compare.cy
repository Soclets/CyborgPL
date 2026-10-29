# A literal past 64 bits' signed range against a narrow place: compared at
# its value. Built at 128 bits, it counted toward the widest side, and
# `2^63 </== 'v1'` became a 16-bit compare with 0 under --assert-proofs
# (check claude, seed 1086).
%extern.C proc exit [$code: i32];
MAIN:
    let $v1: u16 := 0;
    let $w: i8 := -1;
    var $got: i32 = 0;
    if 9223372036854775808 <= $v1:
        change $got to $got + 1;
    ;if
    if $v1 >= 18446744073709551615:
        change $got to $got + 2;
    ;if
    if $w == 18446744073709551615:
        change $got to $got + 4;
    ;if
    if $v1 < 9223372036854775808:
        change $got to $got + 8;
    ;if
    if $w < 18446744073709551615:
        change $got to $got + 16;
    ;if
    no_check my.exit($got);
;MAIN
