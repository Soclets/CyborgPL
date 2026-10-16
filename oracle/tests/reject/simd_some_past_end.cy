# simd.load.some's k items must be in the list (2026-09-27)
MAIN {
    let 'xs': list[int32] = [1, 2, 3];
    let 'v': simd[int32, 4] = simd.load.some[lend 'xs', pos 2, count 3];
}
