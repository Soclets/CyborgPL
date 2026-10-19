# a mask has as many lanes as the vectors it compares: 2, 4, 8 or 16 here (2026-09-27)
MAIN {
    let 'm': simd[bool, count 3] = simd.fill[value true];
}
