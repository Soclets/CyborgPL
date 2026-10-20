# A `lent.mut` borrow handed back changes what was lent (owner, 2026-09-29):
# 'xs' becomes [7, 2, 9], so 7 x 10 + 3 = 73 (the oracle agrees).
extern.C fn exit ['code': i32];
fn pickmut [@x] [lent.mut @x 'a': list[i32], lent 'b': list[i32]] -> lent.mut @x list[i32] {
    return lend.mut 'a';
}
fn run [] -> i32 {
    let mut 'xs': list[i32] = [1, 2];
    let 'ys': list[i32] = [5, 6];
    let 'm' := pickmut[lend.mut 'xs', lend 'ys'];
    change 'm'[pos 1] = 7;
    list.append[lend.mut 'm', value 9];
    return 'xs'[pos 1] times 10 plus to.i32[list.length[lend 'xs']];
}
MAIN {
    no_check my.exit[run[]];
}
