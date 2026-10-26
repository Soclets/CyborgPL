# A borrow of a local that borrows a `lent` parameter is a borrow of that
# parameter, and may be handed back (the oracle's reading, 0e5c5b1): 'r' is
# 's' itself, so 'r'[3] is 3.
%extern.C fn exit [$code: i32];
fn first [lent $a: std::list[i32]] -> lent std::list[i32] {
    let $l := lend $a;
    return lend $l;
}
MAIN {
    let $s: std::list[i32] = [1, 2, 3];
    let $r := first(lend $s);
    no_check my.exit($r[pos: 3]);
}
