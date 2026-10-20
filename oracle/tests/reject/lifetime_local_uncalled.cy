# a borrow of a local is refused at its `return`, even with no call (check claude, 2026-09-29)
fn b1 [lent 'a': list[i32]] -> lent list[i32] {
    let 'loc': list[i32] = [1, 2];
    return lend 'loc';
}
MAIN { }
