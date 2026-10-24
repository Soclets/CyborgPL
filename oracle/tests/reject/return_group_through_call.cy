# A borrow handed back through a call is checked on every path, not only when it runs
# (check claude's finding, seed 12089, 2026-10-02): here the last return is never reached.
fn b1 [@x, @y] [lent @x $p: std::list[i8], lent @y $q: std::list[i8]] -> lent @x std::list[i8] { return lend $p; }
fn b2 [@x, @y] [lent @x $a: std::list[i8], lent @y $c: std::list[i8]] -> lent @x std::list[i8] {
    if std::list.length(lend $a) at_least 0 { return lend $a; }
    return b1(lend $c, lend $a);
}
MAIN {
    let $u: std::list[i8] = [1, 2];
    let $v: std::list[i8] = [3,];
    std::print.useful(b2(lend $u, lend $v), newln);
}
