# The position beside the item (owner, 2026-09-29: a): `for $i, $x in lend $xs` — 'i' is
# 1, 2, 3 … (a u64, like every position), 'x' the item there. Position first, as in 'xs'['i'].
MAIN {
    let $xs := ["ann", "bob", "cy"];
    for $i, $x in lend $xs {
        std::print.useful($i, ". ", $x, newln);
    }
    let $scores := [70, 95, 88];
    let mut $best_at: u64 = 0;
    let mut $best: i32 = 0;
    for $i, $s in lend $scores {
        if $s > $best { change $best = $s; change $best_at = $i; }
    }
    std::print.useful("best is ", $best, " at position ", $best_at, newln);
    for perm $n, $y in lend $xs { }
    std::print.useful("after: ", $n, " ", $y, newln);
}
