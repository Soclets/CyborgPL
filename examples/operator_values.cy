# An operator as a value (owner, 2026-09-29): `op +`, handed wherever a function goes. Its type
# comes from where it goes. `op -` subtracts; negation is `op negate`. A custom operator's operands
# are lent, so `op dot` is a fn[lent Vec, lent Vec] -> f64.
rec Vec:
    var x: float.bin.64;
    var y: float.bin.64;
end of rec Vec
op dot [$a: Vec, $b: Vec] -> float.bin.64:
    ret $a.x times $b.x plus $a.y times $b.y;
end of op dot
fn fold [lent $xs: std::ls[int.sign.32], $start: int.sign.32, $f: fn[int.sign.32, int.sign.32] -> int.sign.32] -> int.sign.32:
    var $acc := $start;
    for $x in lend $xs:
        change $acc = $f($acc, $x);
    end of for
    ret $acc;
end of fn fold
fn apply1 [$v: int.sign.32, $f: fn[int.sign.32] -> int.sign.32] -> int.sign.32:
    ret $f($v);
end of fn apply1
fn test2 [$a: int.sign.32, $b: int.sign.32, $f: fn[int.sign.32, int.sign.32] -> bool] -> bool:
    ret $f($a, $b);
end of fn test2
fn both [lent $a: Vec, lent $b: Vec, $f: fn[lent Vec, lent Vec] -> float.bin.64] -> float.bin.64:
    ret $f(lend $a, lend $b);
end of fn both
!MAIN:
    let $xs: std::ls[int.sign.32] = [1, 2, 3, 4];
    let $sum := fold(lend $xs, 0, op plus);
    let $product := fold(lend $xs, 1, op times);
    let $down := fold(lend $xs, 0, op minus);
    let $n := apply1(5, op neg);
    let $le := test2(3, 3, op at_most );
    let $f: fn[int.sign.32, int.sign.32] -> int.sign.32 = op mod.floor;
    let $m := $f(neg 7, 3);
    let $u := Vec[x = 1.0, y = 2.0];
    let $v := Vec[x = 3.0, y = 4.0];
    let $d := both(lend $u, lend $v, op dot);
    std::print.shortest($sum, " ", $product, " ", $down, " ", $n, " ", $le, " ", $m, " ", $d, newln);
!end of MAIN
