# In a list literal, a bare number takes its family from the other items
# (owner, 2026-09-30: a), whichever comes first: [1, 2.5] is two bins, and
# [1, 'q'] with 'q' a deci is two decis. The oracle (9b29d6f):
# (1 + 5) + (1 + 0.5) x 2 + (1 + 5 - 3 + 10) + (0.1 + 1) x 10 = 6 + 3 + 13 + 11 = 33.
%extern.C fn exit [$code: i32];
MAIN {
    let $xs := [1, 2.5];
    let $q: %d64 = 0.5;
    let $ds := [1, $q];
    let $ys: std::list[f64] = [1, 2.5, -3];
    let $es: std::list[%d64] = [0.1, 1];
    let $a: i32 = to.i32($xs[pos: 1] plus ($xs[pos: 2] times 2.0)) otherwise 90;
    let $b: i32 = to.i32(($ds[pos: 1] plus $ds[pos: 2]) times 2) otherwise 91;
    let $c: i32 = to.i32(($ys[pos: 2] times 2.0) plus $ys[pos: 3] plus $ys[pos: 1] plus 10.0) otherwise 92;
    let $d: i32 = to.i32(($es[pos: 1] plus $es[pos: 2]) times 10) otherwise 93;
    no_check my.exit($a plus $b plus $c plus $d);
}
