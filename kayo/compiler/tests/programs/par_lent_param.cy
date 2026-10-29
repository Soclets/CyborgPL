# Loops in slices over a lent parameter: its slot is the address it borrows
# already, so the slices take a copy of what is there, not of the address.
# The oracle: 64.
%extern.C proc exit [let $code: i32];
fn total [lent $xs: std::ls[i64]] -> i64:
    var $acc: i64 = 0;
    for %para $x in lend $xs:
        change $acc to $acc + ($x mod.floor 7);
    ;for
    ret $acc;
;fn total
proc scale [lent.mut $xs: std::ls[i64], let $k: i64]:
    for %para $i: u64 [from: 1, to: std::ls.len(lend $xs), by: 1]:
        change $xs[pos: $i] to ($xs[pos: $i] x $k) mod.floor 1000;
    ;for
;proc scale
MAIN:
    var $xs: std::ls[i64] = std::ls.fill(value: 3, count: 200000);
    scale(lend.mut $xs, 5);
    let $a := total(lend $xs);
    no_check my.exit(to.i32($a mod.floor 256));
;MAIN
