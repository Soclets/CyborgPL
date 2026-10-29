# deci's leftovers (owner, 2026-09-29), as examples/deci_more.cpls has
# them, checked by the program itself against the oracle's text: mod takes
# the divisor's sign, xx a whole exponent, a narrower deci fails
# too_many_digits, and a deci may be a range type. One point each: 19.
%extern.C proc exit [$code: i32];
fn same [$v: %d64, $want: str.utf8] -> i32:
    if to.stored.str.utf8($v) == $want:
        ret 1;
    ;if
    ret 0;
;fn same
fn in_unit [$v: %d64] -> i32:
    let $p := to.%d64[min: 0.0, max: 1.0]($v)-
    otherwise:
        case out_of_range:
            ret 0;
        ;case out_of_range
    ;otherwise;
    if $p >= 0.0:
        ret 1;
    ;if
    ret 2;
;fn in_unit
MAIN:
    var $got: i32 = 0;
    let $t: %d64 := 7.50;
    let $m: %d64 := -7.50;
    change $got to $got + same($t mod.floor 2, "1.50") + same($m mod.floor 2, "0.50") + same($t mod.floor -2, "-0.50");
    let $c: %d64 := -4.0;
    change $got to $got + same($c mod.floor 2, "0.0") + same($c mod.floor -2, "0.0") + same($t mod.floor 0.7, "0.50");
    let $r: %d64 := 1.05;
    change $got to $got + same($r std::pow 10, "1.628894626777441") + same($r std::pow 0, "1") + same($r std::pow -1, "0.9523809523809524");
    let $r2: %d128 := 1.05;
    if to.stored.str.utf8($r2 std::pow 10) == "1.62889462677744140625":
        change $got to $got + 1;
    ;if
    let $n: i32 := 3;
    change $got to $got + same($r std::pow $n, "1.157625") + same($c std::pow 3, "-64.000");
    let $x: %d64 := 3.141592653589793;
    let $s: %d32 := 7;
    let $y := to.%d32($x) ?? $s;
    let $z: %d64 := 2.50;
    let $zz := to.%d32($z) ?? $s;
    if to.stored.str.utf8($y) == "7" and to.stored.str.utf8($zz) == "2.50":
        change $got to $got + 1;
    ;if
    let $disc: %d64[min: 0.0, max: 1.0] := 0.15;
    let $p := to.%d64[min: 0.0, max: 1.0]("1.5") ?? $disc;
    change $got to $got + same($disc, "0.15") + same($p, "0.15");
    change $got to $got + in_unit(0.25) + in_unit(1.5);
    let $q := to.%d64[min: 0.0, max: 1.0]("0.5") ?? $disc;
    change $got to $got + same($q, "0.5");
    let $e: i64 := 5;
    let $f := to.%d32[min: 0.0, max: 10.0]($e) ?? $s;
    if to.stored.str.utf8($f) == "5":
        change $got to $got + 1;
    ;if
    let $w: f64 := 5.5;
    let $g := to.stored.%d64[min: 0.0, max: 1.0]($w) ?? $disc;
    change $got to $got + same($g, "0.15");
    no_check my.exit($got);
;MAIN
