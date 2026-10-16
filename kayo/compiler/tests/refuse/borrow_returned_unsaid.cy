# refuse: returns a borrow, so its signature must say so
# A borrow another call hands back, returned under `-> list[int8]`: the
# caller would own a list 's' still owns (check claude, seed 10731: built,
# printed 0, and freed the list twice).
fn f [lent 'a': list[int8]] -> lent list[int8] { return lend 'a'; }
fn g [lent 'a': list[int8]] -> list[int8] { return f[lend 'a']; }
MAIN {
    let mut 's': list[int8] = [1, 2, 3];
    let 'r' := g[lend 's'];
}
