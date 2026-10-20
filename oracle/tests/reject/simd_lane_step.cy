# a vector's lanes are checked at every operation at the lane's type (owner, 2026-09-29: b)
MAIN {
    let 'a': simd[u8, count 16] = simd.fill[value 200];
    let 'b': simd[u8, count 16] = ('a' plus 'a') minus 'a';
}
