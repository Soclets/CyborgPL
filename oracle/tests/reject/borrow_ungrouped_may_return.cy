# The result borrows both inputs, being in one group (since 2026-10-07 a function borrowing more than
# one input says which with a group). This run returns 'a', but 'b' could have been returned, so
# giving 't' away is refused.
fn longer [@x] [lent @x $a: std::list[i32], lent @x $b: std::list[i32]] -> lent @x std::list[i32] {
    if std::list.length(lend $a) at_least std::list.length(lend $b) { return lend $a; }
    return lend $b;
}
MAIN {
    let $s: std::list[i32] = [1, 2, 3];
    let $t: std::list[i32] = [4,];
    let $r := longer(lend $s, lend $t);
    let $g := give $t;
    std::print.useful(std::list.length(lend $r), newln);
}
