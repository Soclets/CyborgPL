# `for para` makes a loop parallel by its writer's word (owner,
# 2026-09-30); CCA runs every loop in order so far, so this is 1 + 2 + 3 +
# 4 = 10, as the oracle gives.
%extern.C fn exit [$code: i32];
MAIN {
    let $xs: std::list[i64] = [1, 2, 3, 4];
    let mut $total: i64 = 0;
    for %para $x in lend $xs {
        change $total = $total plus $x;
    }
    no_check my.exit(to.i32($total));
}
