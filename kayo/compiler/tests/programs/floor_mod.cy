# `mod` takes the divisor's sign (operators.md): 1 mod -3 is -2.
%extern.C proc exit [$code: i32];
%extern.C proc labs [$v: i64] -> i64;
MAIN:
    no_check let $x: i64 := labs(1);
    let $a: i64 := 1 mod.floor (-3);
    let $b: i64 := 7 mod.floor (-3);
    let $d: i64 := $x mod.floor (-3);
    let $e: i64 := 1 mod.floor (-2982098350919537422);
    let $g: i32 := 1 mod.floor (-2000000000);
    let $c: i64 := -7 mod.floor 3;
    let $ok: i32 := 0;
    if $a == -2 and $b == -2 and $d == -2 and $e == -2982098350919537421 and $g == -1999999999 and $c == 2:
        no_check my.exit(42);
    ;if
    no_check my.exit(1);
;MAIN
