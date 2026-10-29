# examples/operator_values.cpls as an exit code (owner, 2026-09-29): 10 + 24
# + (-10) + (-5) + 2 + 11, and 100 when `op </==` said 3 </== 3 → 132.
%extern.C proc exit [$code: i32];
rec Vec:
    var x: f64; var y: f64;
;rec Vec
op dot [lent $a: Vec, lent $b: Vec] -> f64:
    ret $a.x x $b.x + $a.y x $b.y;
;op dot
fn fold [lent $xs: std::ls[i32], $start: i32, $f: fn[i32, i32] -> i32] -> i32:
    var $acc = $start;
    for $x in lend $xs:
        change $acc to $f($acc, $x);
    ;for
    ret $acc;
;fn fold
fn apply1 [$v: i32, $f: fn[i32] -> i32] -> i32:
    ret $f[pos: $v];
;fn apply1
fn test2 [$a: i32, $b: i32, $f: fn[i32, i32] -> bool] -> bool:
    ret $f($a, $b);
;fn test2
fn both [lent $a: Vec, lent $b: Vec, $f: fn[lent Vec, lent Vec] -> f64] -> f64:
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
    let $dn := to.i32($d) ?? 0;
    var $bonus: i32 = 0;
    if $le:
        change $bonus to 100;
    ;if
    no_check my.exit($sum + $product + $down + $n + $m + $dn + $bonus);
;MAIN
