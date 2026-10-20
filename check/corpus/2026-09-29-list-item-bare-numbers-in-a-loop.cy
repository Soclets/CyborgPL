# Finding 2026-09-29 (check run, seed 306, shrunk): a bare-number list item past i64 in a loop.
# CCA builds it as proven, then its --assert-proofs binary stops ("the value can be outside what
# a u64 holds"). Same root as list-item-bare-numbers-*: bare numbers alone are exact (dc83c05).
MAIN {
    let 'w37': i32 = 4;
    while 'w37' > 0 {
        change 'w37' = 'w37' minus 1;
        let 'xs38': list[u64] = [0 plus 18446744073709551581,];
    }
}
