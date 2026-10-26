# A comparison is of the two numbers: each side is extended by its own
# sign. Both used to take the left side's, so a `u8` of 201 beside an
# `i64` became -55 and `198 > 201` was true (check claude, seed 77).
# `strlen` is extern, so nothing here is known before it runs; 'n' is 63.
%extern.C fn exit [$code: i32];
%extern.C fn strlen [$s: ptr[u8]] -> u64;
MAIN {
    let $s := "three hundred characters are too many to type, so this is fewer";
    no_check let $n: u64 = strlen(lend $s);
    let $x: i64 = to.i64($n) plus 135;
    let $u: u8 = to.u8($n) plus 138;
    let $big: u64 = $n plus 18446744073709551000;
    let $neg: i64 = to.i64($n) minus 100;
    let mut $got: i32 = 0;
    if $x > $u { change $got = $got plus 1; }
    if $u > $x { change $got = $got plus 2; }
    if $big > $neg { change $got = $got plus 4; }
    if $neg < $big { change $got = $got plus 8; }
    if $neg equal_to $u { change $got = $got plus 16; }
    no_check my.exit($got);
}
