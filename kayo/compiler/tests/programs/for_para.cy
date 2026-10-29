# `for para` makes a loop parallel by its writer's word (owner,
# 2026-09-30); CCA runs every loop in order so far, so this is 1 + 2 + 3 +
# 4 = 10, as the oracle gives.
%extern.C proc exit [$code: i32];
MAIN:
    let $xs: std::ls[i64] := [1, 2, 3, 4];
    var $total: i64 = 0;
    for %para $x in lend $xs:
        change $total to $total + $x;
    ;for
    no_check my.exit(to.i32($total));
;MAIN
