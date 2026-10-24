# refuse: returns a borrow, so its signature must say so
# A borrow another call hands back, returned under `-> list[i8]`: the
# caller would own a list 's' still owns (check claude, seed 10731: built,
# printed 0, and freed the list twice).
fn f [lent $a: std::list[i8]] -> lent std::list[i8] { return lend $a; }
fn g [lent $a: std::list[i8]] -> std::list[i8] { return f(lend $a); }
MAIN {
    let mut $s: std::list[i8] = [1, 2, 3];
    let $r := g(lend $s);
}
