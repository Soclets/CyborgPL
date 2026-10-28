# `@x`: the result lives while everything in the group does, whichever the body returned (check claude, 2026-09-29)
fn b1 [@x] [lent @x $a: std::ls[i32], lent @x $b: std::ls[i32]] -> lent @x std::ls[i32]:
    ret lend $a;
;fn b1
MAIN:
    let $s: std::ls[i32] := [5,];
    let $t: std::ls[i32] := [6, 7];
    let $r := b1(lend $s, lend $t);
    let $g := give $t;
    std::println($r[pos: 1]);
;MAIN
