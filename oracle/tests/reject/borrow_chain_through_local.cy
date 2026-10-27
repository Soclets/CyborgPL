# Finding 2026-10-05 (check run --borrow-wrong 1000, seed 24149, cut down by hand): without @x a
# call's result borrows whatever its body may hand back on any path (oracle README), so 'r' may
# borrow 'o2' (b2's last return, through b1), and 's' := b1[…, lend 'r', lend 'r'] may too. Giving
# 'o2' away while 's' is used is a mistake: kayo refuses it. The oracle runs it (exit 0). Without
# 's' (the length of 'r' read directly) the oracle refuses too, so it is the borrow passed through
# the local 'r' into a second call that it loses.
function b1 [@x] [lent @x $a: std::list[integer.signed.32_bit], lent @x $b: std::list[integer.signed.32_bit], lent @x $c: std::list[integer.signed.32_bit]] -> lent @x std::list[integer.signed.32_bit]:
    return lend $c;
end of function b1

function b2 [@x] [lent @x $a: std::list[integer.signed.32_bit], lent @x $b: std::list[integer.signed.32_bit]] -> lent @x std::list[integer.signed.32_bit]:
    if std::list.length(lend $a) at_least 1:
        return lend $a;
    end of if
    return b1(lend $a, lend $a, lend $b);
end of function b2

!MAIN:
    let $o1: std::list[integer.signed.32_bit] = [2, negative 16];
    let $o2: std::list[integer.signed.32_bit] = [7,];
    let $r := b2(lend $o1, lend $o2);
    let $s := b1(lend $o1, lend $r, lend $r);
    let $g := give $o2;
    let $n := std::list.length(lend $s);
!end of MAIN
