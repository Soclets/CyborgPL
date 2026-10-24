# `for para` forces a loop parallel, `for seq` forces it in order (owner, 2026-09-30); without either,
# the prover decides, and a loop that prints stays in order by itself. The oracle runs every loop in order.
MAIN {
    let $xs: std::list[i64] = [1, 2, 3, 4];
    let mut $total: i64 = 0;
    for para $x in lend $xs {
        change $total = $total + $x;
    }
    for seq $x in lend $xs {
        std::print.stored($x, " ");
    }
    std::print.stored($total, newln);
}
