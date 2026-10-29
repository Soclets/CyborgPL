# The position beside the item (owner, 2026-09-29: a): `for 'i', 'x' in
# lend 'xs'` — 'i' is 1, 2, 3 … (a u64), 'x' the item there; `perm`
# keeps both after the loop (examples/for_position.cpls as an exit code:
# best at 2, and after the loop 3 and 88 → 2 x 100 + 3 x 10 + 8 = 238).
%extern.C proc exit [$code: i32];
MAIN:
    let $scores: std::ls[i32] := [70, 95, 88];
    var $best_at: u64 = 0;
    var $best: i32 = 0;
    for $i, $s in lend $scores:
        if $s > $best:
            change $best to $s; change $best_at to $i;
        ;if
    ;for
    for perm $n, $y in lend $scores:
    ;for
    no_check my.exit(to.i32($best_at) x 100 + to.i32($n) x 10 + $y - 80);
;MAIN
