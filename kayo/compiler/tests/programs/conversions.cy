# Conversions that can fail: text into a whole number (not_a_number), a bin
# into one (not_whole, or out_of_range when too big: owner, 2026-09-30),
# and anything into a range type (out_of_range).
extern.C fn exit [$code: i32];
fn read [$t: str.utf8] -> i32 {
    let $v := to.i32($t) otherwise {
        not_a_number { return -1; }
        out_of_range { return -3; }
    };
    return $v;
}
fn share [$t: str.utf8] -> i32 {
    let $v := to.i32[min: 0, max: 100]($t) otherwise {
        not_a_number { return -1; }
        out_of_range { return -2; }
    };
    return $v;
}
fn whole [$x: f64] -> i32 {
    let $v := to.i32($x) otherwise {
        not_whole { return -1; }
        out_of_range { return -4; }
    };
    return $v;
}
MAIN {
    let mut $got: i32 = 0;
    if read("42") equal_to 42 { change $got = $got plus 1; }
    let $b := to.i64("-7") otherwise -1;
    if $b equal_to -7 { change $got = $got plus 2; }
    if read("4x2") equal_to -1 and read(str.empty) equal_to -1 and read("-") equal_to -1 { change $got = $got plus 4; }
    if share("150") equal_to -2 and share("-1") equal_to -2 and share("x") equal_to -1 { change $got = $got plus 8; }
    if share("+99") equal_to 99 and share("0") equal_to 0 { change $got = $got plus 16; }
    if whole(2.0) equal_to 2 and whole(2.7) equal_to -1 and whole(-3.0) equal_to -3 and whole(10000000000.0) equal_to -4 { change $got = $got plus 32; }
    let $n: i64 = 150;
    let $h := to.i32[min: 0, max: 100]($n) otherwise 7;
    if $h equal_to 7 { change $got = $got plus 64; }
    let $i := to.u8(" 1") otherwise 3;
    if $i equal_to 3 { change $got = $got plus 128; }
    no_check my.exit($got mod.floor 256);
}
