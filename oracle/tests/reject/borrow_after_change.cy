# check claude's borrow findings (2026-09-29): a borrow never aliases a change and never outlives its owner (core-features.md)
MAIN {
    let mut 's': list[i32] = [1, 2, 3];
    let 'r' := lend 's';
    change 's'[pos 1] = 99;
    std::print.useful['r'[pos 1], newln];
}
