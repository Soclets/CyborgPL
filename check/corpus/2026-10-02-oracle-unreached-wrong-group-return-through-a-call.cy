# Finding 2026-10-02 (check run --borrow-wrong, seed 12089, cut down by hand): `b2`'s last return hands
# back a borrow of 'c' (@y) through b1, under -> lent@x. The oracle runs it (exit 0), since that line is
# never reached; with the `if` not taken (>/== 5) it refuses: "`b2` returns a borrow of 'c', which is not
# in @x". A direct `return lend 'c'` there is refused before running. CCA refuses it as written.
fn b1 [@x, @y] [lent @x 'a': std::list[i8], lent @y 'b': std::list[i8]] -> lent @x std::list[i8] {
    return lend 'a';
}

fn b2 [@x, @y] [lent @x 'a': std::list[i8], lent @y 'c': std::list[i8]] -> lent @x std::list[i8] {
    if std::list.length(lend 'a') at_least 0 {
        return lend 'a';
    }
    return b1(lend 'c', lend 'a');
}

MAIN {
    let 'o1': std::list[i8] = [1, 2];
    let 'o2': std::list[i8] = [3,];
    let 'r' := b2(lend 'o1', lend 'o2');
    let 'n' := std::list.length(lend 'r');
}
