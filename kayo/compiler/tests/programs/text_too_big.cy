# Text holding a number its type cannot hold fails `out_of_range`, however
# many digits, above or below; never a check site, never `not_a_number`
# (owner, 2026-09-24: c). `examples/text_to_number.cpls`.
%extern.C proc exit [$code: i32];
fn read [$t: str.utf8] -> i32:
    let $v := to.i32(lend $t)-
    otherwise:
        case not_a_number:
            ret -1;
        ;case not_a_number
        case out_of_range:
            ret -2;
        ;case out_of_range
    ;otherwise;
    ret $v;
;fn read
fn byte [$t: str.utf8] -> i32:
    let $v := to.u8(lend $t)-
    otherwise:
        case not_a_number:
            ret -1;
        ;case not_a_number
        case out_of_range:
            ret -2;
        ;case out_of_range
    ;otherwise;
    ret to.i32($v);
;fn byte
MAIN:
    var $got: i32 = 0;
    if read("99999999999") == -2:
        change $got to $got + 1;
    ;if
    if read("99999999999999999999999999999999999999999999") == -2:
        change $got to $got + 2;
    ;if
    if read("-99999999999999999999999999999999999999999999") == -2:
        change $got to $got + 4;
    ;if
    if byte("-1") == -2 and byte("256") == -2 and byte("255") == 255:
        change $got to $got + 8;
    ;if
    if read("2147483647") == 2147483647 and read("-2147483648") == -2147483648:
        change $got to $got + 16;
    ;if
    if read("12x") == -1 and byte("x") == -1:
        change $got to $got + 32;
    ;if
    let $b := to.i64("99999999999999999999999999999999999999999999") ?? -1;
    if $b == -1:
        change $got to $got + 64;
    ;if
    no_check my.exit($got);
;MAIN
