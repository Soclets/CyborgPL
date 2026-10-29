# Text in two forms (owner, 2026-09-30): `to.str.utf8` the exact value,
# `to.useful.str.utf8` the useful one, as the oracle prints them. Each
# form that reads as expected counts one: 22 in all.
%extern.C proc exit [$code: i32];

union.tag Size:
    small;
    big;
;union.tag Size

%layout.C enum Mode:
    off = 0;
    lit = 7;
;%layout.C enum Mode

MAIN:
    var $got: i32 = 0;
    let $x: f64 := 0.1;
    if to.stored.str.utf8($x) == "0.1000000000000000055511151231257827021181583404541015625":
        change $got to $got + 1;
    ;if
    if to.shortest.str.utf8($x) == "0.1":
        change $got to $got + 1;
    ;if
    let $d: %d64 := 1.50;
    if to.stored.str.utf8($d) == "1.50":
        change $got to $got + 1;
    ;if
    if to.shortest.str.utf8($d) == "1.5":
        change $got to $got + 1;
    ;if
    let $m: %d64 := 100;
    if to.shortest.str.utf8($m) == "100":
        change $got to $got + 1;
    ;if
    let $n: i32 := 42;
    if to.stored.str.utf8($n) == "42":
        change $got to $got + 1;
    ;if
    if to.shortest.str.utf8($n) == "42":
        change $got to $got + 1;
    ;if
    let $ok := true;
    if to.stored.str.utf8($ok) == "true":
        change $got to $got + 1;
    ;if
    if to.shortest.str.utf8($ok) == "true":
        change $got to $got + 1;
    ;if
    let $s := Size.big;
    if to.stored.str.utf8($s) == "Size.big":
        change $got to $got + 1;
    ;if
    if to.shortest.str.utf8($s) == "big":
        change $got to $got + 1;
    ;if
    let $o := Mode.lit;
    if to.stored.str.utf8($o) == "Mode.lit":
        change $got to $got + 1;
    ;if
    if to.shortest.str.utf8($o) == "lit":
        change $got to $got + 1;
    ;if
    let $g: f32 := 0.1;
    if to.shortest.str.utf8($g) == "0.1":
        change $got to $got + 1;
    ;if
    let $b: f64 := 100.0;
    if to.shortest.str.utf8($b) == "100":
        change $got to $got + 1;
    ;if
    let $c: f64 := 0.000001;
    if to.shortest.str.utf8($c) == "0.000001":
        change $got to $got + 1;
    ;if
    let $e: f64 := -2.25;
    if to.shortest.str.utf8($e) == "-2.25":
        change $got to $got + 1;
    ;if
    let $f: f64 := 123456789012345678.0;
    if to.shortest.str.utf8($f) == "123456789012345680":
        change $got to $got + 1;
    ;if
    let $h: f64 := 0.0;
    if to.shortest.str.utf8($h) == "0":
        change $got to $got + 1;
    ;if
    let $k: f64 := 10000000000000000.0;
    if to.shortest.str.utf8($k) == "10000000000000000":
        change $got to $got + 1;
    ;if
    let $l: f64 := 0.0001;
    if to.shortest.str.utf8($l) == "0.0001":
        change $got to $got + 1;
    ;if
    let $p: f64 := 1000000000000000000000000000000.0;
    if to.shortest.str.utf8($p) == "1000000000000000000000000000000":
        change $got to $got + 1;
    ;if
    no_check my.exit($got);
;MAIN
