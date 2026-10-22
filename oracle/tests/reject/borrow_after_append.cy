# check claude's borrow findings (2026-09-29): a borrow never aliases a change and never outlives its owner (core-features.md)
fn first [lent 'a': std::list[i32]] -> lent std::list[i32] { return lend 'a'; }
MAIN {
    let mut 's': std::list[i32] = [1, 2, 3];
    let 'r' := first[lend 's'];
    std::list.append[lend.mut 's', value: 4];
    std::print.useful[std::list.length[lend 'r'], newln];
}
