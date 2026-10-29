# What running the examples found once `std::print` was built (owner,
# 2026-10-09), each freed exactly once, as the leak check here says: a
# struct whose field was given away keeps and frees the rest; a list
# literal lent to a call is freed after it; a text `to.str.utf8` made with
# an `otherwise` is freed; a nested list is moved into its list, not freed
# twice; a `for … in lend` over owning elements borrows them.
%extern.C proc exit [$code: i32];

rec L:
    var owner: str.utf8;
    var items: std::ls[i32];
;rec L

fn count [lent $xs: std::ls[i32]] -> i32:
    ret to.i32(std::ls.len(lend $xs));
;fn count

MAIN:
    let $m := L[owner = to.stored.str.utf8(5), items = [1, 2, 3]];
    let $t := give $m.owner;
    let $c := count([1, 2, 3, 4]);
    let $good: std::ls[u8] := [104, 105];
    let $a := to.str.utf8(lend $good) ?? "?";
    let $zs: std::ls[std::ls[u8]] := [[2,], [1, 3]];
    var $n: i32 = 0;
    for $z in lend $zs:
        change $n to $n + to.i32(std::ls.len(lend $z));
    ;for
    let $ls := [L[owner = "a", items = [1,]], L[owner = "b", items = std::ls.empty]];
    for $l in lend $ls:
        change $n to $n + to.i32(std::ls.len(lend $l.items));
    ;for
    # 4 + 2 + 1 + 3 + 1 = 11
    no_check my.exit($c + to.i32(std::str.bytes(lend $a)) + to.i32(std::str.bytes(lend $t)) + $n);
;MAIN
