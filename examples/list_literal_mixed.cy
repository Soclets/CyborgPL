# In a list literal, a bare number takes its family from the other items (owner, 2026-09-30: a):
# the 1 here is a bin, like the 2.5 beside it. With `:=`, the prover still picks list or array and the width.
MAIN {
    let 'xs' := [1, 2.5];
    let 'ys': list[f64] = [2.5, 1];
    std::print.useful['xs', newln];
    std::print.useful['ys', newln];
    # and it stays a bin when used: 1 plus 2.5 times 2.0
    std::print.useful['xs'[pos: 1] + ('xs'[pos: 2] x 2.0), newln];
    # beside a deci, a bare number is a deci
    let 'q': d64 = 0.5;
    let 'ds' := [1, 'q'];
    std::print.useful['ds'[pos: 1] + 'ds'[pos: 2], newln];
}
