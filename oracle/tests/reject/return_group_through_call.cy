# A borrow handed back through a call is checked on every path, not only when it runs
# (check claude's finding, seed 12089, 2026-10-02): here the last return is never reached.
fn b1 [@x, @y] [lent @x $p: std::ls[i8], lent @y $q: std::ls[i8]] -> lent @x std::ls[i8]:
    ret lend $p;
;fn b1
fn b2 [@x, @y] [lent @x $a: std::ls[i8], lent @y $c: std::ls[i8]] -> lent @x std::ls[i8]:
    if std::ls.len(lend $a) >= 0:
        ret lend $a;
    ;if
    ret b1(lend $c, lend $a);
;fn b2
MAIN:
    let $u: std::ls[i8] := [1, 2];
    let $v: std::ls[i8] := [3,];
    std::print(b2(lend $u, lend $v), newln);
;MAIN
