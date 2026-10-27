# `@x`: the result lives while everything in the group does, whichever the body returned (check claude, 2026-09-29)
function b1 [@x] [lent @x $a: std::list[integer.signed.32_bit], lent @x $b: std::list[integer.signed.32_bit]] -> lent @x std::list[integer.signed.32_bit]:
    return lend $a;
end of function b1
!MAIN:
    let $s: std::list[integer.signed.32_bit] = [5,];
    let $t: std::list[integer.signed.32_bit] = [6, 7];
    let $r := b1(lend $s, lend $t);
    let $g := give $t;
    std::print.shortest($r[position: 1], newline);
!end of MAIN
