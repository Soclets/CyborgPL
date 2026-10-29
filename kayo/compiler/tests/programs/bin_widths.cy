# f16/f32 math rounds after every step, and a bare literal takes the
# typed side's width (owner, 2026-09-24: a): 2^24 + 1.0 at f32 is 2^24.
# With a f64 it is done at the wider (the oracle's pick). A literal in a
# comparison stays at 64 bits: a f32 0.1 is a little more than 0.1.
%extern.C proc exit [$code: i32];
MAIN:
    let $x: f32 := 16777216.0;
    var $got: i32 = 0;
    let $a := $x + 1.0;
    let $na := to.i32($a - 16777216.0) ?? 99;
    if $na == 0:
        change $got to $got + 1;
    ;if
    let $b: f32 := $x + 1.0 + 1.0;
    let $nb := to.i32($b - 16777216.0) ?? 99;
    let $w: f64 := 1.0;
    let $nw := to.i32($x + $w - 16777216.0) ?? 99;
    if $nb == 0 and $nw == 1:
        change $got to $got + 2;
    ;if
    let $c: f32 := 0.1;
    if $c > 0.1 and not ($c == 0.1):
        change $got to $got + 4;
    ;if
    let $d := $c x 10.0;
    if $d > 0.99 and $d < 1.01:
        change $got to $got + 8;
    ;if
    var $h: i32 = 0;
    check if $x > 0.0:
        change $h to to.i32($x div.near 2.0) ?? 0;
    ;if
    if $h == 8388608:
        change $got to $got + 16;
    ;if
    no_check my.exit($got);
;MAIN
