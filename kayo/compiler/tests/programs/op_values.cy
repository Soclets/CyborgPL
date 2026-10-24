# An operator is a value as `op +` (owner, 2026-09-29): its type comes from
# where it goes — a parameter, a typed declaration: 1+2+3+4 = 10,
# 1x2x3x4 = 24, 17 mod 5 = 2, (1,2) dot (3,4) = 11 → 47 (the oracle agrees).
extern.C fn exit [$code: i32];
struct V { x: i32, y: i32 }
op dot [$a: V, $b: V] -> i32 { return $a.x times $b.x plus $a.y times $b.y; }
fn fold [lent $xs: std::list[i32], $start: i32, $f: fn[i32, i32] -> i32] -> i32 {
    let mut $acc := $start;
    for $x in lend $xs { change $acc = $f($acc, $x); }
    return $acc;
}
fn pairwise [$a: V, $b: V, $f: fn[lent V, lent V] -> i32] -> i32 { return $f(lend $a, lend $b); }
MAIN {
    let $xs: std::list[i32] = [1, 2, 3, 4];
    let $sum := fold(lend $xs, 0, op plus);
    let $prod := fold(lend $xs, 1, op times);
    let $m: fn[i32, i32] -> i32 = op mod.floor;
    let $r := $m(17, 5);
    let $d := pairwise(V[x = 1, y = 2], V[x = 3, y = 4], op dot);
    no_check my.exit($sum plus $prod plus $r plus $d);
}
