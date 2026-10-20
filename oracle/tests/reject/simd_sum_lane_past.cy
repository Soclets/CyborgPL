MAIN {
    let 'v': simd[i32, count 4] = simd.fill[value 1200000000];
    let 's': i64 = simd.sum.lane['v'];
}
