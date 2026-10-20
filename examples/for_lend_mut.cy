# Over `lend.mut 'xs'`, the loop's element is the list's own element: changing it changes the list
# (owner, 2026-09-30: "the program looks legal to me. `mut`'s already there"). Over `lend`, it cannot
# be changed.
MAIN {
    let mut 'prices': list[i32] = [10, 20, 30];
    for 'x' in lend.mut 'prices' {
        change 'x' = 'x' x 2;
    }
    std::print.stored['prices', newln];
}
