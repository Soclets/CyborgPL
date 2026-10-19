# A vector's lane has the lane's type, so a shift of it has a width (check claude, seed 42640), as a
# list item does: 's'[pos 2] is a uint64 here.
MAIN {
    let 's': simd[uint64, count 2] = simd.fill[value 40];
    if 3 < ('s'[pos 2] shr 2) { std::print.useful["10 is more than 3", newln]; }
}
