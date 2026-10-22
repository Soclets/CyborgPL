# Finding 2026-09-29 (by hand): a function that hands back a borrow of a local is refused by
# CCA even when nothing calls it (line 3 here); the oracle checks it only at a call, so this
# program runs. Called, the oracle points at the call, CCA at the return.
fn b1 [lent 'a': std::list[i32]] -> lent std::list[i32] {
    let 'loc': std::list[i32] = [1, 2];
    return lend 'loc';
}
MAIN {
    let 's': std::list[i32] = [5,];
}
