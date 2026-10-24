# simd.load.some's k items must be in the list (2026-09-27)
MAIN {
    let $xs: std::list[i32] = [1, 2, 3];
    let $v: simd[i32, count: 4] = simd.load.some(lend $xs, pos: 2, count: 3);
}
