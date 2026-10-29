# A comparison is of the two numbers: each side is extended by its own
# sign. Both used to take the left side's, so a `u8` of 201 beside an
# `i64` became -55 and `198 > 201` was true (check claude, seed 77).
# `strlen` is extern, so nothing here is known before it runs; 'n' is 63.
%extern.C proc exit [$code: i32];
%extern.C proc strlen [$s: ptr[u8]] -> u64;
MAIN:
    let $s := "three hundred characters are too many to type, so this is fewer";
    no_check let $n: u64 := strlen(lend $s);
    let $x: i64 := to.i64($n) + 135;
    let $u: u8 := to.u8($n) + 138;
    let $big: u64 := $n + 18446744073709551000;
    let $neg: i64 := to.i64($n) - 100;
    var $got: i32 = 0;
    if $x > $u:
        change $got to $got + 1;
    ;if
    if $u > $x:
        change $got to $got + 2;
    ;if
    if $big > $neg:
        change $got to $got + 4;
    ;if
    if $neg < $big:
        change $got to $got + 8;
    ;if
    if $neg == $u:
        change $got to $got + 16;
    ;if
    no_check my.exit($got);
;MAIN
