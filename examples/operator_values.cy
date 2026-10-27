# An operator as a value (owner, 2026-09-29): `op +`, handed wherever a function goes. Its type
# comes from where it goes. `op -` subtracts; negation is `op negate`. A custom operator's operands
# are lent, so `op dot` is a fn[lent Vec, lent Vec] -> f64.
record Vec:
    x: float.binary.64_bit;
    y: float.binary.64_bit;
end of record Vec
operator dot [$a: Vec, $b: Vec] -> float.binary.64_bit:
    return $a.x times $b.x plus $a.y times $b.y;
end of operator dot
function fold [lent $xs: std::list[integer.signed.32_bit], $start: integer.signed.32_bit, $f: function[integer.signed.32_bit, integer.signed.32_bit] -> integer.signed.32_bit] -> integer.signed.32_bit:
    variable $acc := $start;
    for $x in lend $xs:
        change $acc = $f($acc, $x);
    end of for
    return $acc;
end of function fold
function apply1 [$v: integer.signed.32_bit, $f: function[integer.signed.32_bit] -> integer.signed.32_bit] -> integer.signed.32_bit:
    return $f($v);
end of function apply1
function test2 [$a: integer.signed.32_bit, $b: integer.signed.32_bit, $f: function[integer.signed.32_bit, integer.signed.32_bit] -> boolean] -> boolean:
    return $f($a, $b);
end of function test2
function both [lent $a: Vec, lent $b: Vec, $f: function[lent Vec, lent Vec] -> float.binary.64_bit] -> float.binary.64_bit:
    return $f(lend $a, lend $b);
end of function both
!MAIN:
    let $xs: std::list[integer.signed.32_bit] = [1, 2, 3, 4];
    let $sum := fold(lend $xs, 0, operator plus);
    let $product := fold(lend $xs, 1, operator times);
    let $down := fold(lend $xs, 0, operator minus);
    let $n := apply1(5, operator negate);
    let $le := test2(3, 3, operator at_most );
    let $f: function[integer.signed.32_bit, integer.signed.32_bit] -> integer.signed.32_bit = operator modulo.floor;
    let $m := $f(negative 7, 3);
    let $u := Vec[x = 1.0, y = 2.0];
    let $v := Vec[x = 3.0, y = 4.0];
    let $d := both(lend $u, lend $v, operator dot);
    std::print.shortest($sum, " ", $product, " ", $down, " ", $n, " ", $le, " ", $m, " ", $d, newline);
!end of MAIN
