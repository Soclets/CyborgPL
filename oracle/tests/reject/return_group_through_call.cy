# A borrow handed back through a call is checked on every path, not only when it runs
# (check claude's finding, seed 12089, 2026-10-02): here the last return is never reached.
fn b1 [@x, @y] [lent @x 'p': list[i8], lent @y 'q': list[i8]] -> lent @x list[i8] { return lend 'p'; }
fn b2 [@x, @y] [lent @x 'a': list[i8], lent @y 'c': list[i8]] -> lent @x list[i8] {
    if list.length[lend 'a'] at_least 0 { return lend 'a'; }
    return b1[lend 'c', lend 'a'];
}
MAIN {
    let 'u': list[i8] = [1, 2];
    let 'v': list[i8] = [3,];
    std::print.useful[b2[lend 'u', lend 'v'], newln];
}
