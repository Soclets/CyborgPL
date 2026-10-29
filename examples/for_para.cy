# `for %para` is MCPS's loop: run in parallel where Precog proves it safe and worth the cores, in order otherwise; a plain
# `for` always runs in order, with MCPS or without, so `seq` is gone (owner, 2026-10-29: "(a)", every limb marked where it
# acts; `for para` / `for seq` from 2026-09-30 before). The oracle runs every loop in order.
MAIN:
    let $xs: std::ls[i64] := [1, 2, 3, 4];
    var $total: i64 = 0;
    for %para $x in lend $xs:
        change $total to $total + $x;
    ;for
    for $x in lend $xs:
        std::print.stored($x, " ");
    ;for
    std::print.stored($total, newln);
;MAIN
