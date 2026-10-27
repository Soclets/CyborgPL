# A borrow handed back through a call is checked on every path, not only when it runs
# (check claude's finding, seed 12089, 2026-10-02): here the last return is never reached.
fn b1 [@x, @y] [lent @x $p: std::ls[int.sign.8], lent @y $q: std::ls[int.sign.8]] -> lent @x std::ls[int.sign.8]:
    ret lend $p;
end of fn b1
fn b2 [@x, @y] [lent @x $a: std::ls[int.sign.8], lent @y $c: std::ls[int.sign.8]] -> lent @x std::ls[int.sign.8]:
    if std::ls.len(lend $a) at_least 0:
        ret lend $a;
    end of if
    ret b1(lend $c, lend $a);
end of fn b2
!MAIN:
    let $u: std::ls[int.sign.8] = [1, 2];
    let $v: std::ls[int.sign.8] = [3,];
    std::print.shortest(b2(lend $u, lend $v), newln);
!end of MAIN
