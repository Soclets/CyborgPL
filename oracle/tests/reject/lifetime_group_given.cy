# `@x`: the result lives while everything in the group does, whichever the body returned (check claude, 2026-09-29)
fn b1 [@x] [lent @x $a: std::ls[int.sign.32], lent @x $b: std::ls[int.sign.32]] -> lent @x std::ls[int.sign.32]:
    ret lend $a;
end of fn b1
!MAIN:
    let $s: std::ls[int.sign.32] := [5,];
    let $t: std::ls[int.sign.32] := [6, 7];
    let $r := b1(lend $s, lend $t);
    let $g := give $t;
    std::print.shortest($r[pos: 1], newln);
!end of MAIN
