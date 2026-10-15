# check claude's borrow findings (2026-09-29): a borrow never aliases a change and never outlives its owner (core-features.md)
MAIN {
    let mut 's': list[int32] = [1, 2, 3];
    for 'x' in lend 's' {
        if 'x' equal_to 1 { list.append[lend.mut 's', value 5]; }
    }
}
