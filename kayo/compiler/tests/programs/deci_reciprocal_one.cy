# A deci whose power's coefficient is 1, to a negative power, is exact at
# the exponent one over it gives: 1 pow -1 is 1, not 1.0 (check claude,
# 2026-10-15, corpus ffe48e05). Each text as the oracle prints it counts
# one: 8 in all.
%extern.C proc exit [$code: i32];

MAIN:
    var $got: i32 = 0;
    let $a: %d32 := to.%d32(1) std::pow (-1);
    if to.stored.str.utf8($a) == "1":
        change $got to $got + 1;
    ;if
    let $b: %d64 := to.%d64(1) std::pow (-2);
    if to.stored.str.utf8($b) == "1":
        change $got to $got + 1;
    ;if
    let $c: %d32 := to.%d32(-1) std::pow (-1);
    if to.stored.str.utf8($c) == "-1":
        change $got to $got + 1;
    ;if
    let $h1: %d32 := 0.01;
    let $e: %d32 := $h1 std::pow (-1);
    if to.stored.str.utf8($e) == "100":
        change $got to $got + 1;
    ;if
    let $g: %d64 := to.%d64(10) std::pow (-3);
    if to.stored.str.utf8($g) == "0.001":
        change $got to $got + 1;
    ;if
    let $h: %d64 := to.%d64(4) std::pow (-1);
    if to.stored.str.utf8($h) == "0.25":
        change $got to $got + 1;
    ;if
    let $h3: %d64 := -0.1;
    let $i: %d64 := $h3 std::pow (-3);
    if to.stored.str.utf8($i) == "-1000":
        change $got to $got + 1;
    ;if
    let $j: %d128 := to.%d128(-1) std::pow (-7);
    if to.stored.str.utf8($j) == "-1":
        change $got to $got + 1;
    ;if
    no_check my.exit($got);
;MAIN
