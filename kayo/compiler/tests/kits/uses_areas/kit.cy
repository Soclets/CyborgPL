# A Kit that uses another (owner, 2026-09-29), examples/uses_a_kit as an
# exit: a circle of radius 2 is 12.56636, so its whole part is 12; square 4
# is 16; pi's whole part is 3. 12 + 16 + 3 = 31.
%extern.C proc exit [$code: i32];
MAIN:
    let $c: areas::Circle := areas::Circle[radius = 2.0];
    let $a := areas::circle(lend $c);
    let $whole: i32 := to.i32(floor_of($a)) ?? 0;
    let $p: i32 := to.i32(floor_of(areas::$pi)) ?? 0;
    no_check my.exit($whole + areas::shapes.square(4) + $p);
;MAIN
fn floor_of [$x: f64] -> f64:
    var $n: f64 = 0.0;
    while $n + 1.0 <= $x:
        change $n to $n + 1.0;
    ;while
    ret $n;
;fn floor_of
