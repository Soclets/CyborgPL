# A kit gives its operators a precedence (owner, 2026-10-03): `precedence.low_first` counts level 1
# as the loosest (good practice), each level says how a chain reads, and a language operator in a
# level pins it where that operator is. std's `pow` binds above a negative, right pair first.
rec Vec:
    var x: f64;
    var y: f64;
;rec Vec

precedence.low_first:
    level 1 [==, ~=] refused;
    level 2 [+] left_first;
    level 3 [x, dot] left_first;
    level 4 [scale] left_first;
;precedence.low_first

op dot [lent $a: Vec, lent $b: Vec] -> f64:
    ret ($a.x x $b.x) + ($a.y x $b.y);
;op dot

op scale [lent $a: Vec, lent $k: f64] -> Vec:
    ret Vec[x = $a.x x $k, y = $a.y x $k];
;op scale

op ~= [lent $a: f64, lent $b: f64] -> bool:
    ret ($a - $b) < 0.000001 and ($b - $a) < 0.000001;
;op ~=

MAIN:
    let $u := Vec[x = 1.0, y = 2.0];
    let $v := Vec[x = 3.0, y = 4.0];
    let $d := $u dot $v + 1.0;          # ('u' dot 'v') plus 1.0
    let $w := $u scale 2.0 dot $v;         # ('u' scale 2.0) dot 'v'
    let $ok := $d ~= 12.0;
    std::print($d, " ", $w, " ", $ok, newln);
    std::print(2 ^ 3 + 1, " ", 2 ^ 3 ^ 2, " ", -2 ^ 2, " ", 2 x 3 ^ 2, newln);
;MAIN
