# The position beside the item (owner, 2026-09-29: a): `for $i, $x in lend $xs` — 'i' is
# 1, 2, 3 … (a u64, like every position), 'x' the item there. Position first, as in 'xs'['i'].
!MAIN:
    let $xs := ["ann", "bob", "cy"];
    for $i, $x in lend $xs:
        std::print.shortest($i, ". ", $x, newln);
    end of for
    let $scores := [70, 95, 88];
    var $best_at: int.unsign.64 = 0;
    var $best: int.sign.32 = 0;
    for $i, $s in lend $scores:
        if $s greater_than $best:
            change $best to $s; change $best_at to $i;
        end of if
    end of for
    std::print.shortest("best is ", $best, " at position ", $best_at, newln);
    for perm $n, $y in lend $xs:
    end of for
    std::print.shortest("after: ", $n, " ", $y, newln);
!end of MAIN
