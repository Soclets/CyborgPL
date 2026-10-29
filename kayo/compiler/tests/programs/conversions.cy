# Conversions that can fail: text into a whole number (not_a_number), a bin
# into one (not_whole, or out_of_range when too big: owner, 2026-09-30),
# and anything into a range type (out_of_range).
%extern.C proc exit [let $code: i32];
fn read [let $t: str.utf8] -> i32:
    let $v := to.i32(lend $t)-
    otherwise:
        case not_a_number:
            ret -1;
        ;case not_a_number
        case out_of_range:
            ret -3;
        ;case out_of_range
    ;otherwise;
    ret $v;
;fn read
fn share [let $t: str.utf8] -> i32:
    let $v := to.i32[min: 0, max: 100](lend $t)-
    otherwise:
        case not_a_number:
            ret -1;
        ;case not_a_number
        case out_of_range:
            ret -2;
        ;case out_of_range
    ;otherwise;
    ret $v;
;fn share
fn whole [let $x: f64] -> i32:
    let $v := to.i32($x)-
    otherwise:
        case not_whole:
            ret -1;
        ;case not_whole
        case out_of_range:
            ret -4;
        ;case out_of_range
    ;otherwise;
    ret $v;
;fn whole
MAIN:
    var $got: i32 = 0;
    if read("42") == 42:
        change $got to $got + 1;
    ;if
    let $b := to.i64("-7") ?? -1;
    if $b == -7:
        change $got to $got + 2;
    ;if
    if read("4x2") == -1 and read(str.empty) == -1 and read("-") == -1:
        change $got to $got + 4;
    ;if
    if share("150") == -2 and share("-1") == -2 and share("x") == -1:
        change $got to $got + 8;
    ;if
    if share("+99") == 99 and share("0") == 0:
        change $got to $got + 16;
    ;if
    if whole(2.0) == 2 and whole(2.7) == -1 and whole(-3.0) == -3 and whole(10000000000.0) == -4:
        change $got to $got + 32;
    ;if
    let $n: i64 := 150;
    let $h := to.i32[min: 0, max: 100]($n) ?? 7;
    if $h == 7:
        change $got to $got + 64;
    ;if
    let $i := to.u8(" 1") ?? 3;
    if $i == 3:
        change $got to $got + 128;
    ;if
    no_check my.exit($got mod.floor 256);
;MAIN
