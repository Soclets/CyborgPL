# A `#strict` file (owner, 2026-09-27): `pos` on every position, every
# parameter's and return type, `mut`/`immut` on every declaration,
# `temp`/`perm` on every `for`, labelled ranges, and each site proven from its own function —
# the range on 'n' is why 100 / 'n' is safe (examples/strict.cpls, read out
# as an exit code; the closure's 'n' carries a range, so its multiply is
# proven there too). 100 + (20 + 30) + 25 + 6 = 181.
%extern.C proc exit [$code: i32];
fn ratio [$n: i32[min: 1]] -> i32:
    ret 100 div.floor $n;
;fn ratio
MAIN:
    let $xs: std::ls[i32] := [10, 20, 30, 40];
    var $total: i32 = 0;
    for $i: u64 [from: 1, to: 4, by: 1]:
        change $total to $total + $xs[pos: $i];
    ;for
    let $mid := std::ls.slice(lend $xs, from: pos 2, to: pos 3);
    let $f := fn [$n: i32[min: 0, max: 1000]] -> i32:
        ret $n x 2;
    ;fn;
    let $r := ratio(4);
    let $six := $f[pos: 3];
    no_check my.exit($total + $mid[pos: 1] + $mid[pos: 2] + $r + $six);
;MAIN
