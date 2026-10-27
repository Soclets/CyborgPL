# The result borrows both inputs, being in one group (since 2026-10-07 a function borrowing more than
# one input says which with a group). This run returns 'a', but 'b' could have been returned, so
# giving 't' away is refused.
fn longer [@x] [lent @x $a: std::ls[int.sign.32], lent @x $b: std::ls[int.sign.32]] -> lent @x std::ls[int.sign.32]:
    if std::ls.len(lend $a) at_least std::ls.len(lend $b):
        ret lend $a;
    end of if
    ret lend $b;
end of fn longer
!MAIN:
    let $s: std::ls[int.sign.32] = [1, 2, 3];
    let $t: std::ls[int.sign.32] = [4,];
    let $r := longer(lend $s, lend $t);
    let $g := give $t;
    std::print.shortest(std::ls.len(lend $r), newln);
!end of MAIN
