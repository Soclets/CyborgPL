# A borrow handed back through a call with no group (check claude,
# 2026-09-29): `b1` borrows one input, so its group is inferred (owner,
# 2026-10-07: "`b`"), and `b2`'s result is in @x.
fn b1 [lent $b: std::list[i32]] -> lent std::list[i32] {
    return lend $b;
}
fn b2 [@x, @y] [lent @y $a: std::list[i32], lent @x $b: std::list[i32]] -> lent @x std::list[i32] {
    return b1(lend $b);
}
%extern.C fn exit [$code: i32];
MAIN {
    let $s: std::list[i32] = [1,];
    let $t: std::list[i32] = [2, 3];
    let $r := b2(lend $s, lend $t);
    no_check my.exit(to.i32(std::list.length(lend $r)));
}
