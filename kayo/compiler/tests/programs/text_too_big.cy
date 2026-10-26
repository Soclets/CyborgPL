# Text holding a number its type cannot hold fails `out_of_range`, however
# many digits, above or below; never a check site, never `not_a_number`
# (owner, 2026-09-24: c). `examples/text_to_number.cpls`.
%extern.C fn exit [$code: i32];
fn read [$t: str.utf8] -> i32 {
    let $v := to.i32($t) otherwise {
        not_a_number { return -1; }
        out_of_range { return -2; }
    };
    return $v;
}
fn byte [$t: str.utf8] -> i32 {
    let $v := to.u8($t) otherwise {
        not_a_number { return -1; }
        out_of_range { return -2; }
    };
    return to.i32($v);
}
MAIN {
    let mut $got: i32 = 0;
    if read("99999999999") equal_to -2 { change $got = $got plus 1; }
    if read("99999999999999999999999999999999999999999999") equal_to -2 { change $got = $got plus 2; }
    if read("-99999999999999999999999999999999999999999999") equal_to -2 { change $got = $got plus 4; }
    if byte("-1") equal_to -2 and byte("256") equal_to -2 and byte("255") equal_to 255 { change $got = $got plus 8; }
    if read("2147483647") equal_to 2147483647 and read("-2147483648") equal_to -2147483648 { change $got = $got plus 16; }
    if read("12x") equal_to -1 and byte("x") equal_to -1 { change $got = $got plus 32; }
    let $b := to.i64("99999999999999999999999999999999999999999999") otherwise -1;
    if $b equal_to -1 { change $got = $got plus 64; }
    no_check my.exit($got);
}
