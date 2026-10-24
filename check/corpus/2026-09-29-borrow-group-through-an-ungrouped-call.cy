# Finding 2026-09-29 (check run, seeds 21, 98, 127, 175, 213, 280, cut down by hand): b1 has no
# groups, so Precog works out from its body what its result borrows (ownership-syntax.md): only
# 'b'. b2 lends its @x 'b' there, so b2's result is in @x. CCA refuses b2 ("returns a borrow
# of 'a', which is not in @x"), as if b1's result could borrow every argument. The oracle runs it.
fn b1 [@x] [lent @x 'a': std::list[i32], lent @x 'b': std::list[i32]] -> lent @x std::list[i32] {
    return lend 'b';
}
fn b2 [@y, @x] [lent @y 'a': std::list[i32], lent @x 'b': std::list[i32]] -> lent @x std::list[i32] {
    return b1(lend 'a', lend 'b');
}
MAIN {
    let 's': std::list[i32] = [1,];
    let 't': std::list[i32] = [2, 3];
    let 'r' := b2(lend 's', lend 't');
    let 'n' := std::list.length(lend 'r');
}
