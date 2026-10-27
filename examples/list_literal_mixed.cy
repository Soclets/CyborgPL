# In a list literal, a bare number takes its family from the other items (owner, 2026-09-30: a):
# the 1 here is a bin, like the 2.5 beside it. With `:=`, the prover still picks list or array and the width.
!MAIN:
    let $xs := [1, 2.5];
    let $ys: std::ls[float.bin.64] = [2.5, 1];
    std::print.shortest($xs, newln);
    std::print.shortest($ys, newln);
    # and it stays a bin when used: 1 plus 2.5 times 2.0
    std::print.shortest($xs[pos: 1] plus ($xs[pos: 2] times 2.0), newln);
    # beside a deci, a bare number is a deci
    let $q: float.%deci.64 = 0.5;
    let $ds := [1, $q];
    std::print.shortest($ds[pos: 1] plus $ds[pos: 2], newln);
!end of MAIN
