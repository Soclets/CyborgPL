# An operator is a value as `op +` (owner, 2026-09-29): its type comes from
# where it goes — a parameter, a typed declaration: 1+2+3+4 = 10,
# 1x2x3x4 = 24, 17 mod 5 = 2, (1,2) dot (3,4) = 11 → 47 (the oracle agrees).
%extern.C proc exit [let $code: i32];
rec V:
    var x: i32; var y: i32;
;rec V
op dot [lent $a: V, lent $b: V] -> i32:
    ret $a.x x $b.x + $a.y x $b.y;
;op dot
fn fold [lent $xs: std::ls[i32], let $start: i32, let $f: fn[i32, i32] -> i32] -> i32:
    var $acc = $start;
    for $x in lend $xs:
        change $acc to $f($acc, $x);
    ;for
    ret $acc;
;fn fold
fn pairwise [let $a: V, let $b: V, let $f: fn[lent V, lent V] -> i32] -> i32:
    ret $f(lend $a, lend $b);
;fn pairwise
MAIN:
    let $xs: std::ls[i32] := [1, 2, 3, 4];
    let $sum := fold(lend $xs, 0, op +);
    let $prod := fold(lend $xs, 1, op x);
    let $m: fn[i32, i32] -> i32 := op mod.floor;
    let $r := $m(17, 5);
    let $d := pairwise(V[x = 1, y = 2], V[x = 3, y = 4], op dot);
    no_check my.exit($sum + $prod + $r + $d);
;MAIN
