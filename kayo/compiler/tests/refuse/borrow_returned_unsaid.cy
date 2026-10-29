# refuse: returns a borrow, so its signature must say so
# A borrow another call hands back, returned under `-> list[i8]`: the
# caller would own a list 's' still owns (check claude, seed 10731: built,
# printed 0, and freed the list twice).
fn f [lent $a: std::ls[i8]] -> lent std::ls[i8]:
    ret lend $a;
;fn f
fn g [lent $a: std::ls[i8]] -> std::ls[i8]:
    ret f(lend $a);
;fn g
MAIN:
    var $s: std::ls[i8] = [1, 2, 3];
    let $r := g(lend $s);
;MAIN
