# A borrow handed back through a call with no group (check claude,
# 2026-09-29): `b1` borrows one input, so its group is inferred (owner,
# 2026-10-07: "`b`"), and `b2`'s result is in @x.
fn b1 [lent 'b': list[i32]] -> lent list[i32] {
    return lend 'b';
}
fn b2 [@x, @y] [lent @y 'a': list[i32], lent @x 'b': list[i32]] -> lent @x list[i32] {
    return b1[lend 'b'];
}
extern.C fn exit ['code': i32];
MAIN {
    let 's': list[i32] = [1,];
    let 't': list[i32] = [2, 3];
    let 'r' := b2[lend 's', lend 't'];
    no_check my.exit[to.i32[list.length[lend 'r']]];
}
