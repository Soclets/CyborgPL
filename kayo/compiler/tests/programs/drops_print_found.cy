# What running the examples found once `std::print` was built (owner,
# 2026-10-09), each freed exactly once, as the leak check here says: a
# struct whose field was given away keeps and frees the rest; a list
# literal lent to a call is freed after it; a text `to.str.utf8` made with
# an `otherwise` is freed; a nested list is moved into its list, not freed
# twice; a `for … in lend` over owning elements borrows them.
%extern.C fn exit [$code: i32];

struct L {
    owner: str.utf8,
    items: std::list[i32]
}

fn count [lent $xs: std::list[i32]] -> i32 {
    return to.i32(std::list.length(lend $xs));
}

MAIN {
    let $m := L[owner = to.stored.str.utf8(5), items = [1, 2, 3]];
    let $t := give $m.owner;
    let $c := count([1, 2, 3, 4]);
    let $good: std::list[u8] = [104, 105];
    let $a := to.str.utf8($good) otherwise "?";
    let $zs: std::list[std::list[u8]] = [[2,], [1, 3]];
    let mut $n: i32 = 0;
    for $z in lend $zs {
        change $n = $n plus to.i32(std::list.length(lend $z));
    }
    let $ls := [L[owner = "a", items = [1,]], L[owner = "b", items = std::list.empty]];
    for $l in lend $ls {
        change $n = $n plus to.i32(std::list.length(lend $l.items));
    }
    # 4 + 2 + 1 + 3 + 1 = 11
    no_check my.exit($c plus to.i32(std::str.bytes(lend $a)) plus to.i32(std::str.bytes(lend $t)) plus $n);
}
