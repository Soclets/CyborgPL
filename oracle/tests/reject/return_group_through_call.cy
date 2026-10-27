# A borrow handed back through a call is checked on every path, not only when it runs
# (check claude's finding, seed 12089, 2026-10-02): here the last return is never reached.
function b1 [@x, @y] [lent @x $p: std::list[integer.signed.8_bit], lent @y $q: std::list[integer.signed.8_bit]] -> lent @x std::list[integer.signed.8_bit]:
    return lend $p;
end of function b1
function b2 [@x, @y] [lent @x $a: std::list[integer.signed.8_bit], lent @y $c: std::list[integer.signed.8_bit]] -> lent @x std::list[integer.signed.8_bit]:
    if std::list.length(lend $a) at_least 0:
        return lend $a;
    end of if
    return b1(lend $c, lend $a);
end of function b2
!MAIN:
    let $u: std::list[integer.signed.8_bit] = [1, 2];
    let $v: std::list[integer.signed.8_bit] = [3,];
    std::print.shortest(b2(lend $u, lend $v), newline);
!end of MAIN
