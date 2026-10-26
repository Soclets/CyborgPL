# Text in two forms (owner, 2026-09-30): `to.str.utf8` the exact value,
# `to.useful.str.utf8` the useful one, as the oracle prints them. Each
# form that reads as expected counts one: 22 in all.
%extern.C fn exit [$code: i32];

enum Size {
    small,
    big
}

%layout.C enum Mode {
    off = 0,
    lit = 7
}

MAIN {
    let mut $got: i32 = 0;
    let $x: f64 = 0.1;
    if to.stored.str.utf8($x) equal_to "0.1000000000000000055511151231257827021181583404541015625" { change $got = $got plus 1; }
    if to.useful.str.utf8($x) equal_to "0.1" { change $got = $got plus 1; }
    let $d: %d64 = 1.50;
    if to.stored.str.utf8($d) equal_to "1.50" { change $got = $got plus 1; }
    if to.useful.str.utf8($d) equal_to "1.5" { change $got = $got plus 1; }
    let $m: %d64 = 100;
    if to.useful.str.utf8($m) equal_to "100" { change $got = $got plus 1; }
    let $n: i32 = 42;
    if to.stored.str.utf8($n) equal_to "42" { change $got = $got plus 1; }
    if to.useful.str.utf8($n) equal_to "42" { change $got = $got plus 1; }
    let $ok := true;
    if to.stored.str.utf8($ok) equal_to "true" { change $got = $got plus 1; }
    if to.useful.str.utf8($ok) equal_to "true" { change $got = $got plus 1; }
    let $s := Size.big;
    if to.stored.str.utf8($s) equal_to "Size.big" { change $got = $got plus 1; }
    if to.useful.str.utf8($s) equal_to "big" { change $got = $got plus 1; }
    let $o := Mode.lit;
    if to.stored.str.utf8($o) equal_to "Mode.lit" { change $got = $got plus 1; }
    if to.useful.str.utf8($o) equal_to "lit" { change $got = $got plus 1; }
    let $g: f32 = 0.1;
    if to.useful.str.utf8($g) equal_to "0.1" { change $got = $got plus 1; }
    let $b: f64 = 100.0;
    if to.useful.str.utf8($b) equal_to "100" { change $got = $got plus 1; }
    let $c: f64 = 0.000001;
    if to.useful.str.utf8($c) equal_to "0.000001" { change $got = $got plus 1; }
    let $e: f64 = -2.25;
    if to.useful.str.utf8($e) equal_to "-2.25" { change $got = $got plus 1; }
    let $f: f64 = 123456789012345678.0;
    if to.useful.str.utf8($f) equal_to "123456789012345680" { change $got = $got plus 1; }
    let $h: f64 = 0.0;
    if to.useful.str.utf8($h) equal_to "0" { change $got = $got plus 1; }
    let $k: f64 = 10000000000000000.0;
    if to.useful.str.utf8($k) equal_to "10000000000000000" { change $got = $got plus 1; }
    let $l: f64 = 0.0001;
    if to.useful.str.utf8($l) equal_to "0.0001" { change $got = $got plus 1; }
    let $p: f64 = 1000000000000000000000000000000.0;
    if to.useful.str.utf8($p) equal_to "1000000000000000000000000000000" { change $got = $got plus 1; }
    no_check my.exit($got);
}
