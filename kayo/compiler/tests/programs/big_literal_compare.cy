# A literal past 64 bits' signed range against a narrow place: compared at
# its value. Built at 128 bits, it counted toward the widest side, and
# `2^63 </== 'v1'` became a 16-bit compare with 0 under --assert-proofs
# (check claude, seed 1086).
extern.C fn exit [$code: i32];
MAIN {
    let $v1: u16 = 0;
    let $w: i8 = -1;
    let mut $got: i32 = 0;
    if 9223372036854775808 at_most $v1 { change $got = $got plus 1; }
    if $v1 at_least 18446744073709551615 { change $got = $got plus 2; }
    if $w equal_to 18446744073709551615 { change $got = $got plus 4; }
    if $v1 < 9223372036854775808 { change $got = $got plus 8; }
    if $w < 18446744073709551615 { change $got = $got plus 16; }
    no_check my.exit($got);
}
