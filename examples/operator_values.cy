# An operator as a value (owner, 2026-09-29): `op +`, handed wherever a function goes. Its type
# comes from where it goes. `op -` subtracts; negation is `op negate`. A custom operator's operands
# are lent, so `op dot` is a fn[lent Vec, lent Vec] -> f64.
rec Vec:
    var x: f64;
    var y: f64;
;rec Vec
op dot [lent $a: Vec, lent $b: Vec] -> f64:
    ret $a.x x $b.x + $a.y x $b.y;
;op dot
fn fold [lent $xs: std::ls[i32], let $start: i32, let $f: fn[i32, i32] -> i32] -> i32:
    var $acc = $start;
    for $x in lend $xs:
        change $acc to $f($acc, $x);
    ;for
    ret $acc;
;fn fold
fn apply1 [let $v: i32, let $f: fn[i32] -> i32] -> i32:
    ret $f($v);
;fn apply1
fn test2 [let $a: i32, let $b: i32, let $f: fn[i32, i32] -> bool] -> bool:
    ret $f($a, $b);
;fn test2
fn both [lent $a: Vec, lent $b: Vec, let $f: fn[lent Vec, lent Vec] -> f64] -> f64:
    ret $f(lend $a, lend $b);
;fn both
MAIN:
    let $xs: std::ls[i32] := [1, 2, 3, 4];
    let $sum := fold(lend $xs, 0, op +);
    let $product := fold(lend $xs, 1, op x);
    let $down := fold(lend $xs, 0, op -);
    let $n := apply1(5, op neg);
    let $le := test2(3, 3, op <= );
    let $f: fn[i32, i32] -> i32 := op mod.floor;
    let $m := $f(-7, 3);
    let $u := Vec[x = 1.0, y = 2.0];
    let $v := Vec[x = 3.0, y = 4.0];
    let $d := both(lend $u, lend $v, op dot);
    std::println($sum, " ", $product, " ", $down, " ", $n, " ", $le, " ", $m, " ", $d);
;MAIN
