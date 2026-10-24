# A Kit that uses another (owner, 2026-09-29), examples/uses_a_kit as an
# exit: a circle of radius 2 is 12.56636, so its whole part is 12; square 4
# is 16; pi's whole part is 3. 12 + 16 + 3 = 31.
extern.C fn exit [$code: i32];
MAIN {
    let $c: areas::Circle = areas::Circle[radius = 2.0];
    let $a := areas::circle(lend $c);
    let $whole: i32 = to.i32(floor_of($a)) otherwise 0;
    let $p: i32 = to.i32(floor_of(areas::$pi)) otherwise 0;
    no_check my.exit($whole plus areas::shapes.square(4) plus $p);
}
fn floor_of [$x: f64] -> f64 {
    let mut $n: f64 = 0.0;
    while $n plus 1.0 at_most $x { change $n = $n plus 1.0; }
    return $n;
}
