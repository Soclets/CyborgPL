# In a list literal, a bare number takes its family from the other items (owner, 2026-09-30: a):
# the 1 here is a bin, like the 2.5 beside it. With `:=`, the prover still picks list or array and the width.
!MAIN:
    let $xs := [1, 2.5];
    let $ys: std::list[float.binary.64_bit] = [2.5, 1];
    std::print.shortest($xs, newline);
    std::print.shortest($ys, newline);
    # and it stays a bin when used: 1 plus 2.5 times 2.0
    std::print.shortest($xs[position: 1] plus ($xs[position: 2] times 2.0), newline);
    # beside a deci, a bare number is a deci
    let $q: float.%decimal.64_bit = 0.5;
    let $ds := [1, $q];
    std::print.shortest($ds[position: 1] plus $ds[position: 2], newline);
!end of MAIN
