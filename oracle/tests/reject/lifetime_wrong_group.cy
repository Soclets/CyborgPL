# a returned borrow must be in the group it names (owner, 2026-09-29)
function wrong [@x, @y] [lent @x $a: string.utf8, lent @y $b: string.utf8] -> lent @x string.utf8:
    return lend $b;
end of function wrong
!MAIN:
    let $s := "a"; let $t := "b"; let $r := wrong(lend $s, lend $t);
!end of MAIN
