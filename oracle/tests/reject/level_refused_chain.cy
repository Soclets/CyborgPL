op ~= [lent $a: f64, lent $b: f64] -> bool:
    ret $a == $b;
;op ~=
precedence.low_first:
    level 1 [==, ~=] refused;
;precedence.low_first
MAIN:
    let $x: f64 := 1.0;
    let $k := $x ~= $x ~= $x;
;MAIN
