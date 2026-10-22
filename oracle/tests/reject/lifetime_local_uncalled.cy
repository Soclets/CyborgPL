# a borrow of a local is refused at its `return`, even with no call (check claude, 2026-09-29)
fn b1 [lent 'a': std::list[i32]] -> lent std::list[i32] {
    let 'loc': std::list[i32] = [1, 2];
    return lend 'loc';
}
MAIN { }
