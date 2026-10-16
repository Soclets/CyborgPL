# Finding 2026-10-05 (check run --borrow-wrong 1000, seed 24149, cut down by hand): without @x a
# call's result borrows whatever its body may hand back on any path (oracle README), so 'r' may
# borrow 'o2' (b2's last return, through b1), and 's' := b1[…, lend 'r', lend 'r'] may too. Giving
# 'o2' away while 's' is used is a mistake: kayo refuses it. The oracle runs it (exit 0). Without
# 's' (the length of 'r' read directly) the oracle refuses too, so it is the borrow passed through
# the local 'r' into a second call that it loses.
fn b1 [@x] [lent @x 'a': list[int32], lent @x 'b': list[int32], lent @x 'c': list[int32]] -> lent @x list[int32] {
    return lend 'c';
}

fn b2 [@x] [lent @x 'a': list[int32], lent @x 'b': list[int32]] -> lent @x list[int32] {
    if list.length[lend 'a'] at_least 1 {
        return lend 'a';
    }
    return b1[lend 'a', lend 'a', lend 'b'];
}

MAIN {
    let 'o1': list[int32] = [2, -16];
    let 'o2': list[int32] = [7,];
    let 'r' := b2[lend 'o1', lend 'o2'];
    let 's' := b1[lend 'o1', lend 'r', lend 'r'];
    let 'g' := give 'o2';
    let 'n' := list.length[lend 's'];
}
