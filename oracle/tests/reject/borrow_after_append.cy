# check claude's borrow findings (2026-09-29): a borrow never aliases a change and never outlives its owner (core-features.md)
fn first [lent 'a': list[i32]] -> lent list[i32] { return lend 'a'; }
MAIN {
    let mut 's': list[i32] = [1, 2, 3];
    let 'r' := first[lend 's'];
    list.append[lend.mut 's', value 4];
    std::print.useful[list.length[lend 'r'], newln];
}
