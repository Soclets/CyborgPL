# `@x`: the result lives while everything in the group does, whichever the body returned (check claude, 2026-09-29)
fn b1 [@x] [lent @x 'a': std::list[i32], lent @x 'b': std::list[i32]] -> lent @x std::list[i32] { return lend 'a'; }
MAIN {
    let 's': std::list[i32] = [5,];
    let 't': std::list[i32] = [6, 7];
    let 'r' := b1[lend 's', lend 't'];
    let 'g' := give 't';
    std::print.useful['r'[pos: 1], newln];
}
