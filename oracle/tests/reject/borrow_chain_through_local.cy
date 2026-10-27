# Finding 2026-10-05 (check run --borrow-wrong 1000, seed 24149, cut down by hand): without @x a
# call's result borrows whatever its body may hand back on any path (oracle README), so 'r' may
# borrow 'o2' (b2's last return, through b1), and 's' := b1[…, lend 'r', lend 'r'] may too. Giving
# 'o2' away while 's' is used is a mistake: kayo refuses it. The oracle runs it (exit 0). Without
# 's' (the length of 'r' read directly) the oracle refuses too, so it is the borrow passed through
# the local 'r' into a second call that it loses.
fn b1 [@x] [lent @x $a: std::ls[int.sign.32], lent @x $b: std::ls[int.sign.32], lent @x $c: std::ls[int.sign.32]] -> lent @x std::ls[int.sign.32]:
    ret lend $c;
end of fn b1

fn b2 [@x] [lent @x $a: std::ls[int.sign.32], lent @x $b: std::ls[int.sign.32]] -> lent @x std::ls[int.sign.32]:
    if std::ls.len(lend $a) at_least 1:
        ret lend $a;
    end of if
    ret b1(lend $a, lend $a, lend $b);
end of fn b2

!MAIN:
    let $o1: std::ls[int.sign.32] = [2, neg 16];
    let $o2: std::ls[int.sign.32] = [7,];
    let $r := b2(lend $o1, lend $o2);
    let $s := b1(lend $o1, lend $r, lend $r);
    let $g := give $o2;
    let $n := std::ls.len(lend $s);
!end of MAIN
