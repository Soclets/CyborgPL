# Finding 2026-09-29 (check run, seed 306, shrunk): a bare-number list item past int64 in a loop.
# CCA builds it as proven, then its --assert-proofs binary stops ("the value can be outside what
# a uint64 holds"). Same root as list-item-bare-numbers-*: bare numbers alone are exact (dc83c05).
MAIN {
    let 'w37': int32 = 4;
    while 'w37' > 0 {
        change 'w37' = 'w37' minus 1;
        let 'xs38': list[uint64] = [0 plus 18446744073709551581,];
    }
}
