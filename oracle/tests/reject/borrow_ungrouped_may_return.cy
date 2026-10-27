# The result borrows both inputs, being in one group (since 2026-10-07 a function borrowing more than
# one input says which with a group). This run returns 'a', but 'b' could have been returned, so
# giving 't' away is refused.
function longer [@x] [lent @x $a: std::list[integer.signed.32_bit], lent @x $b: std::list[integer.signed.32_bit]] -> lent @x std::list[integer.signed.32_bit]:
    if std::list.length(lend $a) at_least std::list.length(lend $b):
        return lend $a;
    end of if
    return lend $b;
end of function longer
!MAIN:
    let $s: std::list[integer.signed.32_bit] = [1, 2, 3];
    let $t: std::list[integer.signed.32_bit] = [4,];
    let $r := longer(lend $s, lend $t);
    let $g := give $t;
    std::print.shortest(std::list.length(lend $r), newline);
!end of MAIN
