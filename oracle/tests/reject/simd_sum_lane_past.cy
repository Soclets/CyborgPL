MAIN {
    let 'v': simd[int32, count 4] = simd.fill[value 1200000000];
    let 's': int64 = simd.sum.lane['v'];
}
