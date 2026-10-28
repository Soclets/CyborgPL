# The position beside the item (owner, 2026-09-29: a): `for $i, $x in lend $xs` — 'i' is
# 1, 2, 3 … (a u64, like every position), 'x' the item there. Position first, as in 'xs'['i'].
MAIN:
    let $xs := ["ann", "bob", "cy"];
    for $i, $x in lend $xs:
        std::println($i, ". ", $x);
    ;for
    let $scores := [70, 95, 88];
    var $best_at: u64 = 0;
    var $best: i32 = 0;
    for $i, $s in lend $scores:
        if $s > $best:
            change $best to $s; change $best_at to $i;
        ;if
    ;for
    std::println("best is ", $best, " at position ", $best_at);
    for perm $n, $y in lend $xs:
    ;for
    std::println("after: ", $n, " ", $y);
;MAIN
